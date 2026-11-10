//! Conventional bindings for compiler macros.

use super::{absolute_range, callee_name, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{Argument, BindingPattern, Expression, VariableDeclarator};
use oxc_ast_visit::{Visit, walk};
use vize_l0::cstr;

static META: RuleMeta = RuleMeta {
    name: "vue/require-macro-variable-name",
    description: "Require conventional macro variable names",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/require-macro-variable-name` with the upstream defaults.
pub struct RequireMacroVariableName;

impl Rule for RequireMacroVariableName {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = setup_program(ctx, |program, offset, _| {
            let mut visitor = MacroVariableNames {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            visitor.findings
        });
        for (macro_name, expected, range) in findings {
            ctx.warn_at_with_help(
                cstr!(
                    "The variable name of `{}` must be `{}`",
                    macro_name,
                    expected
                ),
                range,
                cstr!("Rename this binding to `{}`", expected),
            );
        }
    }
}

struct MacroVariableNames {
    offset: u32,
    findings: Vec<(&'static str, &'static str, ByteRange)>,
}

impl<'a> Visit<'a> for MacroVariableNames {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        let Some(initializer) = declarator.init.as_ref() else {
            return;
        };
        let Some((macro_name, expected)) = macro_binding(initializer) else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        if binding.name.as_str() != expected {
            self.findings.push((
                macro_name,
                expected,
                absolute_range(self.offset, binding.span),
            ));
        }
        walk::walk_variable_declarator(self, declarator);
    }
}

fn macro_binding(expression: &Expression<'_>) -> Option<(&'static str, &'static str)> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    let name = callee_name(call)?;
    let name = if name == "withDefaults" {
        call.arguments
            .first()
            .and_then(Argument::as_expression)
            .and_then(|argument| match argument.get_inner_expression() {
                Expression::CallExpression(inner) => callee_name(inner),
                _ => None,
            })?
    } else {
        name
    };
    match name {
        "defineProps" => Some(("defineProps", "props")),
        "defineEmits" => Some(("defineEmits", "emit")),
        "defineSlots" => Some(("defineSlots", "slots")),
        "useSlots" => Some(("useSlots", "slots")),
        "useAttrs" => Some(("useAttrs", "attrs")),
        _ => None,
    }
}
