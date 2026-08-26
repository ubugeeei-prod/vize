//! vue/prefer-true-attribute-shorthand
//!
//! Prefer the shorthand for a boolean attribute bound to `true`.
//!
//! `:disabled="true"` on a native element with a boolean attribute is
//! equivalent to the shorthand `disabled`. The explicit `="true"` binding adds
//! noise without changing behaviour.
//!
//! The rule only reports attributes that are *statically known* to be boolean:
//! native boolean attributes on native elements. On a component the shorthand
//! passes the empty string `""`, which only casts to `true` when the target
//! component declares the prop as `Boolean` — a declaration this template
//! cannot see. For a fallthrough attr the rewrite silently changes runtime
//! behaviour (`true` becomes the falsy `""`), so components are left alone.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <input :disabled="true" />
//! ```
//!
//! ### Valid
//! ```vue
//! <input disabled />
//! <input :disabled="false" />
//! <MyComponent :visible="true" />
//! <MyComponent :visible="isVisible" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::html::helpers::BOOLEAN_ATTRIBUTES;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};
use vize_s0::is_native_tag;

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-true-attribute-shorthand",
    description: "Prefer the shorthand for a boolean attribute bound to `true`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Prefer the shorthand for a boolean attribute bound to `true`.
pub struct PreferTrueAttributeShorthand;

impl Rule for PreferTrueAttributeShorthand {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "bind" {
            return;
        }
        // Only a static argument (`:foo`), not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        if !arg.is_static {
            return;
        }
        let name = arg.content;
        // Only attributes statically known to be boolean: the shorthand form
        // passes `""`, which is only cast back to `true` for native boolean
        // attributes or props *declared* as `Boolean` on the target component.
        // Prop declarations of other components are not visible here, and for
        // fallthrough attrs the rewrite changes runtime behaviour (#4952), so
        // anything but a native boolean attribute stays silent.
        if !is_native_tag(element.tag) || !BOOLEAN_ATTRIBUTES.contains(&name) {
            return;
        }
        // Modifiers such as `.prop` change semantics; leave them alone.
        if !directive.modifiers.is_empty() {
            return;
        }
        let is_true =
            matches!(&directive.exp, Some(ExpressionNode::Simple(s)) if s.content.trim() == "true");
        if is_true {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "vue/prefer-true-attribute-shorthand.message",
                    &[("name", name)],
                ),
                &directive.loc,
                ctx.t("vue/prefer-true-attribute-shorthand.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreferTrueAttributeShorthand;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PreferTrueAttributeShorthand));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_native_boolean_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input :disabled="true" />"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_dynamic_arguments() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent :[prop]="true" v-bind:[other]="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_native_non_boolean_true_bindings() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :aria-hidden="true" :data-active="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_component_true_binding() {
        // #4952: the target component's prop declarations are not visible from
        // this template. When the prop is not Boolean-declared (e.g. a
        // fallthrough attr), the shorthand passes the empty string instead of
        // `true`, so the rewrite would change runtime behaviour.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Draggable :list="[]" item-key="id" :force-fallback="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent visible />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_false_and_dynamic() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent :visible="false" :open="isOpen" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
