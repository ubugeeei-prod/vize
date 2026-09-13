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
