//! script/no-async-in-computed
//!
//! Disallow async functions in computed properties.
//! Computed properties must be synchronous and return a value immediately.
//! Using async functions or Promises in computed will cause unexpected behavior
//! since the computed will return a Promise object instead of the resolved value.
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

mod bindings;

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, MemberExpression, ObjectPropertyKind, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};

use bindings::ComputedBindings;

use super::no_side_effects_in_computed::{
    GetterBody, accessor_object_getter, computed_getter_body, find_component_options,
    find_computed_object,
};
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
        let computed_bindings = ComputedBindings::collect(program);
        let mut visitor = NoAsyncInComputedVisitor {
            offset,
            result,
            computed_bindings: &computed_bindings,
        };
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
    computed_bindings: &'result ComputedBindings,
}

impl<'a> Visit<'a> for NoAsyncInComputedVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.computed_bindings.resolves(&it.callee)
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
        } else if self.computed_bindings.resolves(&it.callee)
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
        Expression::ObjectExpression(object) => accessor_object_getter(object),
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
    let mut visitor = AsyncActionVisitor {
        offset,
        result,
        reported_receivers: Vec::new(),
    };
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
    reported_receivers: Vec<Span>,
}

impl<'a> Visit<'a> for AsyncActionVisitor<'_> {
    fn visit_function(
        &mut self,
        _it: &oxc_ast::ast::Function<'a>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
    }

    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let belongs_to_reported_chain = self
            .reported_receivers
            .iter()
            .any(|receiver| receiver.start <= call.span.start && call.span.end <= receiver.end);
        let report_call = is_async_call(call) && !belongs_to_reported_chain;
        if report_call {
            report(call.span, self.offset, self.result);
        }

        let receiver = report_call
            .then(|| async_member_receiver_span(call))
            .flatten();
        if let Some(receiver) = receiver {
            self.reported_receivers.push(receiver);
        }
        walk_call_expression(self, call);
        if receiver.is_some() {
            self.reported_receivers.pop();
        }
    }

    fn visit_await_expression(&mut self, expression: &oxc_ast::ast::AwaitExpression<'a>) {
        report(expression.span(), self.offset, self.result);
    }

    fn visit_new_expression(&mut self, expression: &oxc_ast::ast::NewExpression<'a>) {
        if matches!(&expression.callee, Expression::Identifier(identifier) if identifier.name == "Promise")
        {
            report(expression.span, self.offset, self.result);
        }
    }
}

fn async_member_receiver_span(call: &CallExpression<'_>) -> Option<Span> {
    member_expression(call.callee.get_inner_expression()).map(|member| member.object().span())
}

fn is_async_call(call: &CallExpression<'_>) -> bool {
    const TIMERS: &[&str] = &[
        "setInterval",
        "setTimeout",
        "requestAnimationFrame",
        "setImmediate",
    ];
    const PROMISE_STATICS: &[&str] = &[
        "all",
        "allSettled",
        "any",
        "race",
        "reject",
        "resolve",
        "try",
        "withResolvers",
    ];
    let callee = call.callee.get_inner_expression();
    match callee {
        Expression::Identifier(identifier) => {
            identifier.name == "fetch"
                || identifier.name == "queueMicrotask"
                || TIMERS.contains(&identifier.name.as_str())
        }
        _ => {
            let Some(member) = member_expression(callee) else {
                return false;
            };
            let Some(property) = member.static_property_name() else {
                return false;
            };
            matches!(property, "$nextTick" | "then" | "catch" | "finally")
                || (TIMERS.contains(&property)
                    && matches!(member.object().get_inner_expression(), Expression::Identifier(identifier) if identifier.name == "window"))
                || (PROMISE_STATICS.contains(&property)
                    && matches!(member.object().get_inner_expression(), Expression::Identifier(identifier) if identifier.name == "Promise"))
        }
    }
}

fn member_expression<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b MemberExpression<'a>> {
    match expression.without_parentheses() {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
        expression => expression.as_member_expression(),
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

    #[test]
    fn reports_next_tick_after_nested_callbacks_in_options_computed() {
        let source = r#"
export default {
  computed: {
    events () {
      let events = this.view.events.slice(0)
      events = events.filter(e => e.visible)
      events = events.filter(e => {
        const visible = e.start < this.end
        return visible
      })
      events.sort((a, b) => a.start < b.start ? -1 : 1)
      if (!this.cellSplits.length) this.$nextTick(this.refresh)
      return events
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn reports_object_getters_promise_statics_and_browser_timers() {
        let source = r#"
const all = computed(() => Promise.all(urls.map(fetch)).then((values) => values))
const value = computed({ get: () => fetch('/a'), set: () => {} })
const size = computed(() => {
  window.setTimeout(() => {}, 0)
  requestAnimationFrame(() => {})
  setImmediate(() => {})
  return 1
})
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 5, "{:?}", result.diagnostics);
    }

    #[test]
    fn resolves_vue_computed_imports_and_parenthesized_optional_calls() {
        let source = r#"
import * as Vue from "vue"
import { computed, computed as derive, ref } from "vue"
const pending = ref()
const aliased = derive(() => { setTimeout(() => {}, 0); return 1 })
const namespaced = Vue.computed(() => { setTimeout(() => {}, 0); return 1 })
const wrapped_then = computed(() => (pending.value?.then)?.((n) => n))
const wrapped_timer = computed(() => { (window?.setTimeout)?.(() => {}, 0); return 1 })
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 4, "{:?}", result.diagnostics);
    }

    #[test]
    fn resolves_default_vue_import_and_rejects_non_vue_computed_import() {
        let source = r#"
import Vue from "vue"
import computed from "./signals"
const from_vue = Vue.computed(() => { setTimeout(() => {}, 0); return 1 })
const unrelated = computed(() => { setTimeout(() => {}, 0); return 1 })
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    }
}
