//! vue/no-deprecated-html-element-is
//!
//! Disallow the `is` attribute on native HTML elements (removed in Vue 3).
//!
//! Vue 2 let you swap a native element for a component by writing
//! `<div is="MyComponent">` (component substitution). Vue 3 removed that
//! behaviour: `is` on a native element is now interpreted as the standard DOM
//! attribute, so the component is never mounted. To opt back into the legacy
//! "customized built-in element" lookup you must prefix the value with `vue:`
//! (`<div is="vue:MyComponent">`); for true dynamic components use
//! `<component :is="...">`.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-html-element-is`. It is
//! an opt-in migration rule and only fires for the default Vue 3 dialect.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <template>
//!   <div is="MyComponent" />
//! </template>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <template>
//!   <component :is="MyComponent" />
//!   <div is="vue:MyComponent" />
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupElementKind, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::is_html_tag;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-html-element-is",
    description: "Disallow the `is` attribute on native HTML elements",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

mod native;

/// Disallow the `is` attribute on native HTML elements.
pub struct NoDeprecatedHtmlElementIs;

impl NoDeprecatedHtmlElementIs {
    fn is_native_html_element(element: &MarkupElement<'_>) -> bool {
        element.kind() == MarkupElementKind::Element
            && element.tag() != "component"
            && is_html_tag(element.tag())
    }

    fn report(ctx: &mut LintContext<'_>, range: ByteRange) {
        ctx.error_at_with_help(
            ctx.t("vue/no-deprecated-html-element-is.message"),
            range,
            ctx.t("vue/no-deprecated-html-element-is.help"),
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if !Self::is_native_html_element(element) {
            return;
        }

        element.walk_attributes(&mut |attribute| {
            if attribute.name() != "is" {
                return;
            }
            // `is="vue:Foo"` opts into the customized built-in lookup and is
            // still supported in Vue 3, so it must not be flagged.
            if attribute.value().is_some_and(is_vue_prefixed_value)
                || authored_is_vue_prefixed(ctx.source, attribute.range())
            {
                return;
            }
            Self::report(ctx, attribute.range());
        });
    }

    fn check_directive(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        directive: &MarkupDirective<'_>,
    ) {
        if Self::is_native_html_element(element)
            && directive.name_eq("bind")
            && directive.is_static_arg_exact("is")
        {
            Self::report(ctx, directive.range());
        }
    }
}

impl MarkupRule for NoDeprecatedHtmlElementIs {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check_directive(ctx.lint(), element, directive);
    }
}

impl Rule for NoDeprecatedHtmlElementIs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_native_template_rule(&self) -> Option<&dyn crate::native::template::NativeTemplateRule> {
        Some(self)
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check_directive(
            ctx,
            &MarkupElement::new(element),
            &MarkupDirective::from_relief(directive),
        );
    }
}

fn is_vue_prefixed_value(value: &str) -> bool {
    value.starts_with("vue:")
        || value.starts_with("v&#117;e:")
        || value.starts_with("v&#x75;e:")
        || value.starts_with("v&#X75;e:")
}

fn authored_is_vue_prefixed(source: &str, range: ByteRange) -> bool {
    // The HTML parser decodes character references before producing attribute
    // locations, so the end offset can point into the authored value. Find the
    // nearest `is` attribute before the reported location instead of slicing
    // with the decoded end offset.
    let search_end = (range.start as usize + 4).min(source.len());
    let Some(attribute_start) = source
        .get(..search_end)
        .and_then(|prefix| prefix.rfind(" is="))
    else {
        return false;
    };
    let Some(attribute) = source.get(attribute_start + 1..) else {
        return false;
    };
    let value = attribute
        .split_once('=')
        .map_or(attribute, |(_, value)| value);
    let value = value.trim_start().trim_matches(['\'', '"']);
    is_vue_prefixed_value(value)
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedHtmlElementIs;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedHtmlElementIs));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_is_on_native_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div is="MyComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_is_on_native_element_with_children() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<table><tr is="my-row"></tr></table>"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_component_element_with_static_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component is="MyComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_component_element_with_bind_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component :is="currentComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_vue_prefixed_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div is="vue:MyComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn reports_dynamic_is_binding_on_native_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :is="something" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_dynamic_is_binding_in_vue2_authored_projects() {
        let result = create_linter().lint_sfc(
            r#"<template><div :is="component" /></template><script>export default {}</script>"#,
            "Legacy.vue",
        );
        assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn ignores_components() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent is="Other" /><maybe-v-node :is="content" />"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_native_element_without_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="x" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }
}
