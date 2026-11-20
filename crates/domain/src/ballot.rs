//! A player's ballot: in each category, at most one will-win and one should-win pick.

use std::collections::BTreeMap;

use crate::season::{CategoryKey, NomineeKey, Season};
use crate::time::UtcInstant;

/// The two questions a player answers per category. Only will-win scores (spec rule 1.4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKind {
    WillWin,
    ShouldWin,
}

/// One player's picks in one category. The two are independent and may name the same nominee
/// (spec rule 1.2.3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CategoryPicks {
    pub will_win: Option<NomineeKey>,
    pub should_win: Option<NomineeKey>,
}

/// Why a pick may not be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PickRefusal {
    /// The nominees are not seeded yet.
    #[error("the season has no nominees yet")]
    NotOpen,
    /// The lock instant has been reached (spec rule 1.2.5).
    #[error("the ballot is locked")]
    Locked,
    #[error("the season has no such category")]
    NoSuchCategory,
    /// The nominee belongs to another category, or to none (spec rule 1.2.2).
    #[error("the nominee is not in that category")]
    NomineeNotInCategory,
}

impl Season {
    /// Whether a player may, at `now`, set the pick in `category` to `nominee`, or clear it with `None`
    /// (spec rules 1.2.2, 1.2.4 and 1.2.5). The same answer holds for both kinds of pick.
    ///
    /// It never allows a write at or after the lock instant.
    pub fn check_pick_write(
        &self,
        now: UtcInstant,
        category: &CategoryKey,
        nominee: Option<&NomineeKey>,
    ) -> Result<(), PickRefusal> {
        if !self.has_nominees() {
            return Err(PickRefusal::NotOpen);
        }
        if now >= self.lock_instant() {
            return Err(PickRefusal::Locked);
        }
        let category = self.category(category).ok_or(PickRefusal::NoSuchCategory)?;
        match nominee {
            Some(nominee) if category.nominee(nominee).is_none() => Err(PickRefusal::NomineeNotInCategory),
            _ => Ok(()),
        }
    }
}

/// One player's picks in one season.
///
/// It stores what it is given: the caller checks each write with [`Season::check_pick_write`] first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ballot(BTreeMap<CategoryKey, CategoryPicks>);

impl Ballot {
    /// Sets the pick of `kind` in `category`, or clears it with `None`. It replaces what was there,
    /// so a category never holds two picks of one kind.
    pub fn write(&mut self, category: CategoryKey, kind: PickKind, nominee: Option<NomineeKey>) {
        let picks = self.0.entry(category.clone()).or_default();
        match kind {
            PickKind::WillWin => picks.will_win = nominee,
            PickKind::ShouldWin => picks.should_win = nominee,
        }
        // A category with no pick left is the same as one never picked in.
        if *picks == CategoryPicks::default() {
            self.0.remove(&category);
        }
    }

    pub fn pick(&self, category: &CategoryKey, kind: PickKind) -> Option<&NomineeKey> {
        let picks = self.0.get(category)?;
        match kind {
            PickKind::WillWin => picks.will_win.as_ref(),
            PickKind::ShouldWin => picks.should_win.as_ref(),
        }
    }

    /// Whether the player predicted at least one winner, which puts them on the leaderboard
    /// (spec rule 1.4.4).
    pub fn has_a_will_win_pick(&self) -> bool {
        self.0.values().any(|picks| picks.will_win.is_some())
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
    fn rule_1_2_2_a_pick_names_a_nominee_of_its_own_category() {
        let season = season(&[("picture", &["a"]), ("director", &["b"])]);
        let picture = category_key("picture");

        assert_eq!(
            season.check_pick_write(before_lock(), &picture, Some(&nominee_key("a"))),
            Ok(())
        );
        assert_eq!(
            season.check_pick_write(before_lock(), &picture, Some(&nominee_key("b"))),
            Err(PickRefusal::NomineeNotInCategory)
        );
        assert_eq!(
            season.check_pick_write(before_lock(), &picture, Some(&nominee_key("nobody"))),
            Err(PickRefusal::NomineeNotInCategory)
        );
    }

    #[test]
    fn rule_1_2_3_the_two_picks_are_independent_and_may_name_the_same_nominee() {
        let picture = category_key("picture");
        let set = Some(nominee_key("a"));

        for (will_win, should_win) in [
            (None, None),
            (set.clone(), None),
            (None, set.clone()),
            (set.clone(), set),
        ] {
            let mut ballot = Ballot::default();
            ballot.write(picture.clone(), PickKind::WillWin, will_win.clone());
            ballot.write(picture.clone(), PickKind::ShouldWin, should_win.clone());

            assert_eq!(ballot.pick(&picture, PickKind::WillWin), will_win.as_ref());
            assert_eq!(ballot.pick(&picture, PickKind::ShouldWin), should_win.as_ref());
        }
    }

    #[test]
    fn rule_1_2_4_before_the_lock_a_pick_can_be_set_changed_and_cleared() {
        let season = season(&[("picture", &["a", "b"])]);
        let picture = category_key("picture");
        let mut ballot = Ballot::default();

        for nominee in [Some(nominee_key("a")), Some(nominee_key("b")), None] {
            assert_eq!(
                season.check_pick_write(before_lock(), &picture, nominee.as_ref()),
                Ok(())
            );
            ballot.write(picture.clone(), PickKind::WillWin, nominee.clone());
            assert_eq!(ballot.pick(&picture, PickKind::WillWin), nominee.as_ref());
        }
        assert_eq!(ballot, Ballot::default());
    }

    #[test]
    fn rule_1_2_5_a_write_is_refused_from_the_lock_instant_on() {
        let season = season(&[("picture", &["a"])]);
        let picture = category_key("picture");
        let after_lock = UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() + 1);

        assert_eq!(
            season.check_pick_write(before_lock(), &picture, Some(&nominee_key("a"))),
            Ok(())
        );
        assert_eq!(
            season.check_pick_write(LOCK, &picture, Some(&nominee_key("a"))),
            Err(PickRefusal::Locked)
        );
        assert_eq!(
            season.check_pick_write(after_lock, &picture, None),
            Err(PickRefusal::Locked)
        );
    }

    #[test]
    fn rule_4_2_18_an_unseeded_season_is_not_open_for_picks() {
        let season = season(&[("picture", &[])]);
        let picture = category_key("picture");

        assert_eq!(
            season.check_pick_write(before_lock(), &picture, None),
            Err(PickRefusal::NotOpen)
        );
        // Still "not open" once the lock instant has passed: without nominees the season is Upcoming.
        assert_eq!(season.check_pick_write(LOCK, &picture, None), Err(PickRefusal::NotOpen));
    }

    #[test]
    fn a_pick_in_an_unknown_category_is_refused() {
        let season = season(&[("picture", &["a"])]);

        assert_eq!(
            season.check_pick_write(before_lock(), &category_key("costume"), None),
            Err(PickRefusal::NoSuchCategory)
        );
    }

    #[test]
    fn every_pick_refusal_says_why() {
        let refusals = [
            (PickRefusal::NotOpen, "the season has no nominees yet"),
            (PickRefusal::Locked, "the ballot is locked"),
            (PickRefusal::NoSuchCategory, "the season has no such category"),
            (PickRefusal::NomineeNotInCategory, "the nominee is not in that category"),
        ];

        for (refusal, message) in refusals {
            assert_eq!(refusal.to_string(), message);
        }
    }

    /// The part of spec rule 1.2.1 that needs no storage; plan item P4.1 closes the rule in Postgres.
    #[test]
    fn a_second_write_of_one_kind_replaces_the_first() {
        let picture = category_key("picture");
        let mut ballot = Ballot::default();

        ballot.write(picture.clone(), PickKind::ShouldWin, Some(nominee_key("a")));
        ballot.write(picture.clone(), PickKind::ShouldWin, Some(nominee_key("b")));

        assert_eq!(ballot.pick(&picture, PickKind::ShouldWin), Some(&nominee_key("b")));
    }

    #[test]
    fn only_a_will_win_pick_counts_as_a_prediction() {
        let picture = category_key("picture");
        let mut ballot = Ballot::default();
        assert!(!ballot.has_a_will_win_pick());

        ballot.write(picture.clone(), PickKind::ShouldWin, Some(nominee_key("a")));
        assert!(!ballot.has_a_will_win_pick());

        ballot.write(picture, PickKind::WillWin, Some(nominee_key("a")));
        assert!(ballot.has_a_will_win_pick());
    }
}
