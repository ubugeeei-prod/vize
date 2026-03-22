//! vapor/no-vue-lifecycle-events
//!
//! Disallow @vue:xxx per-element lifecycle events in Vapor mode.
//!
//! Per-element lifecycle events like @vue:mounted, @vue:updated, etc.
//! are NOT supported in Vapor mode. Use watchEffect or onMounted from
//! setup instead.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div @vue:mounted="handler"></div>
//! <div @vue:updated="handler"></div>
//! <div @vue:beforeUnmount="handler"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <!-- Use lifecycle hooks in script setup instead -->
//! <script setup vapor>
//! import { onMounted, onUpdated } from 'vue'
//! onMounted(() => { /* ... */ })
//! </script>
//! <div ref="el"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vapor/no-vue-lifecycle-events",
    description: "Disallow @vue:xxx per-element lifecycle events (not supported in Vapor)",
    category: RuleCategory::Vapor,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow @vue:xxx lifecycle events
pub struct NoVueLifecycleEvents;

impl Rule for NoVueLifecycleEvents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "on" {
            return;
        }

        let event_name = match &directive.arg {
            Some(ExpressionNode::Simple(s)) => s.content.as_str(),
            _ => return,
        };

        if let Some(lifecycle_name) = event_name.strip_prefix("vue:") {
            ctx.error_with_help(
                ctx.t_fmt(
                    "vapor/no-vue-lifecycle-events.message",
                    &[("event", lifecycle_name)],
                ),
                &directive.loc,
                ctx.t("vapor/no-vue-lifecycle-events.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoVueLifecycleEvents;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoVueLifecycleEvents));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_vue_mounted() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @vue:mounted="onMounted"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("vue:mounted"));
    }

    #[test]
    fn test_invalid_vue_updated() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @vue:updated="onUpdated"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("vue:updated"));
    }

    #[test]
    fn test_valid_regular_event() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="handleClick"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_custom_event() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent @custom-event="handler"></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }
}
