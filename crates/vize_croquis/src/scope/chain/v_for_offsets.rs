use super::ScopeChain;
use crate::scope::ScopeId;

impl ScopeChain {
    pub(crate) fn set_v_for_source_offset(&mut self, id: ScopeId, offset: u32) {
        self.v_for_source_offsets.insert(id, offset);
    }

    /// Authored template offset where a v-for source expression begins.
    #[inline]
    pub fn v_for_source_offset(&self, id: ScopeId) -> Option<u32> {
        self.v_for_source_offsets.get(&id).copied()
    }
}
