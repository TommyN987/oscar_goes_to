//! A season of the game: one ceremony, its lock instant, and its categories and nominees in seed order.

use std::collections::BTreeSet;
use std::fmt;

use crate::results::Results;
use crate::time::UtcInstant;

/// The author-chosen key of a category, unique within its season (spec rule 4.1.2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CategoryKey(String);

impl CategoryKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CategoryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The author-chosen key of a nominee, unique within its category (spec rule 4.1.2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NomineeKey(String);

impl NomineeKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NomineeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One entry in one category. The same person nominated twice is two nominees (spec rule 1.1.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nominee {
    pub key: NomineeKey,
    pub label: String,
    pub secondary_label: Option<String>,
}

/// A category and its nominees, in seed order. Its position is its place in the season (spec rule 1.1.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub key: CategoryKey,
    pub name: String,
    pub nominees: Vec<Nominee>,
}

impl Category {
    pub fn nominee(&self, key: &NomineeKey) -> Option<&Nominee> {
        self.nominees.iter().find(|nominee| &nominee.key == key)
    }
}

/// Where a season stands. It is derived from the season, the clock and the results, and never stored
/// (spec rule 1.1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The nominees are not seeded yet.
    Upcoming,
    /// Players may pick.
    Open,
    /// The lock instant has passed and at least one category awaits its result.
    Locked,
    /// Every category has a result.
    Complete,
}

/// Why a set of categories is not a season. Each message names the offending key.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SeasonError {
    #[error("the season has no categories")]
    NoCategories,
    #[error("two categories have the key `{0}`")]
    DuplicateCategoryKey(CategoryKey),
    #[error("category `{0}` has no name")]
    EmptyCategoryName(CategoryKey),
    #[error("category `{category}` has two nominees with the key `{nominee}`")]
    DuplicateNomineeKey { category: CategoryKey, nominee: NomineeKey },
    /// The label is blank, or a secondary label is given and blank.
    #[error("nominee `{nominee}` in category `{category}` has an empty label")]
    EmptyNomineeLabel { category: CategoryKey, nominee: NomineeKey },
    /// Some categories have nominees and these have none; a season is seeded as a batch.
    #[error(
        "nominees must be in every category or in none; these have none: {}",
        comma_separated(.without_nominees)
    )]
    NomineesInSomeCategoriesOnly { without_nominees: Vec<CategoryKey> },
}

/// The keys as an error message lists them.
fn comma_separated(keys: &[CategoryKey]) -> String {
    let keys: Vec<&str> = keys.iter().map(CategoryKey::as_str).collect();
    keys.join(", ")
}

/// A season: a ceremony number, a lock instant, and ordered categories (spec rule 1.1.1).
///
/// Invariants, checked once in [`Season::new`]:
/// there is at least one category; category keys are unique; nominee keys are unique within their category;
/// no name or label is empty; and nominees are in every category or in none (spec rule 4.1.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Season {
    ceremony_number: u32,
    lock_instant: UtcInstant,
    categories: Vec<Category>,
}

impl Season {
    pub fn new(ceremony_number: u32, lock_instant: UtcInstant, categories: Vec<Category>) -> Result<Self, SeasonError> {
        if categories.is_empty() {
            return Err(SeasonError::NoCategories);
        }
        let mut category_keys = BTreeSet::new();
        for category in &categories {
            if !category_keys.insert(&category.key) {
                return Err(SeasonError::DuplicateCategoryKey(category.key.clone()));
            }
            if category.name.trim().is_empty() {
                return Err(SeasonError::EmptyCategoryName(category.key.clone()));
            }
            let mut nominee_keys = BTreeSet::new();
            for nominee in &category.nominees {
                if !nominee_keys.insert(&nominee.key) {
                    return Err(SeasonError::DuplicateNomineeKey {
                        category: category.key.clone(),
                        nominee: nominee.key.clone(),
                    });
                }
                let blank = |label: &str| label.trim().is_empty();
                if blank(&nominee.label) || nominee.secondary_label.as_deref().is_some_and(blank) {
                    return Err(SeasonError::EmptyNomineeLabel {
                        category: category.key.clone(),
                        nominee: nominee.key.clone(),
                    });
                }
            }
        }
        let without_nominees: Vec<CategoryKey> = categories
            .iter()
            .filter(|category| category.nominees.is_empty())
            .map(|category| category.key.clone())
            .collect();
        if !without_nominees.is_empty() && without_nominees.len() < categories.len() {
            return Err(SeasonError::NomineesInSomeCategoriesOnly { without_nominees });
        }
        Ok(Self {
            ceremony_number,
            lock_instant,
            categories,
        })
    }

    pub fn ceremony_number(&self) -> u32 {
        self.ceremony_number
    }

    pub fn lock_instant(&self) -> UtcInstant {
        self.lock_instant
    }

    /// The categories in seed order.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn category(&self, key: &CategoryKey) -> Option<&Category> {
        self.categories.iter().find(|category| &category.key == key)
    }

    /// The phase at `now`, given the results recorded so far (spec rule 1.1.5).
    ///
    /// Total: every input has exactly one phase. Before the lock instant a seeded season is Open
    /// whatever the results say, so moving the lock later reopens it (spec rule 1.1.6).
    pub fn phase(&self, now: UtcInstant, results: &Results) -> Phase {
        if !self.has_nominees() {
            Phase::Upcoming
        } else if now < self.lock_instant {
            Phase::Open
        } else if self
            .categories
            .iter()
            .all(|category| results.winners(&category.key).is_some())
        {
            Phase::Complete
        } else {
            Phase::Locked
        }
    }

    /// Whether the nominees have been seeded. By the invariant, one category answers for all.
    pub fn has_nominees(&self) -> bool {
        !self.categories[0].nominees.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ballot::{Ballot, PickKind};
    use crate::fixtures::{LOCK, category, category_key, nominee, nominee_key, season};

    /// The part of spec rule 1.1.1 that needs no seed file; plan item P4.2 closes the rule with the 99th seed.
    #[test]
    fn a_season_keeps_its_ceremony_number_lock_instant_and_category_order() {
        let keys: Vec<String> = (1..=24).map(|position| format!("category-{position}")).collect();
        let categories = keys.iter().map(|key| category(key, &[])).collect();

        let season = Season::new(99, LOCK, categories).unwrap();

        assert_eq!(season.ceremony_number(), 99);
        assert_eq!(season.lock_instant(), LOCK);
        let seeded: Vec<&str> = season
            .categories()
            .iter()
            .map(|category| category.key.as_str())
            .collect();
        assert_eq!(seeded, keys);
    }

    #[test]
    fn rule_1_1_2_categories_with_five_and_with_ten_nominees_both_load() {
        let ten = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

        let season = season(&[("director", &ten[..5]), ("picture", &ten)]);

        assert_eq!(season.category(&category_key("director")).unwrap().nominees.len(), 5);
        assert_eq!(season.category(&category_key("picture")).unwrap().nominees.len(), 10);
    }

    #[test]
    fn rule_1_1_3_one_person_twice_in_a_category_is_two_nominees() {
        let mut first = nominee("penn-film-one");
        let mut second = nominee("penn-film-two");
        first.label = "Sean Penn".into();
        second.label = "Sean Penn".into();
        let mut actor = category("actor", &[]);
        actor.nominees = vec![first.clone(), second.clone()];

        let season = Season::new(99, LOCK, vec![actor]).unwrap();

        let actor = season.category(&category_key("actor")).unwrap();
        assert_eq!(actor.nominee(&nominee_key("penn-film-one")), Some(&first));
        assert_eq!(actor.nominee(&nominee_key("penn-film-two")), Some(&second));

        // Each entry can be picked on its own: one to win, the other as the favorite.
        let before_lock = UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() - 1);
        let mut ballot = Ballot::default();
        for (kind, entry) in [(PickKind::WillWin, &first), (PickKind::ShouldWin, &second)] {
            assert_eq!(
                season.check_pick_write(before_lock, &actor.key, Some(&entry.key)),
                Ok(())
            );
            ballot.write(actor.key.clone(), kind, Some(entry.key.clone()));
        }
        assert_eq!(ballot.pick(&actor.key, PickKind::WillWin), Some(&first.key));
        assert_eq!(ballot.pick(&actor.key, PickKind::ShouldWin), Some(&second.key));
    }

    #[test]
    fn rule_4_1_4_a_season_without_categories_is_refused() {
        assert_eq!(Season::new(99, LOCK, vec![]), Err(SeasonError::NoCategories));
    }

    #[test]
    fn rule_4_1_4_a_repeated_category_key_is_refused() {
        let categories = vec![category("picture", &[]), category("picture", &[])];

        assert_eq!(
            Season::new(99, LOCK, categories),
            Err(SeasonError::DuplicateCategoryKey(category_key("picture")))
        );
    }

    #[test]
    fn rule_4_1_4_a_repeated_nominee_key_in_one_category_is_refused() {
        let categories = vec![category("picture", &["a", "a"])];

        assert_eq!(
            Season::new(99, LOCK, categories),
            Err(SeasonError::DuplicateNomineeKey {
                category: category_key("picture"),
                nominee: nominee_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_2_two_categories_may_use_the_same_nominee_key() {
        let categories = vec![category("picture", &["a"]), category("director", &["a"])];

        assert!(Season::new(99, LOCK, categories).is_ok());
    }

    #[test]
    fn rule_4_1_4_a_blank_category_name_is_refused() {
        let mut picture = category("picture", &[]);
        picture.name = "  ".into();

        assert_eq!(
            Season::new(99, LOCK, vec![picture]),
            Err(SeasonError::EmptyCategoryName(category_key("picture")))
        );
    }

    #[test]
    fn rule_4_1_4_a_blank_nominee_label_is_refused() {
        let mut picture = category("picture", &["a"]);
        picture.nominees[0].label = String::new();

        assert_eq!(
            Season::new(99, LOCK, vec![picture]),
            Err(SeasonError::EmptyNomineeLabel {
                category: category_key("picture"),
                nominee: nominee_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_4_a_blank_secondary_label_is_refused() {
        let mut picture = category("picture", &["a"]);
        picture.nominees[0].secondary_label = Some(" ".into());

        assert_eq!(
            Season::new(99, LOCK, vec![picture]),
            Err(SeasonError::EmptyNomineeLabel {
                category: category_key("picture"),
                nominee: nominee_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_4_nominees_in_only_some_categories_are_refused() {
        let categories = vec![
            category("picture", &["a"]),
            category("director", &[]),
            category("actor", &[]),
        ];

        assert_eq!(
            Season::new(99, LOCK, categories),
            Err(SeasonError::NomineesInSomeCategoriesOnly {
                without_nominees: vec![category_key("director"), category_key("actor")],
            })
        );
    }

    #[test]
    fn rule_4_1_7_a_season_may_have_no_nominees_at_all() {
        let season = season(&[("picture", &[]), ("director", &[])]);

        assert!(!season.has_nominees());
        assert_eq!(season.phase(LOCK, &Results::default()), Phase::Upcoming);
    }

    #[test]
    fn a_seeded_season_has_nominees() {
        assert!(season(&[("picture", &["a"])]).has_nominees());
    }

    /// Results for the first `count` categories of `season`, each won by its first nominee.
    fn results_for_first(season: &Season, count: usize) -> Results {
        let mut results = Results::default();
        for category in &season.categories()[..count] {
            let winners = season
                .winner_set(LOCK, &category.key, [category.nominees[0].key.clone()])
                .unwrap();
            results.record(category.key.clone(), winners);
        }
        results
    }

    #[test]
    fn rule_1_1_5_the_phase_is_fixed_for_every_combination_of_inputs() {
        use Phase::{Complete, Locked, Open, Upcoming};
        let before = UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() - 1);
        let after = UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() + 1);
        let unseeded = season(&[("picture", &[]), ("director", &[])]);
        let seeded = season(&[("picture", &["a"]), ("director", &["b"])]);
        // Results for no category, for one of the two, and for both.
        let results = [
            Results::default(),
            results_for_first(&seeded, 1),
            results_for_first(&seeded, 2),
        ];

        // Rows: before, at and after the lock instant. Columns: no, some and all results.
        let unseeded_phases = [[Upcoming; 3]; 3];
        let seeded_phases = [
            [Open, Open, Open],
            [Locked, Locked, Complete],
            [Locked, Locked, Complete],
        ];

        for (season, phases) in [(&unseeded, unseeded_phases), (&seeded, seeded_phases)] {
            for (now, row) in [before, LOCK, after].into_iter().zip(phases) {
                for (results, phase) in results.iter().zip(row) {
                    assert_eq!(season.phase(now, results), phase, "at {now:?} with {results:?}");
                }
            }
        }
    }

    #[test]
    fn rule_4_1_4_every_refusal_names_what_is_wrong() {
        let picture = || category_key("picture");
        let a = || nominee_key("a");
        let refusals = [
            (SeasonError::NoCategories, "the season has no categories"),
            (
                SeasonError::DuplicateCategoryKey(picture()),
                "two categories have the key `picture`",
            ),
            (
                SeasonError::EmptyCategoryName(picture()),
                "category `picture` has no name",
            ),
            (
                SeasonError::DuplicateNomineeKey {
                    category: picture(),
                    nominee: a(),
                },
                "category `picture` has two nominees with the key `a`",
            ),
            (
                SeasonError::EmptyNomineeLabel {
                    category: picture(),
                    nominee: a(),
                },
                "nominee `a` in category `picture` has an empty label",
            ),
            (
                SeasonError::NomineesInSomeCategoriesOnly {
                    without_nominees: vec![category_key("director"), category_key("actor")],
                },
                "nominees must be in every category or in none; these have none: director, actor",
            ),
        ];

        for (refusal, message) in refusals {
            assert_eq!(refusal.to_string(), message);
        }
    }
}
