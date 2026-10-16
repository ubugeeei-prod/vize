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
//! Built-in Vue 3 presets enable this migration rule; Vue 2 presets leave it off.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script>
//! export default {
//!   mounted() {
//!     this.$on('event', handler)
//!     const vm = this
//!     vm.$once('event', handler)
//!     this.$off('event', handler)
//!   }
//! }
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script>
//! import mitt from 'mitt'
//! const emitter = mitt()
//! export default {
//!   mounted() {
//!     this.$emit('event', payload)
//!     emitter.on('event', handler)
//!   }
//! }
//! </script>
//! ```

use super::vue_instance::VueInstance;
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;
use vize_l0::cstr;

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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !(source.contains("$on")
            || source.contains("$off")
            || source.contains("$once")
            || source.contains('\\'))
        {
            return;
        }
        let instance = VueInstance::new(program, source);
        let mut visitor = NoDeprecatedEventsApiVisitor {
            offset,
            result,
            instance: &instance,
        };
        visitor.visit_program(program);
    }
}

struct NoDeprecatedEventsApiVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    instance: &'result VueInstance,
}

impl<'a> Visit<'a> for NoDeprecatedEventsApiVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.instance.contains(it.span)
            && let Some((span, method)) = removed_event_callee(&it.callee, self.instance)
        {
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

/// Preserve the provider's property-token span, including computed literal keys.
fn removed_event_callee(
    expression: &Expression<'_>,
    instance: &VueInstance,
) -> Option<(Span, &'static str)> {
    // TypeScript wrappers erase at runtime. Preserve warnings on a proven Vue
    // instance even though the pinned ESLint selector misses wrapped callees.
    let member = match expression {
        Expression::ParenthesizedExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression, instance);
        }
        Expression::TSAsExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression, instance);
        }
        Expression::TSSatisfiesExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression, instance);
        }
        Expression::TSNonNullExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression, instance);
        }
        Expression::ChainExpression(chain) => chain.expression.as_member_expression()?,
        _ => expression.as_member_expression()?,
    };
    if !instance.is_this(member.object()) {
        return None;
    }
    let (span, property) = member.static_property_info()?;
    let matched = REMOVED_EVENT_METHODS
        .into_iter()
        .find(|candidate| *candidate == property)?;
    Some((span, matched))
}

#[cfg(test)]
mod tests {
    use super::{NoDeprecatedEventsApi, ScriptLintResult, ScriptRule};

    fn lint(source: &str) -> ScriptLintResult {
        let rule = NoDeprecatedEventsApi;
        let mut result = ScriptLintResult::default();
        let source = format!("export default {{ created() {{ {source} }} }}");
        rule.check(&source, 0, &mut result);
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
    fn test_valid_foreign_emitter_off() {
        let result = lint("emitter.$off('event', handler)");
        assert_eq!(result.error_count, 0);
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
    fn test_valid_dynamic_member_not_flagged() {
        // An unknown computed key cannot prove a removed event API. Known
        // string/template-literal keys are covered by the authored corpus.
        let result = lint("this[methodName]('event', handler)");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_optional_instance_calls() {
        let result = lint("this.$on?.('x', h); this?.$off('x', h); this?.$once?.('x', h)");
        assert_eq!(result.error_count, 3);
        let foreign = lint("emitter.$on?.('x', h); this.$root.$off?.('x', h)");
        assert_eq!(foreign.error_count, 0);
    }

    #[test]
    fn test_invalid_wrapped_callee() {
        for source in [
            "(this.$on as any)('event', handler)",
            "(this.$off satisfies Function)('event', handler)",
            "this.$once!('event', handler)",
            "((this.$on as any)!)('event', handler)",
        ] {
            let result = lint(source);
            assert_eq!(result.error_count, 1, "{source}");
        }
    }
}
