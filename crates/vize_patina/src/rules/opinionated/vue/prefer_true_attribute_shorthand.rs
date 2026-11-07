//! vue/prefer-true-attribute-shorthand
//!
//! Prefer the shorthand for a boolean attribute bound to `true`.
//!
//! On a component, `:visible="true"` is more concisely written as `visible`.
//! Native HTML attributes are outside this Vue component-prop style rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent :visible="true" />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent visible />
//! <MyComponent :visible="false" />
//! <MyComponent :visible="isVisible" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{cstr, is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_relief::{DirectiveNode, ElementNode, ElementType, ExpressionNode, Namespace, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-true-attribute-shorthand",
    description: "Prefer the shorthand for a boolean attribute bound to `true`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Prefer the shorthand for a boolean attribute bound to `true`.
pub struct PreferTrueAttributeShorthand;

fn is_custom_component(ctx: &LintContext<'_>, element: &ElementNode<'_>) -> bool {
    let is_native_in_namespace = match element.ns {
        Namespace::Html => is_html_tag(element.tag),
        Namespace::Svg => {
            is_svg_tag(element.tag)
                && (element.tag == "svg"
                    || authored_inside_namespace(
                        ctx.source,
                        element.loc.span.start as usize,
                        "svg",
                    ))
        }
        Namespace::MathMl => is_math_ml_tag(element.tag),
    };
    element.tag_type == ElementType::Component
        || !is_native_in_namespace
        || element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attribute) => attribute.name == "is",
            PropNode::Directive(directive) => {
                directive.name == "is"
                    || (directive.name == "bind"
                        && matches!(
                            directive.arg.as_ref(),
                            Some(ExpressionNode::Simple(arg))
                                if arg.is_static && arg.content == "is"
                        ))
            }
        })
}

fn authored_inside_namespace(source: &str, offset: usize, tag: &str) -> bool {
    let Some(prefix) = source.get(..offset) else {
        return false;
    };
    let open_tag = cstr!("<{tag}");
    let close_tag = cstr!("</{tag}");
    let open = prefix.rmatch_indices(open_tag.as_str()).next();
    let close = prefix.rmatch_indices(close_tag.as_str()).next();
    open.is_some_and(|(open, _)| close.is_none_or(|(close, _)| open > close))
}

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
        // A bare attribute on <slot> is a string-valued slot prop, not a
        // component Boolean prop. Rewriting `:open="true"` to `open` would
        // therefore change the value exposed to the slot consumer.
        if element.tag_type == ElementType::Slot {
            return;
        }
        if !matches!(directive.name, "bind" | "model") {
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
        if !is_custom_component(ctx, element) {
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
    fn allows_native_boolean_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input :disabled="true" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_true_bindings_on_slot_outlets() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<slot :button="true" :open="true" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
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
    fn reports_component_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Draggable :list="[]" item-key="id" :force-fallback="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_lowercase_single_word_component_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<avatar :member="user" :avatar-only="true" :with-background="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn reports_platform_component_true_binding() {
        let result =
            create_linter().lint_template(r#"<view :catch-move="true" />"#, "Range.taro.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn allows_true_binding_on_view_inside_svg() {
        let result = create_linter().lint_template(
            r#"<svg><view :preserveAspectRatio="true" /></svg>"#,
            "Icon.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
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
