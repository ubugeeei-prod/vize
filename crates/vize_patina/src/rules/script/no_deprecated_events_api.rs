//! script/no-deprecated-events-api
//!
//! Disallow the removed Vue 2 instance event-emitter API.
//!
//! ## Rationale
//!
//! Vue 3 removed the `$on`, `$off`, and `$once` instance methods. Components
//! that used the instance as an event hub must migrate to an external emitter
//! library (such as `mitt` or `tiny-emitter`) or a different pattern. `$emit`
//! is still supported in Vue 3 and is **not** flagged by this rule.
//!
//! This is a Vue 2 -> 3 migration rule and is opt-in.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! this.$on('event', handler)
//! this.$once('event', handler)
//! this.$off('event', handler)
//! emitter.$off('event')
//! ```
//!
//! ### Valid
//! ```ts
//! // $emit is still valid in Vue 3
//! this.$emit('event', payload)
//!
//! // Use an external emitter instead
//! import mitt from 'mitt'
//! const emitter = mitt()
//! emitter.on('event', handler)
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};
use vize_carton::cstr;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-events-api",
    description: "Disallow the removed Vue 2 events API ($on / $off / $once)",
    default_severity: Severity::Error,
};

/// The instance event-emitter methods removed in Vue 3. `$emit` is intentionally
/// excluded because it remains valid in Vue 3.
const REMOVED_EVENT_METHODS: [&str; 3] = ["$on", "$off", "$once"];

/// Disallow the removed Vue 2 events API ($on / $off / $once)
pub struct NoDeprecatedEventsApi;

impl ScriptRule for NoDeprecatedEventsApi {
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
        let mut visitor = NoDeprecatedEventsApiVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct NoDeprecatedEventsApiVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoDeprecatedEventsApiVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some((span, method)) = removed_event_call(it) {
            self.push_diagnostic(span, method);
        }

        walk_call_expression(self, it);
    }
}

impl NoDeprecatedEventsApiVisitor<'_> {
    fn push_diagnostic(&mut self, span: Span, method: &str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result.add_diagnostic(
            LintDiagnostic::error(
                META.name,
                cstr!(
                    "`{method}()` was removed in Vue 3. The instance event-emitter API is no longer available."
                ),
                start,
                end,
            )
            .with_help(
                "Vue 3 removed $on / $off / $once. Use an external emitter library (such as mitt) or refactor to props/emits instead. Note: $emit is still valid.",
            ),
        );
    }
}

/// Returns the span and the matched method name when `call`'s callee is a member
/// access whose static property is one of the removed event methods.
fn removed_event_call<'a>(call: &'a CallExpression<'a>) -> Option<(Span, &'static str)> {
    removed_event_callee(&call.callee)
}

fn removed_event_callee<'a>(expression: &'a Expression<'a>) -> Option<(Span, &'static str)> {
    match expression {
        expression if expression.is_member_expression() => {
            let member = expression.as_member_expression()?;
            let property = member.static_property_name()?;
            let matched = REMOVED_EVENT_METHODS
                .into_iter()
                .find(|candidate| *candidate == property)?;
            Some((expression.span(), matched))
        }
        // Look through wrappers so `(this.$on)(...)`, `(this.$on as any)(...)`,
        // and `this.$on!(...)` are still recognized.
        Expression::ParenthesizedExpression(paren) => removed_event_callee(&paren.expression),
        Expression::TSAsExpression(ts_as) => removed_event_callee(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            removed_event_callee(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            removed_event_callee(&ts_non_null.expression)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{NoDeprecatedEventsApi, ScriptLintResult, ScriptRule};

    fn lint(source: &str) -> ScriptLintResult {
        let rule = NoDeprecatedEventsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        result
    }

    #[test]
    fn test_valid_emit_is_not_flagged() {
        // $emit remains valid in Vue 3 and must NOT be reported.
        let result = lint("this.$emit('change', payload)");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_external_emitter_methods() {
        // Plain `.on` / `.off` / `.once` (no `$`) belong to external emitters.
        let source = r#"
emitter.on('event', handler)
emitter.off('event', handler)
emitter.once('event', handler)
"#;
        let result = lint(source);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_this_on() {
        let result = lint("this.$on('event', handler)");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_this_once() {
        let result = lint("this.$once('event', handler)");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_emitter_off() {
        let result = lint("emitter.$off('event', handler)");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_multiple_in_one_block() {
        let source = r#"
this.$on('a', h1)
this.$off('a', h1)
this.$once('b', h2)
this.$emit('c')
"#;
        let result = lint(source);
        // Three removed-method calls, $emit excluded.
        assert_eq!(result.error_count, 3);
    }

    #[test]
    fn test_invalid_computed_member_not_flagged() {
        // Dynamic/computed member access (`this['$on']`) has no static property
        // name resolvable as an identifier, so it is not flagged. The common
        // migration smell is the static form, which is covered above.
        let result = lint("this[methodName]('event', handler)");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_wrapped_callee() {
        let source = "(this.$on as any)('event', handler)";
        let result = lint(source);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}
