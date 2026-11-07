use std::ops::Range;

use super::ScopeChain;
use crate::scope::{ScopeId, ScopeKind};

impl ScopeChain {
    pub(crate) fn set_v_for_source_span(&mut self, id: ScopeId, span: Range<u32>) {
        self.v_for_source_spans.insert(id, span);
    }

    /// Authored template offset where a v-for source expression begins.
    #[inline]
    pub fn v_for_source_offset(&self, id: ScopeId) -> Option<u32> {
        self.v_for_source_span(id).map(|span| span.start)
    }

    /// Authored template byte range from the v-for source through its expression end.
    #[inline]
    pub fn v_for_source_span(&self, id: ScopeId) -> Option<Range<u32>> {
        if self.get_scope(id)?.kind != ScopeKind::VFor {
            return None;
        }
        self.v_for_source_spans.get(&id).cloned()
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
