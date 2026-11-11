//! Defaults on `defineModel` declarations.

use super::{absolute_range, property_span, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::script::setup_macros::{DefineModel, setup_macros, skip_ts_as};
use oxc_ast::ast::Expression;

static META: RuleMeta = RuleMeta {
    name: "vue/no-define-model-default",
    description: "Disallow default values in defineModel",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-define-model-default`.
pub struct NoDefineModelDefault;

impl Rule for NoDefineModelDefault {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = setup_program(ctx, |program, offset, _| {
            setup_macros(program)
                .into_iter()
                .filter(|found| found.name == "defineModel")
                .filter_map(
                    |found| match DefineModel::of(found.call).options.map(skip_ts_as) {
                        Some(Expression::ObjectExpression(options)) => {
                            property_span(options, "default")
                        }
                        _ => None,
                    },
                )
                .map(|span| absolute_range(offset, span))
                .collect::<Vec<_>>()
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Unexpected default value in `defineModel`",
                finding,
                "Keep the parent and child model values synchronized by applying the default in the parent",
            );
        }
    }
}
