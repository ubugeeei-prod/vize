//! vue/no-v-text-v-html-on-component
//!
//! Disallow v-text / v-html on component elements.
//!
//! Using `v-text` or `v-html` on components is an error because components
//! have their own rendering logic and these directives would overwrite
//! the component's content in an unexpected way.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent v-html="content" />
//! <MyComponent v-text="content" />
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-html="content"></div>
//! <component is="div" v-html="content" />
//! <MyComponent>{{ content }}</MyComponent>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode};
use vize_s0::{is_html_tag, is_native_tag};

static META: RuleMeta = RuleMeta {
    name: "vue/no-v-text-v-html-on-component",
    description: "Disallow v-text / v-html on component elements",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
pub struct NoVTextVHtmlOnComponent;

impl Rule for NoVTextVHtmlOnComponent {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "text" && directive.name != "html" {
            return;
        }

        if !is_component_like_tag(element) {
            return;
        }

        ctx.error_with_help(
            ctx.t_fmt(
                "vue/no-v-text-v-html-on-component.message",
                &[("directive", directive.name), ("tag", element.tag)],
            ),
            &directive.loc,
            ctx.t("vue/no-v-text-v-html-on-component.help"),
        );
    }
}

fn is_component_like_tag(element: &ElementNode<'_>) -> bool {
    // The dynamic-component element renders whatever its `is` prop resolves
    // to, so it must be classified by that prop instead of its tag name (and
    // before the `tag_type` check: parser options decide whether `<component>`
    // itself is typed `Element` or `Component`). A static `is` naming a known
    // native tag renders that native element, where v-text / v-html are as
    // safe as on the literal tag (#3211).
    if is_dynamic_component(element) {
        return !resolves_to_static_native_tag(element);
    }

    if element.tag_type == ElementType::Component {
        return true;
    }

    let tag = element.tag;
    tag.contains('-') && !is_html_tag(tag)
}

/// Whether the element compiles through `resolveDynamicComponent`.
///
/// Mirrors the compiler's `is_dynamic_component` helper in `vize_atelier_core`:
/// lowercase `<component>` is always Vue's reserved dynamic-component element,
/// while `<Component>` is only dynamic when it actually carries an `is` prop —
/// without one it is a user component named `Component` and stays subject to
/// the plain component classification.
fn is_dynamic_component(element: &ElementNode<'_>) -> bool {
    element.tag == "component"
        || (element.tag == "Component" && element.props.iter().any(is_is_prop))
}

/// Whether a prop is the `is` attribute or a `:is` binding.
fn is_is_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => is_bind_is(dir),
    }
}

/// Whether a directive is a static-argument `:is` / `v-bind:is` binding.
fn is_bind_is(dir: &DirectiveNode<'_>) -> bool {
    dir.name == "bind"
        && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "is")
}

/// Whether a dynamic component statically resolves to a native element.
///
/// `resolveDynamicComponent` looks the string up in the component registry
/// first and only falls back to rendering the literal tag, so only values
/// Vue could never register — known native HTML/SVG/MathML tags — are
/// accepted. Anything else may still produce a component and stays flagged:
/// a `:is` binding (which wins over a static `is` attribute in codegen, in
/// any prop order), a missing/valueless `is`, or a non-native value such as
/// `is="MyComponent"`. The `vue:` prefix is deliberately NOT stripped: Vue
/// honours it only on literal native tags (`<button is="vue:x">`), while on
/// the dynamic-component element the raw value reaches
/// `resolveDynamicComponent` verbatim, so `is="vue:div"` does not render a
/// native `<div>`.
fn resolves_to_static_native_tag(element: &ElementNode<'_>) -> bool {
    let mut static_is: Option<&str> = None;
    for prop in element.props.iter() {
        match prop {
            PropNode::Directive(dir) if is_bind_is(dir) => return false,
            // Duplicate `is` attributes: codegen keeps the first.
            PropNode::Attribute(attr) if attr.name == "is" && static_is.is_none() => {
                static_is = attr.value.as_ref().map(|v| v.content);
            }
            _ => {}
        }
    }
    static_is.is_some_and(is_native_tag)
}

#[cfg(test)]
mod tests {
    use super::NoVTextVHtmlOnComponent;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoVTextVHtmlOnComponent));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_html_on_div() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html="content"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_text_on_span() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span v-text="content"></span>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_html_on_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_on_kebab_case_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-component v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_on_dynamic_component() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<component :is="tagName" v-html="content" />"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_text_on_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent v-text="content" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_component_with_slot_content() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent>{{ content }}</MyComponent>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    /// Exact reproduction of #3211: a static `is="div"` resolves to a native
    /// `<div>`, so `v-html` must be allowed exactly as on a literal `<div>`.
    #[test]
    fn test_valid_v_html_on_component_with_static_native_is_sfc() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<script setup lang="ts">
defineProps<{ html: string }>();
</script>

<template>
  <component is="div" v-html="html" />
</template>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_text_on_component_with_static_is_span() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<component is="span" v-text="content" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_html_on_component_with_static_is_svg_tag() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component is="svg" v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_html_on_uppercase_component_tag_with_static_native_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Component is="div" v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_html_on_component_with_static_component_is() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<component is="MyComponent" v-html="content" />"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_on_component_with_unknown_is() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<component is="unknown-thing" v-html="content" />"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_on_component_without_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    /// Vue strips the `vue:` prefix only on literal native tags; on
    /// `<component>` the value reaches `resolveDynamicComponent` verbatim and
    /// does not render a native `<div>`, so the directive stays flagged.
    #[test]
    fn test_invalid_v_html_on_component_with_vue_prefixed_is() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<component is="vue:div" v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    /// A `:is` binding wins over a static `is` attribute in codegen, so the
    /// element may still resolve to a component at runtime.
    #[test]
    fn test_invalid_v_html_when_bind_is_shadows_static_native_is() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<component is="div" :is="tagName" v-html="content" />"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_on_uppercase_component_tag_without_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Component v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    /// On a literal native element `is` means customized built-in elements,
    /// not component substitution — behavior there must not change.
    #[test]
    fn test_valid_v_html_on_native_element_with_is_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div is="my-thing" v-html="content" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
