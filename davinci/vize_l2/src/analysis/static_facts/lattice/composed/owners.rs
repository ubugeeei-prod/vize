//! Every owner fact is one of three levels times sixteen flag combinations.
//! Nearby page ids index a single byte. Isolated owners retain their full id
//! in a sparse map, independently of the total size of the artifact.

use super::StaticFacts;
use crate::analysis::static_facts::StaticLevel;
use alloc::vec::Vec;
use vize_l0::{FxHashMap, SmallVec, id::NodeId};

/// The original stage-local id, copied by value when hashing or exposing it.
/// Its byte alignment lets the hash table keep each complete bucket in five bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, packed)]
struct Key(NodeId);

#[derive(Clone, Copy)]
struct Code(u8);

const _: () = assert!(core::mem::size_of::<(Key, Code)>() == 5);

impl Code {
    fn of(fact: StaticFacts) -> Self {
        let level = match fact.level {
            StaticLevel::NotStatic => 0,
            StaticLevel::HasDynamicText => 1,
            StaticLevel::FullyStatic => 2,
        };
        Self(
            level * 16
                + u8::from(fact.props_hoistable)
                + (u8::from(fact.nested_static) << 1)
                + (u8::from(fact.native_descendants) << 2)
                + (u8::from(fact.foreign) << 3),
        )
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "the private constructor produces exactly the 48 lattice indices"
    )]
    #[inline]
    fn fact(self) -> &'static StaticFacts {
        &LATTICE[usize::from(self.0)]
    }
}

const fn decode(index: usize) -> StaticFacts {
    let level = match index / 16 {
        0 => StaticLevel::NotStatic,
        1 => StaticLevel::HasDynamicText,
        _ => StaticLevel::FullyStatic,
    };
    StaticFacts {
        level,
        props_hoistable: index & 1 != 0,
        nested_static: index & 2 != 0,
        native_descendants: index & 4 != 0,
        foreign: index & 8 != 0,
    }
}
#[expect(
    clippy::indexing_slicing,
    reason = "the loop bound is the length of the same compile-time array"
)]
const LATTICE: [StaticFacts; 48] = {
    let mut facts = [decode(0); 48];
    let mut index = 1;
    while index < facts.len() {
        facts[index] = decode(index);
        index += 1;
    }
    facts
};

const ABSENT: u8 = u8::MAX;
const INLINE: usize = 16;

/// Read-only owner facts in the artifact's original page id space.
#[derive(Default)]
pub struct OwnerFacts {
    dense: SmallVec<[u8; INLINE]>,
    dense_owners: usize,
    first_sparse: Option<NodeId>,
    sparse: FxHashMap<Key, Code>,
}

impl OwnerFacts {
    pub(super) fn insert(&mut self, id: NodeId, fact: StaticFacts) {
        let index = id.index_usize();
        let code = Code::of(fact);
        if let Some(slot) = self.dense.get_mut(index) {
            self.dense_owners += usize::from(*slot == ABSENT);
            *slot = code.0;
            return;
        }
        // Bound the direct table by its actual owner population. Once a distant
        // id is stored in the sparse table the two ranges always stay disjoint,
        // even when the postorder traversal next publishes a lower parent id.
        if self.first_sparse.is_none_or(|first| id < first)
            && index < INLINE.max(self.dense_owners.saturating_mul(4))
        {
            self.dense.resize(index, ABSENT);
            self.dense.push(code.0);
            self.dense_owners += 1;
        } else {
            self.first_sparse = Some(self.first_sparse.map_or(id, |first| first.min(id)));
            self.sparse.insert(Key(id), code);
        }
    }

    /// The static lattice value for this owner. Non-owners have no entry.
    #[must_use]
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&StaticFacts> {
        match self.dense.as_slice().get(id.index_usize()) {
            Some(&code) => LATTICE.get(usize::from(code)),
            None => self.sparse.get(&Key(id)).map(|code| code.fact()),
        }
    }

    /// Number of published owners.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.dense_owners + self.sparse.len()
    }

    /// True when there are no owners.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.dense_owners == 0 && self.sparse.is_empty()
    }

    /// Facts in unspecified order, for folds over the complete result.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &StaticFacts)> {
        let dense = self.dense.iter().enumerate().filter_map(|(index, &code)| {
            Some((
                NodeId::from_index(index as u32)?,
                LATTICE.get(usize::from(code))?,
            ))
        });
        dense.chain(self.sparse.iter().map(|(key, code)| (key.0, code.fact())))
    }

    /// Materialize the same ascending page order as the standalone side table.
    #[must_use]
    pub fn sorted_entries(&self) -> Vec<(NodeId, &StaticFacts)> {
        let mut entries: Vec<_> = self.iter().collect();
        if !self.sparse.is_empty() {
            entries.sort_unstable_by_key(|(id, _)| *id);
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_descendants_and_earlier_parents_keep_full_ids_and_replace_in_place() {
        let mut owners = OwnerFacts::default();
        let mut original = vize_l0::side_table::SideTable::default();
        for (index, code) in [
            (4, 2),
            (1_000_000, 31),
            (26, 10),
            (10, 15),
            (27, 12),
            (4, 46),
            (26, 7),
            (0, 23),
        ] {
            let id = NodeId::from_index(index).unwrap();
            owners.insert(id, LATTICE[code]);
            original.insert(id, LATTICE[code]);
        }
        assert_eq!(owners.len(), original.len());
        assert_eq!(owners.sorted_entries(), original.sorted_entries());
        assert!(owners.dense.len() <= INLINE);
        for index in [0, 1, 4, 10, 15, 16, 23, 26, 27, 990_000, 1_000_000] {
            let id = NodeId::from_index(index).unwrap();
            assert_eq!(owners.get(id), original.get(id));
        }
    }

    #[test]
    fn every_level_and_flag_combination_round_trips_at_full_width_ids() {
        let mut owners = OwnerFacts::default();
        let last = NodeId::from_index(u32::MAX - 1).unwrap();
        for (index, fact) in LATTICE.iter().enumerate() {
            let id = NodeId::from_index(index as u32).unwrap();
            owners.insert(id, *fact);
            assert_eq!(owners.get(id), Some(fact));
            assert_eq!(Code::of(*fact).0 as usize, index);
        }
        assert_eq!(owners.get(last), None);
        owners.insert(last, LATTICE[47]);
        assert_eq!(owners.get(last), Some(&LATTICE[47]));
        assert_eq!(
            owners.sorted_entries().last().map(|(id, _)| *id),
            Some(last)
        );
    }
}
