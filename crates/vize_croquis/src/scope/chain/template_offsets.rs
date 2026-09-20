use super::ScopeChain;
use crate::scope::{ScopeId, ScopeKind};

impl ScopeChain {
    pub(crate) fn set_v_for_source_offset(&mut self, id: ScopeId, offset: u32) {
        self.directive_expression_offsets.insert(id, offset);
    }

    /// Authored template offset where a v-for source expression begins.
    #[inline]
    pub fn v_for_source_offset(&self, id: ScopeId) -> Option<u32> {
        if self.get_scope(id)?.kind != ScopeKind::VFor {
            return None;
        }
        self.directive_expression_offsets.get(&id).copied()
    }
    pub(crate) fn set_v_slot_pattern_offset(&mut self, id: ScopeId, offset: u32) {
        self.directive_expression_offsets.insert(id, offset);
    }

    /// Authored start of the complete v-slot binding pattern, including property keys.
    pub fn v_slot_pattern_offset(&self, id: ScopeId) -> Option<u32> {
        if self.get_scope(id)?.kind != ScopeKind::VSlot {
            return None;
        }
        self.directive_expression_offsets.get(&id).copied()
    }
}
