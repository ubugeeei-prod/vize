//! [`SideTable`] — analysis results keyed by [`NodeId`], not stored on nodes.
//!
//! Keeping facts beside the tree instead of inside it is what lets an S2 node
//! stay small and `Drop`-free while an arbitrary number of analyses attach to
//! it (`architecture.md`: "cross-stage references and analysis results are
//! `NodeId`-keyed tables, not fat nodes").
//!
//! # Residency: why this form is not arena-resident
//!
//! P1-10 proved the choice is not free. `vize_s0::{Box, Vec}` are
//! `oxc_allocator`'s, whose const assertion **rejects `Drop` payloads** — and
//! that assertion caught two real violations during P1-10 rather than
//! theoretical ones. So the two forms are not interchangeable:
//!
//! - The **sparse** form here is a `vize_s0::FxHashMap<NodeId, T>`: an
//!   ordinary non-arena scratch structure, free to hold a `T` that owns heap
//!   memory (a `String` message, a nested `Vec`), and freed when the table is.
//!   Analyses that touch a small fraction of the nodes want this.
//! - The **dense** form would be a `vize_s0::Vec<'a, Option<T>>` living in
//!   the compile arena, which constrains `T` to `Drop`-free and pays
//!   `size_of::<Option<T>>()` per node whether or not the analysis has a fact
//!   there.
//!
//! **P2-1 lands the sparse form only.** The dense form is documented, not
//! built, because building both now would fix a trade-off no measurement has
//! been taken on yet.
//!
//! ## Densification trigger
//!
//! Move an individual analysis to the dense form when **all** of the
//! following hold, measured rather than assumed:
//!
//! 1. **Occupancy is high** — the analysis stores a fact for a majority of the
//!    nodes in the artifact. Below that the `Option<T>` slots for absent
//!    facts cost more than the hash map's entries.
//! 2. **`T` is `Drop`-free** — otherwise the container const assertion rejects
//!    it and the fact has to be redesigned first, not waived.
//! 3. **The access pattern is lookup-dominated** — a dense table's win is the
//!    indexed read; a table that is built once and iterated once gains
//!    nothing.
//!
//! The measurement belongs to whichever pass proposes it, in its own PR, with
//! the alloc delta recorded the way `budgets.toml` requires. A blanket switch
//! is not a valid form of this change.
//!
//! # Iteration order
//!
//! Hash-map iteration order is unspecified, and folio pages require sorted map
//! iteration (`folio-format.md`). [`SideTable::iter`] is therefore **not** the
//! printing path: [`SideTable::sorted_entries`] is, and it says in its name
//! that it allocates.

use alloc::vec::Vec;

use vize_s0::FxHashMap;

use crate::id::NodeId;

/// Facts attached to nodes by [`NodeId`], stored beside the tree.
///
/// See the module docs for the residency decision and the densification
/// trigger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideTable<T> {
    entries: FxHashMap<NodeId, T>,
}

impl<T> Default for SideTable<T> {
    fn default() -> Self {
        Self {
            entries: FxHashMap::default(),
        }
    }
}

impl<T> SideTable<T> {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty table sized for `capacity` facts.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Attach `value` to `id`, returning the fact it replaced.
    pub fn insert(&mut self, id: NodeId, value: T) -> Option<T> {
        self.entries.insert(id, value)
    }

    /// The fact attached to `id`, if any.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.entries.get(&id)
    }

    /// The fact attached to `id`, mutably.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.entries.get_mut(&id)
    }

    /// Detach and return the fact attached to `id`.
    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        self.entries.remove(&id)
    }

    /// Whether `id` carries a fact.
    ///
    /// Named for its key the way `HashMap::contains_key` is, rather than a
    /// bare `contains`: a keyed table's membership question is about the id,
    /// not the value, and the bare name reads as a value search.
    #[must_use]
    pub fn contains_id(&self, id: NodeId) -> bool {
        self.entries.contains_key(&id)
    }

    /// How many nodes carry a fact.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no node carries a fact.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drop every fact, keeping the allocation.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Iterate the facts in **unspecified** order.
    ///
    /// Correct for folds and counts. Never use it to print a folio page or to
    /// build any other byte-stable artifact — see
    /// [`SideTable::sorted_entries`].
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.entries.iter().map(|(id, value)| (*id, value))
    }

    /// Iterate the facts mutably, in **unspecified** order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.entries.iter_mut().map(|(id, value)| (*id, value))
    }

    /// The facts in ascending [`NodeId`] order.
    ///
    /// This is the folio-printing path (`folio-format.md`: sorted map
    /// iteration). It allocates a `Vec` of references, which is why it is a
    /// separate, differently-named method rather than the `Iterator` impl:
    /// the cost is visible at the call site.
    #[must_use]
    pub fn sorted_entries(&self) -> Vec<(NodeId, &T)> {
        let mut entries: Vec<(NodeId, &T)> = self.iter().collect();
        entries.sort_unstable_by_key(|(id, _)| *id);
        entries
    }
}

impl<T> FromIterator<(NodeId, T)> for SideTable<T> {
    fn from_iter<I: IntoIterator<Item = (NodeId, T)>>(iter: I) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SideTable;
    use crate::id::NodeId;
    use alloc::vec;
    use alloc::vec::Vec;

    fn id(index: u32) -> NodeId {
        NodeId::from_index(index).expect("test indices are far below u32::MAX")
    }

    #[test]
    fn a_fresh_table_holds_nothing() {
        let table: SideTable<u8> = SideTable::new();
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
        assert_eq!(table.get(id(0)), None);
        assert!(!table.contains_id(id(0)));
    }

    #[test]
    fn insert_attaches_and_replaces() {
        let mut table = SideTable::new();
        assert_eq!(table.insert(id(3), "first"), None);
        assert_eq!(table.get(id(3)), Some(&"first"));
        assert_eq!(table.insert(id(3), "second"), Some("first"));
        assert_eq!(table.get(id(3)), Some(&"second"));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn remove_detaches_exactly_one_fact() {
        let mut table = SideTable::new();
        table.insert(id(1), 10);
        table.insert(id(2), 20);
        assert_eq!(table.remove(id(1)), Some(10));
        assert_eq!(table.remove(id(1)), None);
        assert_eq!(table.get(id(2)), Some(&20));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn get_mut_edits_in_place() {
        let mut table = SideTable::new();
        table.insert(id(0), 1u32);
        *table.get_mut(id(0)).expect("the fact was just inserted") += 41;
        assert_eq!(table.get(id(0)), Some(&42));
    }

    #[test]
    fn clear_drops_every_fact() {
        let mut table = SideTable::new();
        table.insert(id(0), 1);
        table.insert(id(9), 2);
        table.clear();
        assert!(table.is_empty());
        assert_eq!(table.get(id(0)), None);
    }

    #[test]
    fn sorted_entries_orders_by_id_whatever_the_insertion_order() {
        let mut table = SideTable::new();
        for index in [7u32, 0, 3, 12, 1] {
            table.insert(id(index), index);
        }
        let sorted: Vec<(u32, u32)> = table
            .sorted_entries()
            .into_iter()
            .map(|(node, value)| (node.index(), *value))
            .collect();
        assert_eq!(sorted, vec![(0, 0), (1, 1), (3, 3), (7, 7), (12, 12)]);
    }

    #[test]
    fn iter_visits_every_fact_exactly_once() {
        let mut table = SideTable::new();
        for index in 0..8u32 {
            table.insert(id(index), index);
        }
        let mut seen: Vec<u32> = table.iter().map(|(node, _)| node.index()).collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..8).collect::<Vec<u32>>());
    }

    #[test]
    fn from_iter_builds_the_same_table_as_repeated_insert() {
        let built: SideTable<u32> = [(id(0), 0u32), (id(4), 4)].into_iter().collect();
        let mut inserted = SideTable::new();
        inserted.insert(id(0), 0);
        inserted.insert(id(4), 4);
        assert_eq!(built, inserted);
    }
}
