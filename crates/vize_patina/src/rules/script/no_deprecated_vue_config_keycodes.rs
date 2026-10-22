//! script/no-deprecated-vue-config-keycodes
//!
//! `Vue.config.keyCodes` configured `v-on` key modifiers. Vue 3 removed it.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! Vue.config.keyCodes.enter = 13
//! ```
//!
//! ### Valid
//! ```ts
//! const enter = 13
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_static_member_expression};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-vue-config-keycodes",
    description: "Disallow Vue.config.keyCodes, which was removed in Vue 3",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "`Vue.config.keyCodes` are deprecated.";

pub struct NoDeprecatedVueConfigKeycodes;

impl ScriptRule for NoDeprecatedVueConfigKeycodes {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !source.contains("keyCodes") {
            return;
        }
        let mut visitor = KeycodesVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct KeycodesVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for KeycodesVisitor<'_> {
    fn visit_static_member_expression(&mut self, it: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if it.property.name.as_str() == "keyCodes" && is_vue_config(&it.object) {
            let span = it.span;
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result
                .add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end));
        }
        walk_static_member_expression(self, it);
    }
}

fn is_vue_config(expression: &Expression<'_>) -> bool {
    let Some(member) = member_of(expression) else {
        return false;
    };
    member.static_property_name() == Some("config") && is_vue(member.object())
}

fn is_vue(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name.as_str() == "Vue",
        Expression::ChainExpression(chain) => chain
            .expression
            .as_member_expression()
            .is_some_and(|member| is_vue(member.object())),
        _ => false,
    }
}

fn member_of<'a>(expression: &'a Expression<'a>) -> Option<&'a oxc_ast::ast::MemberExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
        other => other.as_member_expression(),
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedVueConfigKeycodes;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeprecatedVueConfigKeycodes));
        linter.lint(source, 0)
    }

    #[test]
    fn flags_vue_config_keycodes_only() {
        assert_eq!(lint("Vue.config.keyCodes.enter = 13").error_count, 1);
        assert_eq!(lint("config.keyCodes.enter = 13").error_count, 0);
    }
}
