//! Results: the winners recorded for a category once the ballots are locked.

use std::collections::{BTreeMap, BTreeSet};

use crate::season::{CategoryKey, NomineeKey, Season};
use crate::time::UtcInstant;

/// The winners of one category: one nominee, or several for a tie (spec rule 1.3.2).
///
/// Invariant: never empty, and every nominee belongs to the category it was checked for.
/// Only [`Season::winner_set`] builds one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WinnerSet(BTreeSet<NomineeKey>);

impl WinnerSet {
    pub fn contains(&self, nominee: &NomineeKey) -> bool {
        self.0.contains(nominee)
    }

    pub fn nominees(&self) -> impl Iterator<Item = &NomineeKey> {
        self.0.iter()
    }
}

/// Why a set of winners cannot be recorded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResultRefusal {
    /// Results open at the lock instant (spec rule 1.3.3).
    #[error("results open at the lock instant")]
    BeforeLock,
    #[error("the season has no such category")]
    NoSuchCategory,
    #[error("a result needs at least one winner")]
    NoWinners,
    #[error("nominee `{0}` is not in that category")]
    NomineeNotInCategory(NomineeKey),
}

impl Season {
    /// Whether results may be recorded or cleared at `now`: from the lock instant on (spec rule 1.3.3).
    pub fn results_are_open(&self, now: UtcInstant) -> bool {
        now >= self.lock_instant()
    }

    /// Checks `winners` as the result of `category` at `now` (spec rules 1.3.2 and 1.3.3).
    /// Naming a winner twice is the same as naming it once.
    pub fn winner_set(
        &self,
        now: UtcInstant,
        category: &CategoryKey,
        winners: impl IntoIterator<Item = NomineeKey>,
    ) -> Result<WinnerSet, ResultRefusal> {
        if !self.results_are_open(now) {
            return Err(ResultRefusal::BeforeLock);
        }
        let category = self.category(category).ok_or(ResultRefusal::NoSuchCategory)?;
        let winners: BTreeSet<NomineeKey> = winners.into_iter().collect();
        if winners.is_empty() {
            return Err(ResultRefusal::NoWinners);
        }
        if let Some(stranger) = winners.iter().find(|winner| category.nominee(winner).is_none()) {
            return Err(ResultRefusal::NomineeNotInCategory(stranger.clone()));
        }
        Ok(WinnerSet(winners))
    }
}

/// The results recorded so far in one season: at most one winner set per category.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Results(BTreeMap<CategoryKey, WinnerSet>);

impl Results {
    /// Records the winners of `category`, replacing any earlier result.
    pub fn record(&mut self, category: CategoryKey, winners: WinnerSet) {
        self.0.insert(category, winners);
    }

    pub fn winners(&self, category: &CategoryKey) -> Option<&WinnerSet> {
        self.0.get(category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{LOCK, category_key, nominee_key, season};

    fn before_lock() -> UtcInstant {
        UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() - 1)
    }

    #[test]
    fn rule_1_3_2_one_winner_is_a_result() {
        let season = season(&[("picture", &["a", "b"])]);

        let winners = season
            .winner_set(LOCK, &category_key("picture"), [nominee_key("a")])
            .unwrap();

        assert!(winners.contains(&nominee_key("a")));
        assert!(!winners.contains(&nominee_key("b")));
    }

    #[test]
    fn rule_1_3_2_two_winners_can_be_recorded_for_one_category() {
        let season = season(&[("picture", &["a", "b", "c"])]);

        let winners = season
            .winner_set(LOCK, &category_key("picture"), [nominee_key("c"), nominee_key("a")])
            .unwrap();

        assert_eq!(
            winners.nominees().collect::<Vec<_>>(),
            [&nominee_key("a"), &nominee_key("c")]
        );
    }

    #[test]
    fn a_winner_named_twice_counts_once() {
        let season = season(&[("picture", &["a", "b"])]);
        let picture = category_key("picture");

        let named_twice = season.winner_set(LOCK, &picture, [nominee_key("a"), nominee_key("a")]);

        assert_eq!(named_twice, season.winner_set(LOCK, &picture, [nominee_key("a")]));
        assert_eq!(named_twice.unwrap().nominees().count(), 1);
    }

    #[test]
    fn rule_1_3_2_a_result_needs_at_least_one_winner() {
        let season = season(&[("picture", &["a"])]);

        assert_eq!(
            season.winner_set(LOCK, &category_key("picture"), []),
            Err(ResultRefusal::NoWinners)
        );
    }

    #[test]
    fn rule_1_3_2_a_winner_from_another_category_is_refused() {
        let season = season(&[("picture", &["a"]), ("director", &["b"])]);

        assert_eq!(
            season.winner_set(LOCK, &category_key("picture"), [nominee_key("a"), nominee_key("b")]),
            Err(ResultRefusal::NomineeNotInCategory(nominee_key("b")))
        );
    }

    #[test]
    fn rule_1_3_2_an_unknown_category_takes_no_result() {
        let season = season(&[("picture", &["a"])]);

        assert_eq!(
            season.winner_set(LOCK, &category_key("costume"), [nominee_key("a")]),
            Err(ResultRefusal::NoSuchCategory)
        );
    }

    #[test]
    fn rule_1_3_3_results_are_refused_before_the_lock_instant() {
        let season = season(&[("picture", &["a"])]);

        assert_eq!(
            season.winner_set(before_lock(), &category_key("picture"), [nominee_key("a")]),
            Err(ResultRefusal::BeforeLock)
        );
        assert!(!season.results_are_open(before_lock()));
        assert!(season.results_are_open(LOCK));
    }

    #[test]
    fn every_result_refusal_says_why() {
        let refusals = [
            (ResultRefusal::BeforeLock, "results open at the lock instant"),
            (ResultRefusal::NoSuchCategory, "the season has no such category"),
            (ResultRefusal::NoWinners, "a result needs at least one winner"),
            (
                ResultRefusal::NomineeNotInCategory(nominee_key("b")),
                "nominee `b` is not in that category",
            ),
        ];

        for (refusal, message) in refusals {
            assert_eq!(refusal.to_string(), message);
        }
    }

    #[test]
    fn a_later_result_replaces_the_earlier_one() {
        let season = season(&[("picture", &["a", "b"])]);
        let picture = category_key("picture");
        let mut results = Results::default();

        results.record(
            picture.clone(),
            season.winner_set(LOCK, &picture, [nominee_key("a")]).unwrap(),
        );
        results.record(
            picture.clone(),
            season.winner_set(LOCK, &picture, [nominee_key("b")]).unwrap(),
        );

        assert!(results.winners(&picture).unwrap().contains(&nominee_key("b")));
        assert!(!results.winners(&picture).unwrap().contains(&nominee_key("a")));
        assert_eq!(results.winners(&category_key("director")), None);
    }
}
