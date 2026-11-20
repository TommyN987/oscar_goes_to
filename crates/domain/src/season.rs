//! A season of the game: one ceremony, its lock instant, its films, and its categories and nominees in seed order.

use std::collections::BTreeSet;

use crate::key::key_type;
use crate::results::Results;
use crate::time::UtcInstant;

key_type! {
    /// The author-chosen key of a film, unique within its season (spec rule 4.1.2).
    FilmKey
}

key_type! {
    /// The author-chosen key of a category, unique within its season (spec rule 4.1.2).
    CategoryKey
}

key_type! {
    /// The author-chosen key of a nominee, unique within its category (spec rule 4.1.2).
    NomineeKey
}

/// A film with a nomination this season. Nominees name it by key, so its title is written once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Film {
    pub key: FilmKey,
    pub title: String,
}

/// One entry in one category: a nomination, which is always for one film (spec rule 1.1.3).
/// The same person nominated twice is two nominees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nominee {
    pub key: NomineeKey,
    pub film: FilmKey,
    /// Whoever is named with the nomination: one person, several, or a country.
    /// Left out where a guesser would not know the names.
    pub credited: Option<String>,
    /// The nominated work's own title when it is not the film itself: a song.
    pub title: Option<String>,
}

/// What a guesser picks a category's nominees by: the film, or who is credited.
/// Whatever shows a nominee leads with that part (spec rule 4.1.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lead {
    Film,
    Credited,
}

/// A category and its nominees, in seed order. Its position is its place in the season (spec rule 1.1.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub key: CategoryKey,
    pub name: String,
    pub lead: Lead,
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

/// Why a set of films and categories is not a season. Each message names the offending key.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SeasonError {
    #[error("the season has no categories")]
    NoCategories,
    #[error("two films have the key `{0}`")]
    DuplicateFilmKey(FilmKey),
    #[error("film `{0}` has no title")]
    EmptyFilmTitle(FilmKey),
    /// A film is listed for its nominations; one without any is most likely a mistyped key.
    #[error("no nominee names the film `{0}`")]
    UnusedFilm(FilmKey),
    #[error("two categories have the key `{0}`")]
    DuplicateCategoryKey(CategoryKey),
    #[error("category `{0}` has no name")]
    EmptyCategoryName(CategoryKey),
    #[error("category `{category}` has two nominees with the key `{nominee}`")]
    DuplicateNomineeKey { category: CategoryKey, nominee: NomineeKey },
    #[error("nominee `{nominee}` in category `{category}` names the film `{film}`, which is not listed")]
    UnknownFilm {
        category: CategoryKey,
        nominee: NomineeKey,
        film: FilmKey,
    },
    #[error("nominee `{nominee}` in category `{category}` needs a credit, because its category leads with one")]
    MissingCredit { category: CategoryKey, nominee: NomineeKey },
    #[error("nominee `{nominee}` in category `{category}` has an empty credit")]
    EmptyCredit { category: CategoryKey, nominee: NomineeKey },
    #[error("nominee `{nominee}` in category `{category}` has an empty title")]
    EmptyNomineeTitle { category: CategoryKey, nominee: NomineeKey },
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

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

/// Checks what one nominee must satisfy on its own: a listed film, a credit where its category leads
/// with one, and no blank credit or title.
fn check_nominee(category: &Category, nominee: &Nominee, film_keys: &BTreeSet<&FilmKey>) -> Result<(), SeasonError> {
    let keys = || (category.key.clone(), nominee.key.clone());
    if !film_keys.contains(&nominee.film) {
        let (category, nominee_key) = keys();
        return Err(SeasonError::UnknownFilm {
            category,
            nominee: nominee_key,
            film: nominee.film.clone(),
        });
    }
    match nominee.credited.as_deref() {
        None if category.lead == Lead::Credited => {
            let (category, nominee) = keys();
            return Err(SeasonError::MissingCredit { category, nominee });
        }
        Some(credited) if blank(credited) => {
            let (category, nominee) = keys();
            return Err(SeasonError::EmptyCredit { category, nominee });
        }
        _ => {}
    }
    if nominee.title.as_deref().is_some_and(blank) {
        let (category, nominee) = keys();
        return Err(SeasonError::EmptyNomineeTitle { category, nominee });
    }
    Ok(())
}

/// A season: a ceremony number, a lock instant, films, and ordered categories (spec rule 1.1.1).
///
/// Invariants, checked once in [`Season::new`] (spec rule 4.1.4):
/// - there is at least one category, and nominees are in every category or in none;
/// - film keys and category keys are unique, and nominee keys are unique within their category;
/// - no name, title or credit is blank;
/// - every nominee names a listed film, and every listed film is named by a nominee;
/// - where a category leads with who is credited, each of its nominees has a credit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Season {
    ceremony_number: u32,
    lock_instant: UtcInstant,
    films: Vec<Film>,
    categories: Vec<Category>,
}

impl Season {
    pub fn new(
        ceremony_number: u32,
        lock_instant: UtcInstant,
        films: Vec<Film>,
        categories: Vec<Category>,
    ) -> Result<Self, SeasonError> {
        if categories.is_empty() {
            return Err(SeasonError::NoCategories);
        }
        let mut film_keys = BTreeSet::new();
        for film in &films {
            if !film_keys.insert(&film.key) {
                return Err(SeasonError::DuplicateFilmKey(film.key.clone()));
            }
            if blank(&film.title) {
                return Err(SeasonError::EmptyFilmTitle(film.key.clone()));
            }
        }
        let mut films_named = BTreeSet::new();
        let mut category_keys = BTreeSet::new();
        for category in &categories {
            if !category_keys.insert(&category.key) {
                return Err(SeasonError::DuplicateCategoryKey(category.key.clone()));
            }
            if blank(&category.name) {
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
                check_nominee(category, nominee, &film_keys)?;
                films_named.insert(&nominee.film);
            }
        }
        if let Some(unused) = films.iter().find(|film| !films_named.contains(&film.key)) {
            return Err(SeasonError::UnusedFilm(unused.key.clone()));
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
            films,
            categories,
        })
    }

    pub fn ceremony_number(&self) -> u32 {
        self.ceremony_number
    }

    pub fn lock_instant(&self) -> UtcInstant {
        self.lock_instant
    }

    /// The films in seed order.
    pub fn films(&self) -> &[Film] {
        &self.films
    }

    pub fn film(&self, key: &FilmKey) -> Option<&Film> {
        self.films.iter().find(|film| &film.key == key)
    }

    /// The categories in seed order.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn category(&self, key: &CategoryKey) -> Option<&Category> {
        self.categories.iter().find(|category| &category.key == key)
    }

    /// The film `nominee` is nominated for.
    ///
    /// `nominee` must be one of this season's; by the invariant its film is then listed.
    /// A nominee of another season is a caller's bug, and panics.
    pub fn film_of(&self, nominee: &Nominee) -> &Film {
        self.film(&nominee.film)
            .expect("a nominee of this season names a listed film")
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
    use crate::fixtures::{LOCK, category, category_key, film, film_key, nominee_key, season, season_of};

    /// A nominee of `film`, with a credit and a title of its own as given.
    fn nomination(key: &str, film: &str, credited: Option<&str>, title: Option<&str>) -> Nominee {
        Nominee {
            key: nominee_key(key),
            film: film_key(film),
            credited: credited.map(str::to_owned),
            title: title.map(str::to_owned),
        }
    }

    fn category_led_by(lead: Lead, key: &str, nominees: Vec<Nominee>) -> Category {
        Category {
            key: category_key(key),
            name: key.to_uppercase(),
            lead,
            nominees,
        }
    }

    /// The part of spec rule 1.1.1 that needs no seed file; plan item P4.2 closes the rule with the 99th seed.
    #[test]
    fn a_season_keeps_its_ceremony_number_lock_instant_and_category_order() {
        let keys: Vec<String> = (1..=24).map(|position| format!("category-{position}")).collect();
        let categories = keys.iter().map(|key| category(key, &[])).collect();

        let season = Season::new(99, LOCK, vec![], categories).unwrap();

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
        let first = nomination("penn-film-one", "film-one", Some("Sean Penn"), None);
        let second = nomination("penn-film-two", "film-two", Some("Sean Penn"), None);
        let actor = category_led_by(Lead::Credited, "actor", vec![first.clone(), second.clone()]);

        let season = Season::new(99, LOCK, vec![film("film-one"), film("film-two")], vec![actor]).unwrap();

        let actor = season.category(&category_key("actor")).unwrap();
        assert_eq!(actor.nominee(&first.key), Some(&first));
        assert_eq!(actor.nominee(&second.key), Some(&second));

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
    fn rule_4_1_2_one_film_serves_nominees_in_several_categories() {
        let categories = vec![
            category_led_by(Lead::Film, "picture", vec![nomination("eeaao", "eeaao", None, None)]),
            category_led_by(Lead::Film, "editing", vec![nomination("eeaao", "eeaao", None, None)]),
        ];

        let season = Season::new(99, LOCK, vec![film("eeaao")], categories).unwrap();

        assert_eq!(season.films(), [film("eeaao")]);
        assert_eq!(season.film(&film_key("eeaao")), Some(&film("eeaao")));
    }

    #[test]
    fn rule_4_1_4_a_season_without_categories_is_refused() {
        assert_eq!(season_of(vec![]), Err(SeasonError::NoCategories));
    }

    #[test]
    fn rule_4_1_4_a_repeated_film_key_is_refused() {
        let categories = vec![category("picture", &["a"])];

        assert_eq!(
            Season::new(99, LOCK, vec![film("a"), film("a")], categories),
            Err(SeasonError::DuplicateFilmKey(film_key("a")))
        );
    }

    #[test]
    fn rule_4_1_4_a_blank_film_title_is_refused() {
        let mut untitled = film("a");
        untitled.title = " ".into();

        assert_eq!(
            Season::new(99, LOCK, vec![untitled], vec![category("picture", &["a"])]),
            Err(SeasonError::EmptyFilmTitle(film_key("a")))
        );
    }

    #[test]
    fn rule_4_1_4_a_nominee_names_a_listed_film() {
        let categories = vec![category("picture", &["a"])];

        assert_eq!(
            Season::new(99, LOCK, vec![], categories),
            Err(SeasonError::UnknownFilm {
                category: category_key("picture"),
                nominee: nominee_key("a"),
                film: film_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_4_a_film_that_no_nominee_names_is_refused() {
        let categories = vec![category("picture", &["a"])];

        assert_eq!(
            Season::new(99, LOCK, vec![film("a"), film("mistyped")], categories),
            Err(SeasonError::UnusedFilm(film_key("mistyped")))
        );
    }

    #[test]
    fn rule_4_1_4_a_repeated_category_key_is_refused() {
        let categories = vec![category("picture", &[]), category("picture", &[])];

        assert_eq!(
            season_of(categories),
            Err(SeasonError::DuplicateCategoryKey(category_key("picture")))
        );
    }

    #[test]
    fn rule_4_1_4_a_repeated_nominee_key_in_one_category_is_refused() {
        let categories = vec![category("picture", &["a", "a"])];

        assert_eq!(
            season_of(categories),
            Err(SeasonError::DuplicateNomineeKey {
                category: category_key("picture"),
                nominee: nominee_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_2_two_categories_may_use_the_same_nominee_key() {
        let categories = vec![category("picture", &["a"]), category("director", &["a"])];

        assert!(season_of(categories).is_ok());
    }

    #[test]
    fn rule_4_1_4_a_blank_category_name_is_refused() {
        let mut picture = category("picture", &[]);
        picture.name = "  ".into();

        assert_eq!(
            season_of(vec![picture]),
            Err(SeasonError::EmptyCategoryName(category_key("picture")))
        );
    }

    #[test]
    fn rule_4_1_4_a_category_that_leads_with_the_credit_needs_one_on_every_nominee() {
        let uncredited = nomination("a", "a", None, None);
        let actor = category_led_by(Lead::Credited, "actor", vec![uncredited]);

        assert_eq!(
            season_of(vec![actor]),
            Err(SeasonError::MissingCredit {
                category: category_key("actor"),
                nominee: nominee_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_4_a_blank_credit_is_refused() {
        let sound = category_led_by(Lead::Film, "sound", vec![nomination("a", "a", Some(" "), None)]);

        assert_eq!(
            season_of(vec![sound]),
            Err(SeasonError::EmptyCredit {
                category: category_key("sound"),
                nominee: nominee_key("a"),
            })
        );
    }

    #[test]
    fn rule_4_1_4_a_blank_title_of_a_nominee_is_refused() {
        let song = category_led_by(Lead::Film, "song", vec![nomination("a", "a", None, Some(""))]);

        assert_eq!(
            season_of(vec![song]),
            Err(SeasonError::EmptyNomineeTitle {
                category: category_key("song"),
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
            season_of(categories),
            Err(SeasonError::NomineesInSomeCategoriesOnly {
                without_nominees: vec![category_key("director"), category_key("actor")],
            })
        );
    }

    #[test]
    fn rule_4_1_7_a_season_may_have_no_nominees_at_all() {
        let season = season(&[("picture", &[]), ("director", &[])]);

        assert!(!season.has_nominees());
        assert!(season.films().is_empty());
        assert_eq!(season.phase(LOCK, &Results::default()), Phase::Upcoming);
    }

    #[test]
    fn rule_4_1_4_a_season_without_nominees_lists_no_film() {
        let categories = vec![category("picture", &[]), category("director", &[])];

        assert_eq!(
            Season::new(99, LOCK, vec![film("early")], categories),
            Err(SeasonError::UnusedFilm(film_key("early")))
        );
    }

    #[test]
    fn a_seeded_season_has_nominees() {
        assert!(season(&[("picture", &["a"])]).has_nominees());
    }

    #[test]
    fn a_nominee_resolves_to_its_film() {
        let season = season(&[("picture", &["a", "b"]), ("editing", &["b"])]);
        let editing = season.category(&category_key("editing")).unwrap();

        assert_eq!(season.film_of(&editing.nominees[0]), &film("b"));
    }

    #[test]
    fn rule_4_1_2_each_kind_of_key_refuses_what_is_not_a_key() {
        assert!(FilmKey::parse("Not A Key").is_err());
        assert!(CategoryKey::parse("Not A Key").is_err());
        assert!(NomineeKey::parse("Not A Key").is_err());
        assert_eq!(CategoryKey::parse("best-picture").unwrap().as_str(), "best-picture");
        assert_eq!(FilmKey::parse("eeaao").unwrap().to_string(), "eeaao");
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
                SeasonError::DuplicateFilmKey(film_key("eeaao")),
                "two films have the key `eeaao`",
            ),
            (
                SeasonError::EmptyFilmTitle(film_key("eeaao")),
                "film `eeaao` has no title",
            ),
            (
                SeasonError::UnusedFilm(film_key("eeaao")),
                "no nominee names the film `eeaao`",
            ),
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
                SeasonError::UnknownFilm {
                    category: picture(),
                    nominee: a(),
                    film: film_key("eeaao"),
                },
                "nominee `a` in category `picture` names the film `eeaao`, which is not listed",
            ),
            (
                SeasonError::MissingCredit {
                    category: picture(),
                    nominee: a(),
                },
                "nominee `a` in category `picture` needs a credit, because its category leads with one",
            ),
            (
                SeasonError::EmptyCredit {
                    category: picture(),
                    nominee: a(),
                },
                "nominee `a` in category `picture` has an empty credit",
            ),
            (
                SeasonError::EmptyNomineeTitle {
                    category: picture(),
                    nominee: a(),
                },
                "nominee `a` in category `picture` has an empty title",
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
