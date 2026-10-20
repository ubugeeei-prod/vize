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
        let receivers = receiver::Receivers::new(program);
        let mut visitor = NoDeprecatedEventsApiVisitor {
            offset,
            receivers,
            result,
        };
        visitor.visit_program(program);
    }
}

struct NoDeprecatedEventsApiVisitor<'a, 'result> {
    offset: usize,
    receivers: receiver::Receivers<'a>,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoDeprecatedEventsApiVisitor<'a, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some((span, method, receiver)) = removed_event_callee(&it.callee)
            && self.receivers.reports(method, receiver)
        {
            self.push_diagnostic(span, method);
        }

        walk_call_expression(self, it);
    }
}

impl NoDeprecatedEventsApiVisitor<'_, '_> {
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
fn removed_event_callee<'b, 'a>(
    expression: &'b Expression<'a>,
) -> Option<(Span, &'static str, &'b Expression<'a>)> {
    // TypeScript wrappers erase at runtime. Preserve the historical warning
    // policy even when the pinned ESLint selector misses wrapped callees.
    let member = match expression {
        Expression::ParenthesizedExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression);
        }
        Expression::TSAsExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression);
        }
        Expression::TSSatisfiesExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression);
        }
        Expression::TSNonNullExpression(wrapper) => {
            return removed_event_callee(&wrapper.expression);
        }
        Expression::ChainExpression(chain) => chain.expression.as_member_expression()?,
        _ => expression.as_member_expression()?,
    };
    let (span, property) = member.static_property_info()?;
    let matched = REMOVED_EVENT_METHODS
        .into_iter()
        .find(|candidate| *candidate == property)?;
    Some((span, matched, member.object()))
}

// Only receivers proven to be a Vue instance are reported; see `receiver`.
mod receiver;

#[cfg(test)]
mod tests {
    use super::{NoDeprecatedEventsApi, ScriptLintResult, ScriptRule};

    fn lint(source: &str) -> ScriptLintResult {
        let rule = NoDeprecatedEventsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        result
    }

    fn count(source: &str) -> usize {
        lint(source).error_count
    }

    #[test]
    fn test_valid_emit_is_not_flagged() {
        // $emit remains valid in Vue 3 and must NOT be reported.
        assert_eq!(count("this.$emit('change', payload)"), 0);
    }

    #[test]
    fn test_valid_external_emitter_methods() {
        // Plain `.on` / `.off` / `.once` (no `$`) belong to external emitters.
        let source = "emitter.on('event', h)\nemitter.off('event', h)\nemitter.once('event', h)";
        assert_eq!(count(source), 0);
    }

    #[test]
    fn test_invalid_this_calls() {
        let source = "this.$on('a', h1)\nthis.$off('a', h1)\nthis.$once('b', h2)\nthis.$emit('c')";
        assert_eq!(count(source), 3);
    }

    #[test]
    fn test_valid_dynamic_member_not_flagged() {
        // An unknown computed key cannot prove a removed event API. Known
        // string/template-literal keys are covered by the authored corpus.
        assert_eq!(count("this[methodName]('event', handler)"), 0);
    }

    #[test]
    fn test_invalid_optional_instance_calls() {
        assert_eq!(
            count("this.$on?.('x', h); this?.$off('x', h); this?.$once?.('x', h)"),
            3
        );
        assert_eq!(count("emitter.$on?.('x', h); this.$root.$off?.('x', h)"), 1);
    }

    #[test]
    fn test_invalid_wrapped_callee() {
        for source in [
            "(this.$on as any)('event', handler)",
            "(this.$off satisfies Function)('event', handler)",
            "this.$once!('event', handler)",
            "((this.$on as any)!)('event', handler)",
            "((this.$root as any).$parent!).$on('event', handler)",
        ] {
            assert_eq!(count(source), 1, "{source}");
        }
    }

    #[test]
    fn test_invalid_proven_instances() {
        for source in [
            "const vm = this; vm.$on('x', h)",
            "let vm = this; const self = vm; self.$parent.$root.$off('x', h)",
            "import Vue from 'vue'; const bus = new Vue(); bus.$on('x', h)",
            "const bus = new Vue(); bus.$once('x', h)",
            "import Vue from 'vue'; const C = Vue.extend({}); const vm = new C(); vm.el = 1; vm.$on('x', h)",
            "import Vue from 'vue'; let vm = new (Vue.extend({}))().$mount(); vm.$off('x', h)",
        ] {
            assert_eq!(count(source), 1, "{source}");
        }
    }

    /// Reduced #9217 receivers whose identity a single file cannot prove.
    #[test]
    fn test_valid_unproven_receivers() {
        for source in [
            // cube-ui select.vue: a caller-provided Function prop may return any emitter.
            "export default { props: { action: Function }, methods: { show() {\
             const component = this.actionComponent = this.action ? this.action(this) : this.defaultAction(this);\
             component.$on('select', this.onSelect); component.$on('hide', this.onHide);\
             component.$on('cancel', this.onHide); component.$on('close', this.onHide) } } }",
            // gogocode Comp.vue: the imported hub may be the Vue 3 migration shim.
            "import eventHub from './EventHub'; export default {\
             mounted() { eventHub.$on('inc', h); eventHub.$once('inc', h) },\
             beforeUnmount() { eventHub.$off('inc') } }",
            // gogocode B.vue line 51: `eventHub` is local to the sibling hook.
            "export default { mounted() { let eventHub = {} }, beforeUnmount() { eventHub.$off('inc') } }",
            "import Vue from 'vue'; export default { inject: ['eventBus'], mounted() { this.eventBus.$on('x', h); this.$bus.$off('x', h) } }",
            "function make() { return new Vue() } make().$on('x', h)",
            "import { Vue } from 'other'; new Vue().$on('x', h)",
            "const Vue = shim; new Vue().$on('x', h)",
            "export default { mounted() { this.$refs.child.$on('x', h); this.$root.foo.$on('x', h) } }",
        ] {
            assert_eq!(count(source), 0, "{source}");
        }
    }

    #[test]
    fn test_valid_escaped_or_reassigned_instances() {
        for source in [
            "const bus = new Vue(); install(bus); bus.$on('x', h)",
            "const bus = new Vue(); export default bus; bus.$on('x', h)",
            "let bus = new Vue(); bus = shim; bus.$on('x', h)",
            "const bus = new Vue(); bus[key] = shim; bus.$on('x', h)",
            "const bus = new Vue(); bus.__proto__ = shim; bus.$on('x', h)",
            "const { bus } = { bus: new Vue() }; bus.$on('x', h)",
        ] {
            assert_eq!(count(source), 0, "{source}");
        }
    }

    /// gogocode B.vue line 46 and the ways a file can install a foreign method.
    #[test]
    fn test_valid_authored_event_method_definitions() {
        for source in [
            "import tiny from 'tiny-emitter/instance'; export default { mounted() { let eventHub = {};\
             const override = { $on: (...a) => tiny.on(...a), $off: (...a) => tiny.off(...a) };\
             Object.assign(eventHub, override); eventHub.$on('inc', h) } }",
            "const O = Object; O.assign(this, { $on: shim }); this.$on('x', h)",
            "Object.defineProperty(this, '$on', { value: shim }); this.$on('x', h)",
            "Object.prototype.$on = shim; this.$on('x', h)",
            "Object.defineProperty(Object.prototype, `$on`, { set(v) {} }); this.$on('x', h)",
            "const bus = new Vue({ methods: { $on() {} } }); bus.$on('x', h)",
            "class Hub { $on() {} once() { this.$on('x', h) } }",
            "const hub = { set $on(v) {} }; this.$on('x', h)",
            "export default { created() { extend(this, options); this.$on('x', h) } }",
            "const vm = this; merge(vm, mixin); vm.$on('x', h)",
        ] {
            assert_eq!(count(source), 0, "{source}");
        }
        // Only the defined name is shadowed; a named key leaves the rest alone.
        assert_eq!(
            count("const hub = { $on() {} }; this.$on('x', h); this.$off('x', h)"),
            1
        );
        assert_eq!(
            count("Object.defineProperty(this, 'initial', {}); arr.push(this); this.$on('x', h)"),
            1
        );
        assert_eq!(count("this.$emit('done', this); this.$on('x', h)"), 1);
    }
}
