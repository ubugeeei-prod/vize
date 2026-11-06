//! `--report-unused-disable-directives` for template comments, matching
//! eslint-plugin-vue's `vue/comment-directive` option.

use vize_l0::cstr;

use crate::Linter;
use crate::context::LintContext;
use crate::context::SuppressionSource;
use crate::diagnostic::LintDiagnostic;

const RULE: &str = "vue/comment-directive";

/// Report each template disable pragma that suppressed nothing. A listed rule
/// counts only when it ran in this pass: a name Vize does not run may belong to
/// another linter.
pub(super) fn report(linter: &Linter, ctx: &mut LintContext<'_>) {
    // The catalogue rule owns this diagnostic. An explicit disable or a plan
    // without that catalogue owner retires it.
    if !linter.can_report_unused_directives() {
        return;
    }
    let unused = ctx.unused_disable_directives(|rule| {
        (linter.registry.has_rule(rule) || ctx.has_executed_template_rule(rule))
            && linter.is_rule_enabled(rule)
    });
    for directive in unused {
        let message = match (directive.source, &directive.rule) {
            (SuppressionSource::Eslint, Some(rule)) => {
                cstr!("Unused eslint-disable directive (no problems were reported from '{rule}').")
            }
            (SuppressionSource::Eslint, None) => {
                "Unused eslint-disable directive (no problems were reported).".into()
            }
            (SuppressionSource::VizeForget, Some(rule)) => {
                cstr!("Unused @vize:forget directive (no problems were reported from '{rule}').")
            }
            (SuppressionSource::VizeForget, None) => {
                "Unused @vize:forget directive (no problems were reported).".into()
            }
        };
        ctx.report_directive(LintDiagnostic::error(
            RULE,
            message,
            directive.start,
            directive.end,
        ));
    }
}
