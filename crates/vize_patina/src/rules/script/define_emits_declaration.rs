//! script/define-emits-declaration
//!
//! Enforce the type-based `defineEmits<{ ... }>()` declaration over the
//! runtime/array form `defineEmits([ ... ])` in `<script setup>`.
//!
//! The type-based form gives stronger, self-documenting type information for
//! emitted events and their payloads, and is the recommended style for
//! TypeScript single-file components.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // Runtime array form
//! const emit = defineEmits(['change', 'update'])
//!
//! // Runtime object form
//! const emit = defineEmits({
//!   change: (id: number) => true,
//! })
//! ```
//!
//! ### Valid
//! ```ts
//! // Type-based form (preferred)
//! const emit = defineEmits<{ change: [id: number] }>()
//!
//! // Named type alias
//! const emit = defineEmits<Emits>()
//! ```

use oxc_ast::ast::{Argument, CallExpression, Expression, Program, Statement};
use oxc_span::Span;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-emits-declaration",
    description: "Enforce the type-based defineEmits<{}>() form over the runtime/array form",
    default_severity: Severity::Warning,
};

/// Enforce type-based `defineEmits<{}>()` over the runtime/array form.
pub struct DefineEmitsDeclaration;

impl ScriptRule for DefineEmitsDeclaration {
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
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // `defineEmits` is a compiler macro: it is only valid at the top level
        // of `<script setup>`, so we inspect top-level statements directly
        // rather than walking the whole tree (mirrors the defineProps macro
        // detection approach).
        for statement in &program.body {
            for call in top_level_calls(statement) {
                if let Some(span) = runtime_define_emits(call) {
                    report(result, offset, span);
                }
            }
        }
    }
}

/// Collect the call expressions that appear directly at the top level of a
/// single statement: a bare `defineEmits(...)` expression statement, or the
/// initializer of `const emit = defineEmits(...)`.
fn top_level_calls<'a, 'b>(statement: &'b Statement<'a>) -> Vec<&'b CallExpression<'a>> {
    match statement {
        Statement::ExpressionStatement(expression_statement) => {
            unwrap_call(&expression_statement.expression)
                .into_iter()
                .collect()
        }
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .filter_map(unwrap_call)
            .collect(),
        _ => Vec::new(),
    }
}

/// Strip TS-only wrappers / parentheses so the underlying call is seen, then
/// return it if the expression is (ultimately) a call expression.
fn unwrap_call<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => unwrap_call(&paren.expression),
        Expression::TSAsExpression(ts_as) => unwrap_call(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => unwrap_call(&ts_satisfies.expression),
        Expression::TSNonNullExpression(ts_non_null) => unwrap_call(&ts_non_null.expression),
        _ => None,
    }
}

/// If `call` is a `defineEmits(...)` call that uses the runtime/array form
/// (a runtime argument rather than a type argument), return its callee span.
///
/// A call with a type argument (`defineEmits<Emits>()`) and no runtime
/// argument is the preferred form and yields `None`.
fn runtime_define_emits(call: &CallExpression<'_>) -> Option<Span> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name.as_str() != "defineEmits" {
        return None;
    }

    // Prefer the type-based form: a runtime argument (array or object literal)
    // is what we flag. A type-argument-only call is valid.
    if has_runtime_argument(call) {
        Some(callee.span)
    } else {
        None
    }
}

/// Whether the call carries a runtime argument (an array or object literal),
/// i.e. the runtime/array form of `defineEmits`.
fn has_runtime_argument(call: &CallExpression<'_>) -> bool {
    call.arguments.iter().any(|argument| {
        matches!(
            argument,
            Argument::ArrayExpression(_) | Argument::ObjectExpression(_)
        )
    })
}

fn report(result: &mut ScriptLintResult, offset: usize, span: Span) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::warn(
            META.name,
            "Prefer the type-based defineEmits<{}>() declaration over the runtime/array form",
            start,
            end,
        )
        .with_help(
            "Use the type-based form: `const emit = defineEmits<{ change: [id: number] }>()`",
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::DefineEmitsDeclaration;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefineEmitsDeclaration));
        linter
    }

    #[test]
    fn test_valid_type_based_inline() {
        let linter = create_linter();
        let result = linter.lint("const emit = defineEmits<{ change: [id: number] }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_type_based_named() {
        let linter = create_linter();
        let result = linter.lint("const emit = defineEmits<Emits>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_runtime_array() {
        let linter = create_linter();
        let result = linter.lint("const emit = defineEmits(['change', 'update'])", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_runtime_object() {
        let linter = create_linter();
        let result = linter.lint(
            "const emit = defineEmits({ change: (id: number) => true })",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_bare_expression_statement() {
        let linter = create_linter();
        let result = linter.lint("defineEmits(['change'])", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_define_emits() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps<{ name: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_nested_define_emits_in_function() {
        // Only top-level macro calls are flagged; an identifier named
        // `defineEmits` used inside a function body is not the macro.
        let linter = create_linter();
        let result = linter.lint("function make() { return defineEmits(['change']) }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_define_emits_in_string_literal() {
        // The pattern inside a string literal must not be flagged.
        let linter = create_linter();
        let result = linter.lint("const code = \"defineEmits(['change'])\"", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_type_and_runtime_argument() {
        // A runtime argument is flagged even if a type argument is also given.
        let linter = create_linter();
        let result = linter.lint("const emit = defineEmits<Emits>(['change'])", 0);
        assert_eq!(result.warning_count, 1);
    }
}
