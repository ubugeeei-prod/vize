//! Defaults on `defineModel` declarations.

use super::{absolute_range, callee_name, object_argument, property_span, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::{Visit, walk};

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
            let mut visitor = DefineModelDefaults {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            visitor.findings
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

struct DefineModelDefaults {
    offset: u32,
    findings: Vec<ByteRange>,
}

impl<'a> Visit<'a> for DefineModelDefaults {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if callee_name(call) == Some("defineModel")
            && let Some(object) = call.arguments.iter().rev().find_map(object_argument)
            && let Some(span) = property_span(object, "default")
        {
            self.findings.push(absolute_range(self.offset, span));
        }
        walk::walk_call_expression(self, call);
    }
}
