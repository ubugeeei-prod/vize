//! `ui.for` fact attachment for the lowering context.

use vize_l0::id::NodeId;
use vize_l0::{Span, cstr};
use vize_l2::provenance::ProvenanceSink;

use super::Cx;

impl<P: ProvenanceSink> Cx<'_, P> {
    /// Attach a `ui.for` binding view to its op, when the op has an id.
    pub(crate) fn attach_for_parts(
        &mut self,
        node: Option<NodeId>,
        parts: super::super::forop::ForParts,
        binding_count: usize,
        span: Span,
    ) {
        if let Some(id) = node {
            assert!(
                !self
                    .for_facts
                    .iter()
                    .any(|(_, existing)| existing.tag == parts.tag),
                "hygiene law broken: ui.for {id} reuses scope tag {} - introduction sites mint fresh tags",
                parts.tag,
            );
            self.record_text(
                "lower.for-fact",
                node,
                || cstr!("scope {} bindings={binding_count}", parts.tag),
                || {
                    cstr!(
                        "fact value={} key={} index={}",
                        parts.value.spell(),
                        parts.key.spell(),
                        parts.index.spell()
                    )
                },
                span,
            );
            self.for_facts.insert(id, parts);
        }
    }
}
