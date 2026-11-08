//! Bounded control-flow emission for giant flat interpolation lists.

use super::{ExpressionListEmitContext, generate_expression};
use crate::virtual_ts::{template_binding_access::TemplateBindingAccess, types::VizeMapping};
use vize_carton::{String, append, cstr};
use vize_croquis::croquis::{TemplateExpression, TemplateExpressionKind};

/// Keep the outer setup control-flow graph bounded even when one SFC owns
/// thousands of independent interpolations. TypeScript still checks every
/// uninvoked arrow body, but no body contributes more than this many reads.
const INTERPOLATION_CFA_CHUNK: usize = 256;
const INTERPOLATION_CFA_ISOLATION_THRESHOLD: usize = INTERPOLATION_CFA_CHUNK * 2;

/// A plain interpolation has no render-branch narrowing to preserve. Splitting
/// a sufficiently large all-interpolation list therefore changes only the
/// checker lifecycle: from one O(N) control-flow graph to O(N / chunk) small
/// graphs. Keep `await` in the setup function that established its async
/// context instead of accidentally making it legal or illegal in a new arrow.
pub(super) fn should_isolate_interpolations(
    exprs: &[&TemplateExpression],
    context: &ExpressionListEmitContext<'_>,
) -> bool {
    let mut active = 0;
    for expr in exprs {
        if context
            .skipped_expression_ranges
            .contains(&(expr.start, expr.end))
        {
            continue;
        }
        if expr.kind != TemplateExpressionKind::Interpolation
            || expr.vif_guard.is_some()
            || expr.content.contains("await")
        {
            return false;
        }
        active += 1;
    }
    active >= INTERPOLATION_CFA_ISOLATION_THRESHOLD
}

pub(super) fn generate_isolated_interpolations(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    template_binding_access: &TemplateBindingAccess,
    context: &ExpressionListEmitContext<'_>,
) {
    let inner_indent = cstr!("{}  ", context.indent);
    let mut emitted = 0;
    for expr in exprs {
        if context
            .skipped_expression_ranges
            .contains(&(expr.start, expr.end))
        {
            continue;
        }
        if emitted % INTERPOLATION_CFA_CHUNK == 0 {
            append!(*ts, "{}void (() => {{\n", context.indent);
        }
        generate_expression(
            ts,
            mappings,
            expr,
            template_binding_access,
            context.template_offset,
            inner_indent.as_str(),
            context.checks,
        );
        emitted += 1;
        if emitted % INTERPOLATION_CFA_CHUNK == 0 {
            append!(*ts, "{}}});\n", context.indent);
        }
    }
    if emitted % INTERPOLATION_CFA_CHUNK != 0 {
        append!(*ts, "{}}});\n", context.indent);
    }
}
