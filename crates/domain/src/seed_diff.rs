//! What a re-import would change: the difference between the stored season and the one in the seed file.
//!
//! Entries are matched by key, never by label, so a corrected label keeps its picks (spec rule 4.1.2).

use std::collections::BTreeSet;

use crate::season::{CategoryKey, NomineeKey, Season};
use crate::time::UtcInstant;

/// A nominee as the diff names it: its category's key and its own.
pub type NomineeRef = (CategoryKey, NomineeKey);

/// How one kind of entry differs between the stored season and the incoming one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changes<Key> {
    /// Keys only the incoming season has, in its order.
    pub added: Vec<Key>,
    /// Keys both have, whose name or label differs, in the incoming order.
    pub relabeled: Vec<Key>,
    /// Keys only the stored season has, in its order.
    pub removed: Vec<Key>,
    /// Whether the entries both seasons have appear in a different order.
    pub reordered: bool,
}

impl<Key> Default for Changes<Key> {
    fn default() -> Self {
        Self {
            added: Vec::new(),
            relabeled: Vec::new(),
            removed: Vec::new(),
            reordered: false,
        }
    }
}

impl<Key> Changes<Key> {
    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.relabeled.is_empty() && self.removed.is_empty() && !self.reordered
    }
}

/// Everything an import of `incoming` over `stored` would change (spec rule 4.1.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonDiff {
    /// The stored lock instant and the incoming one, if they differ (spec rule 1.1.6).
    pub lock_instant: Option<(UtcInstant, UtcInstant)>,
    pub categories: Changes<CategoryKey>,
    /// The nominees of an added category are all added, and those of a removed category all removed.
    pub nominees: Changes<NomineeRef>,
}

impl SeasonDiff {
    /// Whether the import would change nothing.
    pub fn is_empty(&self) -> bool {
        self.lock_instant.is_none() && self.categories.is_empty() && self.nominees.is_empty()
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
        |stored, incoming| stored.name != incoming.name,
    );

    let mut nominees = Changes::default();
    for category in stored.categories() {
        // A removed category has no incoming nominees, so all of its own count as removed.
        let incoming_nominees = incoming
            .category(&category.key)
            .map_or(&[][..], |incoming| &incoming.nominees);
        let within = changes(
            category.nominees.iter().map(|nominee| (&nominee.key, nominee)),
            incoming_nominees.iter().map(|nominee| (&nominee.key, nominee)),
            |stored, incoming| stored.label != incoming.label || stored.secondary_label != incoming.secondary_label,
        );
        let in_category = |nominee: NomineeKey| (category.key.clone(), nominee);
        nominees.relabeled.extend(within.relabeled.into_iter().map(in_category));
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
        categories,
        nominees,
    }
}

/// Compares two sequences of keyed entries. `relabeled` says whether an entry both hold reads differently.
fn changes<'a, Key: Clone + Ord + 'a, Entry: 'a>(
    stored: impl Iterator<Item = (&'a Key, &'a Entry)>,
    incoming: impl Iterator<Item = (&'a Key, &'a Entry)>,
    relabeled: impl Fn(&Entry, &Entry) -> bool,
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
                if relabeled(stored_entry, entry) {
                    changes.relabeled.push((*key).clone());
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
    use crate::fixtures::{LOCK, category, category_key, nominee_key, season};

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
        assert!(difference.categories.relabeled.is_empty());
        assert!(difference.nominees.relabeled.is_empty());
        assert!(!difference.is_empty());
    }

    #[test]
    fn rule_4_1_2_a_changed_label_keeps_its_key() {
        let stored = season(&[("picture", &["a", "b"])]);
        let mut picture = category("picture", &["a", "b"]);
        picture.name = "Best Picture".into();
        picture.nominees[0].label = "A, corrected".into();
        picture.nominees[1].secondary_label = Some("Producer B".into());
        let incoming = Season::new(99, LOCK, vec![picture]).unwrap();

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.categories.relabeled, [category_key("picture")]);
        assert_eq!(
            difference.nominees.relabeled,
            [nominee_ref("picture", "a"), nominee_ref("picture", "b")]
        );
        assert!(difference.nominees.added.is_empty());
        assert!(difference.nominees.removed.is_empty());
    }

    #[test]
    fn rule_4_1_2_a_changed_key_is_a_removal_plus_an_addition() {
        let stored = season(&[("picture", &["a"])]);
        let incoming = season(&[("picture", &["a-renamed"])]);

        let difference = diff(&stored, &incoming);

        assert_eq!(difference.nominees.removed, [nominee_ref("picture", "a")]);
        assert_eq!(difference.nominees.added, [nominee_ref("picture", "a-renamed")]);
    }

    /// The part of spec rule 1.1.6 that needs no storage; plan item P4.3 closes the rule.
    #[test]
    fn a_new_lock_instant_is_reported_with_the_old_one() {
        let stored = season(&[("picture", &["a"])]);
        let later = UtcInstant::from_millis_since_epoch(LOCK.millis_since_epoch() + 3_600_000);
        let incoming = Season::new(99, later, stored.categories().to_vec()).unwrap();

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
    fn an_addition_between_two_entries_is_not_a_new_order() {
        let stored = season(&[("picture", &["a", "b"]), ("director", &["c"])]);
        let incoming = season(&[("picture", &["a", "x", "b"]), ("actor", &["y"]), ("director", &["c"])]);

        let difference = diff(&stored, &incoming);

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
