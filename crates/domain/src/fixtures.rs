//! Seasons for tests, written as keys only.
//!
//! A name or a title is its key in capitals, and each nominee is for a film that has the nominee's own key.

use crate::season::{Category, CategoryKey, Film, FilmKey, Lead, Nominee, NomineeKey, Season, SeasonError};
use crate::time::UtcInstant;

pub const LOCK: UtcInstant = UtcInstant::from_millis_since_epoch(1_000_000);

pub fn film_key(key: &str) -> FilmKey {
    FilmKey::parse(key).expect("the fixture uses valid keys")
}

pub fn category_key(key: &str) -> CategoryKey {
    CategoryKey::parse(key).expect("the fixture uses valid keys")
}

pub fn nominee_key(key: &str) -> NomineeKey {
    NomineeKey::parse(key).expect("the fixture uses valid keys")
}

pub fn film(key: &str) -> Film {
    Film {
        key: film_key(key),
        title: key.to_uppercase(),
    }
}

/// A nominee for the film with the same key, with no credit and no title of its own.
pub fn nominee(key: &str) -> Nominee {
    Nominee {
        key: nominee_key(key),
        film: film_key(key),
        credited: None,
        title: None,
    }
}

/// A category that leads with the film.
pub fn category(key: &str, nominee_keys: &[&str]) -> Category {
    Category {
        key: category_key(key),
        name: key.to_uppercase(),
        lead: Lead::Film,
        nominees: nominee_keys.iter().map(|key| nominee(key)).collect(),
    }
}

/// The films the nominees of `categories` name, each once, in order of first mention.
pub fn films_of(categories: &[Category]) -> Vec<Film> {
    let mut films: Vec<Film> = Vec::new();
    for nominee in categories.iter().flat_map(|category| &category.nominees) {
        if films.iter().all(|film| film.key != nominee.film) {
            films.push(film(nominee.film.as_str()));
        }
    }
    films
}

/// The 99th season, locking at [`LOCK`], with `categories` and exactly the films their nominees name.
pub fn season_of(categories: Vec<Category>) -> Result<Season, SeasonError> {
    Season::new(99, LOCK, films_of(&categories), categories)
}

/// The 99th season, locking at [`LOCK`], with the given categories and nominees.
pub fn season(categories: &[(&str, &[&str])]) -> Season {
    season_of(
        categories
            .iter()
            .map(|(key, nominees)| category(key, nominees))
            .collect(),
    )
    .expect("the fixture is a valid season")
}
