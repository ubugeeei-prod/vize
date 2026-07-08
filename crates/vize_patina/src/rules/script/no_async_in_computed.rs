//! script/no-async-in-computed
//!
//! Disallow async functions in computed properties.
//!
//! Computed properties must be synchronous and return a value immediately.
//! Using async functions or Promises in computed will cause unexpected behavior
//! since the computed will return a Promise object instead of the resolved value.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const data = computed(async () => {
//!   const response = await fetch('/api/data')
//!   return response.json()
//! })
//! ```
//!
//! ### Valid
//! ```ts
//! // Use ref + watch with cleanup for async operations
//! const data = ref(null)
//! watch(query, async (value, _oldValue, onCleanup) => {
//!   const controller = new AbortController()
//!   let active = true
//!   onCleanup(() => {
//!     active = false
//!     controller.abort()
//!   })
//!   const response = await fetch(`/api/data?q=${value}`, { signal: controller.signal })
//!   const next = await response.json()
//!   if (active) data.value = next
//! })
//!
//! // Or use a dedicated async state library
//! const { data } = useAsyncData(() => fetch('/api/data'))
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-async-in-computed",
    description: "Disallow async functions in computed properties",
    default_severity: Severity::Error,
};

/// Disallow async in computed
pub struct NoAsyncInComputed;

impl ScriptRule for NoAsyncInComputed {
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
        let mut visitor = NoAsyncInComputedVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct NoAsyncInComputedVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoAsyncInComputedVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if matches!(&it.callee, Expression::Identifier(identifier) if identifier.name.as_str() == "computed")
            && let Some(argument) = it
                .arguments
                .first()
                .and_then(|argument| argument.as_expression())
            && let Some(span) = async_function_span(argument)
        {
            let start = self.offset as u32 + it.span.start;
            let end = self.offset as u32 + span.start + 5;
            self.result.add_diagnostic(
                LintDiagnostic::error(
                    META.name,
                    "Computed properties cannot be async. They must return a value synchronously.",
                    start,
                    end,
                )
                .with_help(
                    "Use ref with watch() and cleanup for async operations; \
                     abort or ignore stale work before assigning the result.",
                ),
            );
        }

        walk_call_expression(self, it);
    }
}

fn async_function_span(expression: &Expression<'_>) -> Option<Span> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) if arrow.r#async => Some(arrow.span),
        Expression::FunctionExpression(function) if function.r#async => Some(function.span),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::NoAsyncInComputed;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoAsyncInComputed));
        linter
    }

    #[test]
    fn test_valid_sync_computed() {
        let linter = create_linter();
        let result = linter.lint("const doubled = computed(() => count.value * 2)", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_async_arrow_computed() {
        let linter = create_linter();
        let result = linter.lint("const data = computed(async () => await fetch('/api'))", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_async_function_computed() {
        let linter = create_linter();
        let result = linter.lint(
            "const data = computed(async function() { return await fetch('/api') })",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_async_watch_with_cleanup() {
        let linter = create_linter();
        let result = linter.lint(
            r#"watch(source, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController()
  let active = true
  onCleanup(() => {
    active = false
    controller.abort()
  })
  const next = await fetchData(value, { signal: controller.signal })
  if (active) data.value = next
})"#,
            0,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_string_literal_false_positive() {
        let linter = create_linter();
        let result = linter.lint("const source = 'computed(async () => value)'", 0);
        assert_eq!(result.error_count, 0);
    }
}
