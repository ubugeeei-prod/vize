use vize_davinci::id::NodeId;
use vize_s0::{SmallVec, String};

use super::PatchFacts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StoredPatchFacts {
    flag: i32,
    dynamic_props: SmallVec<[String; 8]>,
}

impl StoredPatchFacts {
    pub(super) fn from_patch(facts: &PatchFacts) -> Self {
        Self {
            flag: facts.flag,
            dynamic_props: facts.dynamic_props.iter().cloned().collect(),
        }
    }
}

pub(in crate::emit) struct PatchFactsTable {
    // Keep owner IDs ordered so every emitted VNode does not scan all earlier
    // facts. Small components still use the inline allocation.
    entries: SmallVec<[(NodeId, StoredPatchFacts); 16]>,
}

impl PatchFactsTable {
    pub(in crate::emit) fn new() -> Self {
        Self {
            entries: SmallVec::new(),
        }
    }

    pub(super) fn materialize(&mut self, owner: Option<NodeId>, facts: PatchFacts) -> PatchFacts {
        let Some(owner) = owner else {
            return facts;
        };
        let stored = StoredPatchFacts::from_patch(&facts);
        match self.entries.binary_search_by_key(&owner, |(id, _)| *id) {
            Ok(index) => {
                if let Some(entry) = self.entries.get_mut(index) {
                    entry.1 = stored;
                }
            }
            Err(index) => self.entries.insert(index, (owner, stored)),
        }
        facts
    }

    #[cfg(any(test, feature = "davinci-differential"))]
    pub(in crate::emit) fn materialized_len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(super) fn get(&self, owner: NodeId) -> Option<&StoredPatchFacts> {
        self.entries
            .binary_search_by_key(&owner, |(id, _)| *id)
            .ok()
            .map(|index| &self.entries[index].1)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.materialized_len()
    }
}
