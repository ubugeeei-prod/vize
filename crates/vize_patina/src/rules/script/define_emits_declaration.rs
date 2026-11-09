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

use oxc_ast::ast::{CallExpression, Expression, Program, Statement};
use oxc_span::Span;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};

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

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if sfc.is_script_setup && sfc.typescript {
            self.check_program(program, source, offset, result);
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
/// (a runtime argument rather than a type argument), return its call span.
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
        Some(call.span)
    } else {
        None
    }
}

/// Whether the call carries any runtime argument. Runtime declarations are
/// often imported identifiers (`defineEmits(buttonEmits)`), not only inline
/// arrays or objects.
fn has_runtime_argument(call: &CallExpression<'_>) -> bool {
    !call.arguments.is_empty()
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
    use crate::linter::{LintResult, Linter};

    fn lint(source: &str) -> LintResult {
        let sfc = format!("<script setup lang=\"ts\">{source}</script>");
        Linter::new()
            .with_enabled_rules(Some(vec!["script/define-emits-declaration".into()]))
            .lint_sfc(&sfc, "Probe.vue")
    }

    #[test]
    fn test_valid_type_based_inline() {
        let result = lint("const emit = defineEmits<{ change: [id: number] }>()");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_type_based_named() {
        let result = lint("const emit = defineEmits<Emits>()");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_runtime_array() {
        let result = lint("const emit = defineEmits(['change', 'update'])");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_runtime_object() {
        let result = lint("const emit = defineEmits({ change: (id: number) => true })");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_imported_runtime_declaration() {
        let result = lint("const emit = defineEmits(buttonEmits)");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_bare_expression_statement() {
        let result = lint("defineEmits(['change'])");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_define_emits() {
        let result = lint("const props = defineProps<{ name: string }>()");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_nested_define_emits_in_function() {
        // Only top-level macro calls are flagged; an identifier named
        // `defineEmits` used inside a function body is not the macro.
        let result = lint("function make() { return defineEmits(['change']) }");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_define_emits_in_string_literal() {
        // The pattern inside a string literal must not be flagged.
        let result = lint("const code = \"defineEmits(['change'])\"");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_type_and_runtime_argument() {
        // A runtime argument is flagged even if a type argument is also given.
        let result = lint("const emit = defineEmits<Emits>(['change'])");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_normal_script_is_not_a_macro_context() {
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["script/define-emits-declaration".into()]))
            .lint_sfc(
                "<script lang=\"ts\">const value = defineEmits(['change'])</script>",
                "Probe.vue",
            );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_javascript_script_setup_is_not_a_type_based_context() {
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["script/define-emits-declaration".into()]))
            .lint_sfc(
                "<script setup>const emit = defineEmits(['change'])</script>",
                "Probe.vue",
            );
        assert_eq!(result.warning_count, 0);
    }
}
