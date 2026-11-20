//! What a re-import would change: the difference between the stored season and the one in the seed file.
//!
//! Entries are matched by key, never by what they read, so a corrected title or credit keeps its picks
//! (spec rule 4.1.2).

use std::collections::BTreeSet;

use crate::season::{CategoryKey, FilmKey, NomineeKey, Season};
use crate::time::UtcInstant;

/// A nominee as the diff names it: its category's key and its own.
pub type NomineeRef = (CategoryKey, NomineeKey);

/// How one kind of entry differs between the stored season and the incoming one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changes<Key> {
    /// Keys only the incoming season has, in its order.
    pub added: Vec<Key>,
    /// Keys both have, whose entry differs in anything but its place, in the incoming order.
    pub changed: Vec<Key>,
    /// Keys only the stored season has, in its order.
    pub removed: Vec<Key>,
    /// Whether the entries both seasons have appear in a different order.
    pub reordered: bool,
}

impl<Key> Default for Changes<Key> {
    fn default() -> Self {
        Self {
            added: Vec::new(),
            changed: Vec::new(),
            removed: Vec::new(),
            reordered: false,
        }
    }
}

impl<Key> Changes<Key> {
    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty() && !self.reordered
    }
}

/// Everything an import of `incoming` over `stored` would change (spec rule 4.1.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonDiff {
    /// The stored lock instant and the incoming one, if they differ (spec rule 1.1.6).
    pub lock_instant: Option<(UtcInstant, UtcInstant)>,
    /// A film is changed when its title differs. The order of films carries no meaning and is never reported.
    pub films: Changes<FilmKey>,
    /// A category is changed when its name or its lead differs.
    pub categories: Changes<CategoryKey>,
    /// A nominee is changed when its film, its credit or its own title differs.
    /// The nominees of an added category are all added, and those of a removed category all removed.
    pub nominees: Changes<NomineeRef>,
}

impl SeasonDiff {
    /// Whether the import would change nothing.
    pub fn is_empty(&self) -> bool {
        self.lock_instant.is_none() && self.films.is_empty() && self.categories.is_empty() && self.nominees.is_empty()
    }

    /// The removed nominees among `in_use`, the nominees that picks or results name.
    /// Removing one deletes those picks or results, so an import refuses it unless forced
    /// (spec rules 1.1.8 and 4.1.6).
    pub fn removals_in_use(&self, in_use: &BTreeSet<NomineeRef>) -> Vec<NomineeRef> {
        self.nominees
            .removed
            .iter()
            .filter(|removed| in_use.contains(removed))
            .cloned()
            .collect()
    }
}

/// Compares the stored season with the incoming one. Neither is changed.
pub fn diff(stored: &Season, incoming: &Season) -> SeasonDiff {
    let categories = changes(
        stored.categories().iter().map(|category| (&category.key, category)),
        incoming.categories().iter().map(|category| (&category.key, category)),
        |stored, incoming| stored.name != incoming.name || stored.lead != incoming.lead,
    );
    // Films are a list to look titles up in; their order shows nowhere, so a new order is no change.
    let films = Changes {
        reordered: false,
        ..changes(
            stored.films().iter().map(|film| (&film.key, film)),
            incoming.films().iter().map(|film| (&film.key, film)),
            |stored, incoming| stored.title != incoming.title,
        )
    };

    let mut nominees = Changes::default();
    for category in stored.categories() {
        // A removed category has no incoming nominees, so all of its own count as removed.
        let incoming_nominees = incoming
            .category(&category.key)
            .map_or(&[][..], |incoming| &incoming.nominees);
        let within = changes(
            category.nominees.iter().map(|nominee| (&nominee.key, nominee)),
            incoming_nominees.iter().map(|nominee| (&nominee.key, nominee)),
            |stored, incoming| {
                stored.film != incoming.film || stored.credited != incoming.credited || stored.title != incoming.title
            },
        );
        let in_category = |nominee: NomineeKey| (category.key.clone(), nominee);
        nominees.changed.extend(within.changed.into_iter().map(in_category));
        nominees.removed.extend(within.removed.into_iter().map(in_category));
        nominees.reordered |= within.reordered;
    }
    for category in incoming.categories() {
        let stored_nominee = |key: &NomineeKey| stored.category(&category.key)?.nominee(key);
        let added = category
            .nominees
            .iter()
            .filter(|nominee| stored_nominee(&nominee.key).is_none());
        nominees
            .added
            .extend(added.map(|nominee| (category.key.clone(), nominee.key.clone())));
    }

    SeasonDiff {
        lock_instant: (stored.lock_instant() != incoming.lock_instant())
            .then(|| (stored.lock_instant(), incoming.lock_instant())),
        films,
        categories,
        nominees,
    }
}

/// Compares two sequences of keyed entries. `changed` says whether an entry both hold differs.
fn changes<'a, Key: Clone + Ord + 'a, Entry: 'a>(
    stored: impl Iterator<Item = (&'a Key, &'a Entry)>,
    incoming: impl Iterator<Item = (&'a Key, &'a Entry)>,
    changed: impl Fn(&Entry, &Entry) -> bool,
) -> Changes<Key> {
    let stored: Vec<(&Key, &Entry)> = stored.collect();
    let incoming: Vec<(&Key, &Entry)> = incoming.collect();
    let entry_in = |entries: &[(&'a Key, &'a Entry)], key: &Key| {
        entries.iter().find(|(other, _)| *other == key).map(|(_, entry)| *entry)
    };

    let mut changes = Changes::default();
    let mut kept_in_incoming_order = Vec::new();
    for (key, entry) in &incoming {
        match entry_in(&stored, key) {
            None => changes.added.push((*key).clone()),
            Some(stored_entry) => {
                kept_in_incoming_order.push(*key);
                if changed(stored_entry, entry) {
                    changes.changed.push((*key).clone());
                }
            }
        }
    }
    let mut kept_in_stored_order = Vec::new();
    for (key, _) in &stored {
        match entry_in(&incoming, key) {
            None => changes.removed.push((*key).clone()),
            Some(_) => kept_in_stored_order.push(*key),
        }
    }
    changes.reordered = kept_in_stored_order != kept_in_incoming_order;
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{LOCK, category, category_key, film, film_key, nominee, nominee_key, season, season_of};
    use crate::season::Lead;

    fn nominee_ref(category: &str, nominee: &str) -> NomineeRef {
        (category_key(category), nominee_key(nominee))
    }

    #[test]
    fn rule_4_1_5_an_identical_season_has_no_difference() {
        let stored = season(&[("picture", &["a", "b"]), ("director", &["c"])]);

        assert!(diff(&stored, &stored.clone()).is_empty());
    }

    #[test]
    fn rule_4_1_5_added_and_removed_entries_are_listed() {
        let stored = season(&[("picture", &["a", "b"]), ("director", &["c"])]);
        let incoming = season(&[("picture", &["b", "x"]), ("actor", &["y", "z"])]);

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.films.added, [film_key("x"), film_key("y"), film_key("z")]);
        assert_eq!(difference.films.removed, [film_key("a"), film_key("c")]);
        assert_eq!(difference.categories.added, [category_key("actor")]);
        assert_eq!(difference.categories.removed, [category_key("director")]);
        assert_eq!(
            difference.nominees.added,
            [
                nominee_ref("picture", "x"),
                nominee_ref("actor", "y"),
                nominee_ref("actor", "z")
            ]
        );
        assert_eq!(
            difference.nominees.removed,
            [nominee_ref("picture", "a"), nominee_ref("director", "c")]
        );
        assert!(difference.films.changed.is_empty());
        assert!(difference.categories.changed.is_empty());
        assert!(difference.nominees.changed.is_empty());
        assert!(!difference.is_empty());
    }

    #[test]
    fn rule_4_1_2_a_corrected_title_name_or_credit_keeps_its_key() {
        let stored = season(&[("picture", &["a", "b", "c"])]);
        let mut corrected_film = film("a");
        corrected_film.title = "A, corrected".into();
        let mut picture = category("picture", &["a", "b", "c"]);
        picture.name = "Best Picture".into();
        picture.nominees[1].credited = Some("Producer B".into());
        picture.nominees[2].title = Some("C's own title".into());
        let incoming = Season::new(99, LOCK, vec![corrected_film, film("b"), film("c")], vec![picture]).unwrap();

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.films.changed, [film_key("a")]);
        assert_eq!(difference.categories.changed, [category_key("picture")]);
        assert_eq!(
            difference.nominees.changed,
            [nominee_ref("picture", "b"), nominee_ref("picture", "c")]
        );
        for added_or_removed in [&difference.nominees.added, &difference.nominees.removed] {
            assert!(added_or_removed.is_empty());
        }
        assert!(difference.films.added.is_empty() && difference.films.removed.is_empty());
        assert!(difference.categories.added.is_empty() && difference.categories.removed.is_empty());
    }

    #[test]
    fn rule_4_1_2_a_film_with_a_new_key_is_removed_and_added_and_its_nominees_change() {
        let stored = season(&[("picture", &["a"])]);
        let mut picture = category("picture", &["a"]);
        picture.nominees[0].film = film_key("a-rekeyed");
        let incoming = season_of(vec![picture]).unwrap();

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.films.removed, [film_key("a")]);
        assert_eq!(difference.films.added, [film_key("a-rekeyed")]);
        assert_eq!(difference.nominees.changed, [nominee_ref("picture", "a")]);
        assert!(difference.nominees.removed.is_empty() && difference.nominees.added.is_empty());
    }

    #[test]
    fn a_nominee_moved_to_another_film_is_changed() {
        let stored = season_of(vec![category("picture", &["a"]), category("editing", &["b"])]).unwrap();
        let mut editing = category("editing", &["b"]);
        editing.nominees[0].film = film_key("a");
        let incoming = Season::new(99, LOCK, vec![film("a")], vec![category("picture", &["a"]), editing]).unwrap();

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.nominees.changed, [nominee_ref("editing", "b")]);
        assert_eq!(difference.films.removed, [film_key("b")]);
    }

    #[test]
    fn a_category_that_leads_with_something_else_is_changed() {
        let stored = season(&[("actor", &["a"])]);
        let mut actor = category("actor", &["a"]);
        actor.lead = Lead::Credited;
        actor.nominees[0].credited = Some("An Actor".into());
        let incoming = season_of(vec![actor]).unwrap();

        assert_eq!(diff(&stored, &incoming).categories.changed, [category_key("actor")]);
    }

    #[test]
    fn rule_4_1_2_a_changed_key_is_a_removal_plus_an_addition() {
        let stored = season(&[("picture", &["a"])]);
        let mut renamed = nominee("a");
        renamed.key = nominee_key("a-renamed");
        let mut picture = category("picture", &[]);
        picture.nominees = vec![renamed];
        let incoming = season_of(vec![picture]).unwrap();

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.nominees.removed, [nominee_ref("picture", "a")]);
        assert_eq!(difference.nominees.added, [nominee_ref("picture", "a-renamed")]);
        assert!(difference.films.is_empty());
    }

    /// The part of spec rule 1.1.6 that needs no storage; plan item P4.3 closes the rule.
    #[test]
    fn a_new_lock_instant_is_reported_with_the_old_one() {
        let stored = season(&[("picture", &["a"])]);
        let later = UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() + 3_600_000);
        let incoming = Season::new(99, later, stored.films().to_vec(), stored.categories().to_vec()).unwrap();

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.lock_instant, Some((LOCK, later)));
        assert!(!difference.is_empty());
    }

    #[test]
    fn a_new_order_is_reported_for_categories_and_for_nominees() {
        let stored = season(&[("picture", &["a", "b"]), ("director", &["c", "d"])]);
        let categories_swapped = season(&[("director", &["c", "d"]), ("picture", &["a", "b"])]);
        let nominees_swapped = season(&[("picture", &["a", "b"]), ("director", &["d", "c"])]);

        assert!(diff(&stored, &categories_swapped).categories.reordered);
        assert!(!diff(&stored, &categories_swapped).nominees.reordered);
        assert!(diff(&stored, &nominees_swapped).nominees.reordered);
        assert!(!diff(&stored, &nominees_swapped).categories.reordered);
        assert!(!diff(&stored, &nominees_swapped).is_empty());
    }

    #[test]
    fn the_order_of_films_carries_no_meaning() {
        let stored = season(&[("picture", &["a", "b"])]);
        let films_swapped = Season::new(99, LOCK, vec![film("b"), film("a")], stored.categories().to_vec()).unwrap();

        let difference = diff(&stored, &films_swapped);

        assert!(!difference.films.reordered);
        assert!(difference.is_empty());
    }

    #[test]
    fn an_addition_between_two_entries_is_not_a_new_order() {
        let stored = season(&[("picture", &["a", "b"]), ("director", &["c"])]);
        let incoming = season(&[("picture", &["a", "x", "b"]), ("actor", &["y"]), ("director", &["c"])]);

        let difference = diff(&stored, &incoming);

        assert!(!difference.films.reordered);
        assert!(!difference.categories.reordered);
        assert!(!difference.nominees.reordered);
    }

    #[test]
    fn rule_4_1_6_only_a_removal_that_picks_or_results_name_needs_force() {
        let stored = season(&[("picture", &["a", "b", "c"]), ("director", &["d"])]);
        let incoming = season(&[("picture", &["c"])]);
        let in_use = BTreeSet::from([
            nominee_ref("picture", "a"),
            nominee_ref("picture", "c"),
            nominee_ref("director", "d"),
        ]);

        let difference = diff(&stored, &incoming);

        assert_eq!(
            difference.removals_in_use(&in_use),
            [nominee_ref("picture", "a"), nominee_ref("director", "d")]
        );
        assert!(difference.removals_in_use(&BTreeSet::new()).is_empty());
    }
}
