//! Seasons for tests, written as keys only: a label is its key in capitals.

use crate::season::{Category, CategoryKey, Nominee, NomineeKey, Season};
use crate::time::UtcInstant;

pub const LOCK: UtcInstant = UtcInstant::from_millis_since_epoch(1_000_000);

pub fn category_key(key: &str) -> CategoryKey {
    CategoryKey::new(key)
}

pub fn nominee_key(key: &str) -> NomineeKey {
    NomineeKey::new(key)
}

pub fn nominee(key: &str) -> Nominee {
    Nominee {
        key: nominee_key(key),
        label: key.to_uppercase(),
        secondary_label: None,
    }
}

pub fn category(key: &str, nominee_keys: &[&str]) -> Category {
    Category {
        key: category_key(key),
        name: key.to_uppercase(),
        nominees: nominee_keys.iter().map(|key| nominee(key)).collect(),
    }
}

/// The 99th season, locking at [`LOCK`], with the given categories.
pub fn season(categories: &[(&str, &[&str])]) -> Season {
    Season::new(
        99,
        LOCK,
        categories
            .iter()
            .map(|(key, nominees)| category(key, nominees))
            .collect(),
    )
    .expect("the fixture is a valid season")
}
