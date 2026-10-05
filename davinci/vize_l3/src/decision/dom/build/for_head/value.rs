//! Current original callback value, without script provenance or another walk.

use super::super::{DomBuilder, DomExpressionFacts};
use crate::decision::dom::vue::{VueReadKind, policy::FileReads};
use vize_l2::{
    expr::JsExpr,
    file::{BindingRef, FileResolution},
    op::Op,
    resolution::{ForAliasRole, Occurrence, Usage},
};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(in crate::decision::dom::build) fn for_value(
        &self,
        resolution: FileResolution<'owner, 'arena>,
        expression: &JsExpr<'arena>,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        if !self.in_original_for_body() || occurrence.usage != Usage::Read {
            return None;
        }
        let [(origin, frame), (_, body)] = self.frames.as_slice() else {
            return None;
        };
        let Op::OriginalFor(original) = frame.node.op else {
            return None;
        };
        if !matches!(body.node.op, Op::Element(element) if element.attributes.is_empty()) {
            return None;
        }
        let file = self.file?;
        let [original_occurrence] = resolution.table()?.occurrences() else {
            return None;
        };
        if !core::ptr::eq(original_occurrence, occurrence)
            || !crate::decision::dom::file::matches_expression(resolution, expression)
        {
            return None;
        }
        let row = self.facts.file_for_heads.get(*origin)?;
        let expected = row.resolution().value_declaration();
        let head = row.head();
        let alias = head.value()?;
        let scope = head.scope()?;
        let identifier = expression.ast.get_identifier_reference()?;
        if !core::ptr::eq(resolution.file(), file)
            || !core::ptr::eq(binding.file(), file)
            || !row.accepts_original(original)
            || resolution.scope() != Some(scope)
            || expected.fact().role() != ForAliasRole::Value
            || binding.id() != alias.id()
            || !binding.same_owner(alias)
            || occurrence.binding != binding.id()
            || occurrence.name != expected.fact().name()
            || identifier.name.as_str() != occurrence.name
            || expression.source != occurrence.name
            || expression.ast_span_to_source(identifier.span) != Some(occurrence.span)
            || expression
                .source
                .get(occurrence.span.start as usize..occurrence.span.end as usize)
                != Some(occurrence.name)
            || !super::alias(Some(binding), &expected, file, head.id(), scope)
        {
            return None;
        }
        Some(VueReadKind::ForValue)
    }
}

#[cfg(test)]
mod tests;
