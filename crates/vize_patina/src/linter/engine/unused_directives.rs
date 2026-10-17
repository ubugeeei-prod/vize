//! `--report-unused-disable-directives` for template comments, matching
//! eslint-plugin-vue's `vue/comment-directive` option.

use vize_l0::cstr;

use crate::Linter;
use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;

const RULE: &str = "vue/comment-directive";

/// Report each template disable pragma that suppressed nothing. A listed rule
/// counts only when it ran in this pass: a name Vize does not run may belong to
/// another linter.
pub(super) fn report(linter: &Linter, ctx: &mut LintContext<'_>) {
    let unused = ctx.unused_disable_directives(|rule| {
        (linter.registry.has_rule(rule) || ctx.has_executed_template_rule(rule))
            && linter.is_rule_enabled(rule)
    });
    for directive in unused {
        let message = match &directive.rule {
            Some(rule) => {
                cstr!("Unused eslint-disable directive (no problems were reported from '{rule}').")
            }
            None => "Unused eslint-disable directive (no problems were reported).".into(),
        };
        ctx.report_directive(LintDiagnostic::error(
            RULE,
            message,
            directive.start,
            directive.end,
        ));
    }
}
