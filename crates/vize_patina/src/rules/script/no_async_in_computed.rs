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
use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::no_side_effects_in_computed::{
    GetterBody, computed_getter_body, find_component_options, find_computed_object,
};

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
        if let Some(options) = find_component_options(program)
            && let Some(computed) = find_computed_object(options)
        {
            for property in &computed.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed {
                    continue;
                }
                let Some(getter) = computed_getter_body(property) else {
                    continue;
                };
                check_getter(getter, offset, result);
            }
        }
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
        } else if matches!(&it.callee, Expression::Identifier(identifier) if identifier.name.as_str() == "computed")
            && let Some(argument) = it
                .arguments
                .first()
                .and_then(|argument| argument.as_expression())
            && let Some(getter) = getter_body(argument)
        {
            check_getter(getter, self.offset, self.result);
        }

        walk_call_expression(self, it);
    }
}

fn getter_body<'a>(expression: &'a Expression<'a>) -> Option<GetterBody<'a>> {
    match expression.get_inner_expression() {
        Expression::FunctionExpression(function) => Some(GetterBody::Function(function)),
        Expression::ArrowFunctionExpression(arrow) => Some(GetterBody::Arrow(arrow)),
        _ => None,
    }
}

fn check_getter(getter: GetterBody<'_>, offset: usize, result: &mut ScriptLintResult) {
    let (is_async, span) = match getter {
        GetterBody::Function(function) => (function.r#async, function.span),
        GetterBody::Arrow(arrow) => (arrow.r#async, arrow.span),
    };
    if is_async {
        report(span, offset, result);
        return;
    }
    let mut visitor = AsyncActionVisitor { offset, result };
    match getter {
        GetterBody::Function(function) => {
            if let Some(body) = &function.body {
                visitor.visit_function_body(body);
            }
        }
        GetterBody::Arrow(arrow) => {
            for statement in &arrow.body.statements {
                visitor.visit_statement(statement);
            }
        }
    }
}

struct AsyncActionVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for AsyncActionVisitor<'_> {
    fn visit_function(&mut self, _it: &oxc_ast::ast::Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let asynchronous = match call.callee.get_inner_expression() {
            Expression::Identifier(identifier) => matches!(
                identifier.name.as_str(),
                "fetch" | "setInterval" | "setTimeout" | "queueMicrotask"
            ),
            Expression::StaticMemberExpression(member) => {
                matches!(
                    member.property.name.as_str(),
                    "$nextTick" | "then" | "catch" | "finally"
                )
            }
            Expression::ComputedMemberExpression(member) => matches!(
                member.static_property_name().as_deref(),
                Some("$nextTick" | "then" | "catch" | "finally")
            ),
            _ => false,
        };
        if asynchronous {
            report(call.span, self.offset, self.result);
        }
        walk_call_expression(self, call);
    }

    fn visit_await_expression(&mut self, expression: &oxc_ast::ast::AwaitExpression<'a>) {
        report(expression.span(), self.offset, self.result);
    }

    fn visit_new_expression(&mut self, expression: &oxc_ast::ast::NewExpression<'a>) {
        if matches!(&expression.callee, Expression::Identifier(identifier) if identifier.name == "Promise") {
            report(expression.span, self.offset, self.result);
        }
    }
}

fn report(span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            "Unexpected asynchronous action in a computed property.",
            start,
            end,
        )
        .with_help("Move asynchronous work to a watch handler or lifecycle hook."),
    );
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

    #[test]
    fn reports_next_tick_in_options_computed() {
        let source = r#"
export default {
  computed: {
    events() {
      this.$nextTick(this.refresh)
      return []
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            &source[diagnostic.start as usize..diagnostic.end as usize],
            "this.$nextTick(this.refresh)"
        );
    }
}
