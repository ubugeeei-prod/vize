//! script/prefer-ref-over-reactive
//!
//! Recommend using ref() over reactive() for state management.
//!
//! While reactive() can be convenient for objects, ref() is generally preferred
//! because:
//! - It works consistently with primitives and objects
//! - It makes the reactive nature explicit with `.value`
//! - It's easier to destructure and pass around without losing reactivity
//! - TypeScript inference is often better with ref()
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // reactive requires careful handling to avoid losing reactivity
//! const state = reactive({
//!   count: 0,
//!   name: 'foo'
//! })
//! ```
//!
//! ### Valid
//! ```ts
//! // ref is more explicit and safer
//! const count = ref(0)
//! const name = ref('foo')
//!
//! // For objects, ref still works
//! const user = ref({ name: 'foo', age: 20 })
//!
//! // Or use multiple refs for related data
//! const userName = ref('foo')
//! const userAge = ref(20)
//! ```

use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-ref-over-reactive",
    description: "Recommend using ref() over reactive() for state management",
    default_severity: Severity::Warning,
};

/// Prefer ref over reactive
pub struct PreferRefOverReactive;

impl ScriptRule for PreferRefOverReactive {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        PreferRefVisitor { offset, result }.visit_program(program);
    }
}

struct PreferRefVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for PreferRefVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let span = match call.callee.get_inner_expression() {
            Expression::Identifier(identifier) if identifier.name == "reactive" => {
                Some(identifier.span)
            }
            Expression::StaticMemberExpression(member) if member.property.name == "reactive" => {
                Some(member.property.span)
            }
            _ => None,
        };
        if let Some(span) = span {
            self.result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using ref() instead of reactive() for simpler state management",
                    self.offset as u32 + span.start,
                    self.offset as u32 + span.end,
                )
                .with_help(
                    "ref() is more explicit with `.value` access, easier to pass around, \
                     and avoids reactivity loss from destructuring. \
                     Use `const count = ref(0)` instead of `const state = reactive({ count: 0 })`",
                ),
            );
        }
        walk_call_expression(self, call);
    }
}

#[cfg(test)]
mod tests {
    use super::PreferRefOverReactive;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferRefOverReactive));
        linter
    }

    #[test]
    fn test_valid_ref() {
        let linter = create_linter();
        let result = linter.lint("const count = ref(0)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_ref_object() {
        let linter = create_linter();
        let result = linter.lint("const user = ref({ name: 'foo' })", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_reactive() {
        let linter = create_linter();
        let result = linter.lint("const state = reactive({ count: 0 })", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_non_ascii_before_reactive_does_not_panic() {
        let linter = create_linter();
        let result = linter.lint("const state = /*éé日*/reactive({ count: 0 })", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_shallow_reactive_not_matched() {
        let linter = create_linter();
        let result = linter.lint("const state = shallowReactive({ count: 0 })", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_reactive() {
        let linter = create_linter();
        let result = linter.lint("const x = 1", 0);
        assert_eq!(result.warning_count, 0);
    }
}
