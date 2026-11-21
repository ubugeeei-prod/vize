//! Every owner fact is one of three levels times sixteen flag combinations.
//! A compact sparse key and a lattice index keep a table resize from retaining
//! two full owner maps alongside the growing complexity breakdown.

use super::StaticFacts;
use crate::analysis::static_facts::StaticLevel;
use alloc::vec::Vec;
use vize_l0::{FxHashMap, id::NodeId};

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

/// Read-only owner facts in the artifact's original page id space.
#[derive(Default)]
pub struct OwnerFacts {
    entries: FxHashMap<Key, Code>,
}

impl OwnerFacts {
    pub(super) fn insert(&mut self, id: NodeId, fact: StaticFacts) {
        self.entries.insert(Key(id), Code::of(fact));
    }

    /// The static lattice value for this owner. Non-owners have no entry.
    #[must_use]
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&StaticFacts> {
        self.entries.get(&Key(id)).map(|code| code.fact())
    }

    /// Number of published owners.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when there are no owners.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Facts in unspecified order, for folds over the complete result.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &StaticFacts)> {
        self.entries.iter().map(|(key, code)| (key.0, code.fact()))
    }

    /// Materialize the same ascending page order as the standalone side table.
    #[must_use]
    pub fn sorted_entries(&self) -> Vec<(NodeId, &StaticFacts)> {
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort_unstable_by_key(|(id, _)| *id);
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
