//! script/prefer-use-template-ref
//!
//! Recommend useTemplateRef over ref for template references.
//!
//! Since Vue 3.5, useTemplateRef() is the recommended way to obtain template refs.
//! It provides better type inference and clearer intent compared to using
//! ref(null) with a matching template ref attribute.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // Old pattern (less clear intent)
//! const input = ref<HTMLInputElement | null>(null)
//! // <input ref="input" />
//!
//! // Using ref with null for template refs
//! const myElement = ref(null)
//! ```
//!
//! ### Valid
//! ```ts
//! // New pattern (Vue 3.5+)
//! const input = useTemplateRef<HTMLInputElement>('input')
//! // <input ref="input" />
//!
//! // Regular refs for reactive data (not template refs)
//! const count = ref(0)
//! const name = ref('hello')
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-use-template-ref",
    description: "Recommend useTemplateRef over ref(null) for template references (Vue 3.5+)",
    default_severity: Severity::Warning,
};

/// Prefer useTemplateRef for template references
pub struct PreferUseTemplateRef;

impl ScriptRule for PreferUseTemplateRef {
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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = PreferUseTemplateRefVisitor {
            source,
            offset,
            result,
        };
        visitor.visit_program(program);
    }
}

struct PreferUseTemplateRefVisitor<'source, 'result> {
    source: &'source str,
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for PreferUseTemplateRefVisitor<'_, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_ref_call(it) && initialized_with_null(it) {
            let start = self.offset as u32 + it.span.start;
            let end = self.offset as u32 + it.span.end;

            if element_type_argument(it, self.source) {
                self.result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Use useTemplateRef() for DOM element references (Vue 3.5+)",
                        start,
                        end,
                    )
                    .with_help("Replace with: `const el = useTemplateRef<ElementType>('refName')`"),
                );
            } else if it.type_arguments.is_none() {
                self.result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Consider using useTemplateRef() for template references (Vue 3.5+)",
                        start,
                        end,
                    )
                    .with_help(
                        "If this is a template ref, use: `const el = useTemplateRef<ElementType>('refName')`. \
                         If this is a regular ref that starts as null, you can ignore this warning.",
                    ),
                );
            }
        }

        walk_call_expression(self, it);
    }
}

fn is_ref_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "ref"
    )
}

fn initialized_with_null(call: &CallExpression<'_>) -> bool {
    call.arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .is_some_and(is_null)
}

fn is_null(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::NullLiteral(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => is_null(&parenthesized.expression),
        _ => false,
    }
}

fn element_type_argument(call: &CallExpression<'_>, source: &str) -> bool {
    call.type_arguments.as_ref().is_some_and(|type_arguments| {
        let span = type_arguments.span;
        let start = span.start as usize;
        let end = span.end as usize;
        source.get(start..end).is_some_and(|type_source| {
            type_source.contains("Element")
                || type_source.contains("HTML")
                || type_source.contains("SVG")
        })
    })
}

#[cfg(test)]
mod tests {
    use super::PreferUseTemplateRef;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferUseTemplateRef));
        linter
    }

    #[test]
    fn test_valid_use_template_ref() {
        let linter = create_linter();
        let result = linter.lint("const input = useTemplateRef<HTMLInputElement>('input')", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_regular_ref() {
        let linter = create_linter();
        let result = linter.lint("const count = ref(0)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_ref_with_value() {
        let linter = create_linter();
        let result = linter.lint("const name = ref('hello')", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_suspicious_ref_null() {
        let linter = create_linter();
        let result = linter.lint("const el = ref(null)", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_suspicious_element_ref() {
        let linter = create_linter();
        let result = linter.lint("const input = ref<HTMLInputElement | null>(null)", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_toref_not_matched() {
        let linter = create_linter();
        let result = linter.lint("const x = toRef(state, 'count')", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_ref_null_string_not_matched() {
        let linter = create_linter();
        let result = linter.lint(r#"const text = "ref(null)""#, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_typed_ref_null_string_not_matched() {
        let linter = create_linter();
        let result = linter.lint(r#"const text = "ref<HTMLInputElement | null>(null)""#, 0);
        assert_eq!(result.warning_count, 0);
    }
}
