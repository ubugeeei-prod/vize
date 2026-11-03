//! A render read is an echo only after its real typed primary was emitted.

use super::TemplateValueChecks;
use crate::virtual_ts::{VizeMapping, diagnostic_ownership::InputOwner};
use vize_croquis::{TemplateExpression, TemplateExpressionKind};

pub(super) fn record(
    expression: &TemplateExpression,
    mappings: &[VizeMapping],
    template_offset: u32,
    checks: TemplateValueChecks<'_>,
) {
    let Some(collector) = checks.diagnostic_ownership else {
        return;
    };
    let owner = match expression.kind {
        // A component v-model value is a prop like `:prop` (#11655).
        TemplateExpressionKind::VBind | TemplateExpressionKind::VModel => checks
            .prop_expression_owners
            .and_then(|owners| {
                owners.get(&(
                    expression.scope_id.as_u32(),
                    expression.start,
                    expression.end,
                ))
            })
            .copied(),
        TemplateExpressionKind::VOn => Some(InputOwner::for_listener_reference(
            expression.scope_id.as_u32(),
            expression.start,
            expression.end,
        )),
        _ => None,
    };
    let Some(owner) = owner else { return };
    let source = template_offset as usize + expression.start as usize
        ..template_offset as usize + expression.end as usize;
    if let Some(mapping) = mappings.iter().find(|row| row.src_range == source) {
        collector
            .borrow_mut()
            .record_bare_navigation(owner, mapping.gen_range.clone(), source);
    }
}
