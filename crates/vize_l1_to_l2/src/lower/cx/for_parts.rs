//! `ui.for` fact attachment for the lowering context.

use vize_davinci::id::NodeId;
use vize_l0::{Span, String, cstr};

use super::Cx;

impl Cx<'_> {
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
            let before = cstr!("scope {} bindings={binding_count}", parts.tag);
            let value = parts.value.spell();
            let key = parts.key.spell();
            let index = parts.index.spell();
            let mut after = String::with_capacity(
                "fact value=".len()
                    + value.len()
                    + " key=".len()
                    + key.len()
                    + " index=".len()
                    + index.len(),
            );
            after.push_str("fact value=");
            after.push_str(value);
            after.push_str(" key=");
            after.push_str(key);
            after.push_str(" index=");
            after.push_str(index);
            self.record("lower.for-fact", node, before.as_str(), after, span);
            self.for_facts.insert(id, parts);
        }
    }
}
