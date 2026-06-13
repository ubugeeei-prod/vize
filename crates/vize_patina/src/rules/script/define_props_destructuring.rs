//! script/define-props-destructuring
//!
//! Disallow destructuring the return value of `defineProps` in `<script setup>`.
//!
//! Destructuring `defineProps()` (`const { foo } = defineProps(...)`) was, before
//! Vue 3.5, a reactivity foot-gun: the destructured bindings were plain values
//! that lost their reactive link to the props. Even with the reactive-props
//! destructure transform, projects that want to keep props access explicit
//! prefer holding the props object (`const props = defineProps(...)`) and reading
//! `props.foo`. This rule flags the destructuring binding itself.
//!
//! Both object and array destructuring of the macro result are reported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const { foo, bar } = defineProps<{ foo: string; bar?: number }>()
//! const [first] = defineProps<[string]>()
//! ```
//!
//! ### Valid
//! ```ts
//! const props = defineProps<{ foo: string }>()
//! // ...later: props.foo
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, Program, VariableDeclarator};
use oxc_ast_visit::{Visit, walk::walk_variable_declarator};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-props-destructuring",
    description: "Disallow destructuring the return value of defineProps in <script setup>",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "Avoid destructuring the return value of defineProps().";
const HELP: &str = "Assign the props to a single binding (`const props = defineProps(...)`) and \
     access `props.foo` instead of destructuring, which can drop reactivity.";

/// Disallow destructuring the result of `defineProps`.
pub struct DefinePropsDestructuring;

impl ScriptRule for DefinePropsDestructuring {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = DefinePropsDestructuringVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct DefinePropsDestructuringVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for DefinePropsDestructuringVisitor<'_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init
            && is_define_props_call(init)
            && let Some(span) = destructuring_pattern_span(&it.id)
        {
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(META.name, MESSAGE, start, end)
                    .with_label("destructured defineProps() result", start, end)
                    .with_help(HELP),
            );
        }
        walk_variable_declarator(self, it);
    }
}

/// The span of a destructuring binding pattern (object or array), or `None` for a
/// plain identifier binding (`const props = ...`).
fn destructuring_pattern_span(pattern: &BindingPattern<'_>) -> Option<Span> {
    match pattern {
        BindingPattern::ObjectPattern(object) => Some(object.span),
        BindingPattern::ArrayPattern(array) => Some(array.span),
        _ => None,
    }
}

/// Whether `expression` is a bare `defineProps(...)` call, looking through
/// `withDefaults(defineProps(...), ...)` so the destructured-withDefaults form is
/// also covered.
fn is_define_props_call(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression else {
        return false;
    };
    if call_is_named(call, "defineProps") {
        return true;
    }
    // `const { x } = withDefaults(defineProps<...>(), { ... })`
    if call_is_named(call, "withDefaults")
        && let Some(first) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
    {
        return is_define_props_call(first);
    }
    false
}

fn call_is_named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == name
    )
}

#[cfg(test)]
mod tests {
    use super::DefinePropsDestructuring;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefinePropsDestructuring));
        linter
    }

    #[test]
    fn test_valid_assigned_to_identifier() {
        let result = create_linter().lint("const props = defineProps<{ foo: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_assignment() {
        let result = create_linter().lint("defineProps<{ foo: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_unrelated_destructure() {
        let result = create_linter().lint("const { foo } = someObject", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_object_destructure() {
        let result = create_linter().lint(
            "const { foo, bar } = defineProps<{ foo: string; bar?: number }>()",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_object_destructure_with_defaults() {
        let result =
            create_linter().lint("const { count = 0 } = defineProps<{ count?: number }>()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_array_destructure() {
        let result = create_linter().lint("const [first] = defineProps<[string]>()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_with_defaults_destructure() {
        let result = create_linter().lint(
            "const { count = 0 } = withDefaults(defineProps<{ count?: number }>(), { count: 0 })",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_offset_applied() {
        let source = "const { foo } = defineProps<{ foo: string }>()";
        let result = create_linter().lint(source, 200);
        assert_eq!(result.warning_count, 1);
        let pattern_start = source.find('{').unwrap() as u32 + 200;
        assert_eq!(result.diagnostics[0].start, pattern_start);
    }
}
