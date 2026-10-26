//! Source-owned binder facts shared with the native expression transformer.

use vize_l0::{FxHashSet, String, id::NodeId};
use vize_l1_to_l2::{TransformScopeMark, exceptional_for_binding_names};
use vize_l2::op::ForOp;

use super::Emitter;
use crate::codegen::helpers::extract_destructure_params;

impl Emitter<'_, '_, '_, '_, '_, '_> {
    pub(super) fn enter_for_bindings(
        &mut self,
        for_op: &ForOp<'_>,
        aliases: [Option<&str>; 3],
        fact: u32,
    ) -> TransformScopeMark {
        let binding = &for_op.binding;
        let spans = [
            Some(binding.value.span()),
            binding.key.as_ref().map(|alias| alias.span()),
            binding.index.as_ref().map(|alias| alias.span()),
        ];
        let facts = NodeId::from_index(fact).and_then(|id| self.facts.scopes.get(id));
        let mark = self.exprs.enter_for_bindings(aliases, spans, facts);
        let mut params = FxHashSet::default();
        for (alias, span) in aliases.into_iter().zip(spans) {
            let Some(alias) = alias else {
                continue;
            };
            if let Some(names) =
                span.and_then(|span| exceptional_for_binding_names(alias, span, facts))
            {
                params.extend(names.map(String::from));
            } else if alias.is_ascii() && !alias.contains('\\') {
                // Preserve the existing SSR scanner and its ordinary output.
                extract_destructure_params(alias.trim(), &mut params);
            }
        }
        self.scoped_params.push(params);
        mark
    }
}
