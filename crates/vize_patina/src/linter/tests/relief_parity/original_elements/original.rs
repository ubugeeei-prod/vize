//! Independent original configured rules and their original depth walk.

use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use vize_l0::String;

mod max_depth;
mod restricted_html;
mod template_walk;

pub(super) use max_depth::MaxTemplateDepth;
pub(super) use restricted_html::NoRestrictedHtmlElements;

fn report_template(
    ctx: &mut LintContext<'_>,
    rule: &'static str,
    findings: Vec<(String, u32, u32)>,
) {
    for (message, start, end) in findings {
        if end > start {
            ctx.report(LintDiagnostic::warn(rule, message, start, end));
        }
    }
}
