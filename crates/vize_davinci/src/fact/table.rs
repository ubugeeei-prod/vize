//! [`FactTable`] — one group's facts, as a query returns them.

use alloc::vec::Vec;
use core::fmt;

use super::FactGroup;

/// One fact group's table: facts sorted by key.
///
/// Read-mostly by design — a producer builds it once per artifact and every
/// consumer borrows it — so the storage is a key-sorted vector: lookups are
/// a binary search over contiguous memory, iteration is already in key
/// order (what folio pages and the P4-2 α export need), and there is no
/// hashing. Keys are unique.
pub struct FactTable<G: FactGroup> {
    entries: Vec<(G::Key, G::Value)>,
}

impl<G: FactGroup> FactTable<G> {
    /// The fact stored under `key`.
    #[must_use]
    pub fn get(&self, key: &G::Key) -> Option<&G::Value> {
        self.entries
            .binary_search_by(|(probe, _)| probe.cmp(key))
            .ok()
            .map(|at| &self.entries[at].1)
    }

    /// Whether a fact is stored under `key`.
    #[must_use]
    pub fn contains_key(&self, key: &G::Key) -> bool {
        self.get(key).is_some()
    }

    /// How many facts the table holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table holds no facts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every fact, in ascending key order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&G::Key, &G::Value)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }
}

impl<G: FactGroup> Default for FactTable<G> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<G: FactGroup> PartialEq for FactTable<G>
where
    G::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<G: FactGroup> Eq for FactTable<G> where G::Value: Eq {}

impl<G: FactGroup> fmt::Debug for FactTable<G>
where
    G::Key: fmt::Debug,
    G::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<G: FactGroup> FromIterator<(G::Key, G::Value)> for FactTable<G> {
    /// Collect facts; a repeated key keeps its **last** value, as a map
    /// insert would.
    fn from_iter<I: IntoIterator<Item = (G::Key, G::Value)>>(iter: I) -> Self {
        let mut builder = FactTableBuilder::default();
        for (key, value) in iter {
            builder.insert(key, value);
        }
        builder.finish()
    }
}

/// Accumulates a producer's facts, then freezes them into a [`FactTable`].
pub struct FactTableBuilder<G: FactGroup> {
    entries: Vec<(G::Key, G::Value)>,
}

impl<G: FactGroup> Default for FactTableBuilder<G> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<G: FactGroup> FactTableBuilder<G> {
    /// A builder with room for `capacity` facts.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Record a fact. A later insert under the same key wins.
    pub fn insert(&mut self, key: G::Key, value: G::Value) {
        self.entries.push((key, value));
    }

    /// Sort by key and keep the last value of every repeated key.
    #[must_use]
    pub fn finish(self) -> FactTable<G> {
        let mut entries = self.entries;
        // Stable, so equal keys stay in insertion order and the last
        // inserted one is the last of its run.
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        let mut unique: Vec<(G::Key, G::Value)> = Vec::with_capacity(entries.len());
        for entry in entries {
            match unique.last_mut() {
                Some(last) if last.0 == entry.0 => *last = entry,
                _ => unique.push(entry),
            }
        }
        FactTable { entries: unique }
    }
}

#[cfg(test)]
mod tests {
    use super::{FactTable, FactTableBuilder};
    use crate::fact::{Demand, FactGroup};
    use crate::pass::AnalysisId;
    use alloc::vec::Vec;

    struct Names;
    impl FactGroup for Names {
        const ID: AnalysisId = AnalysisId::new(0);
        const NAME: &'static str = "names";
        const STRATUM: u8 = 0;
        const DEPENDS: Demand = Demand::NONE;
        type Key = u32;
        type Value = &'static str;
    }

    #[test]
    fn a_built_table_is_key_sorted_and_last_insert_wins() {
        let mut builder = FactTableBuilder::<Names>::with_capacity(4);
        builder.insert(9, "nine");
        builder.insert(2, "two");
        builder.insert(9, "NINE");
        builder.insert(5, "five");
        let table = builder.finish();
        let entries: Vec<(u32, &str)> = table.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(entries, [(2, "two"), (5, "five"), (9, "NINE")]);
        assert_eq!((table.get(&9), table.get(&3)), (Some(&"NINE"), None));
        assert_eq!((table.len(), table.contains_key(&5)), (3, true));
    }

    #[test]
    fn collecting_is_building() {
        let table: FactTable<Names> = [(3, "c"), (1, "a"), (3, "C")].into_iter().collect();
        let mut expected = FactTableBuilder::<Names>::default();
        expected.insert(1, "a");
        expected.insert(3, "C");
        assert_eq!(table, expected.finish());
        assert!(FactTable::<Names>::default().is_empty());
    }
}
