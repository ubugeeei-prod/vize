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

mod dynamic_component;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use dynamic_component::{dynamic_component_may_render_component, is_dynamic_component};
use vize_l0::is_native_tag;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-v-text-v-html-on-component",
    description: "Disallow v-text / v-html on component elements",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
/// Implements the `vue/no-v-text-v-html-on-component` lint rule.
pub struct NoVTextVHtmlOnComponent;

impl NoVTextVHtmlOnComponent {
    fn check(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        directive: &MarkupDirective<'_>,
    ) {
        if !matches!(directive.name(), "text" | "html") || !is_component_like_tag(ctx, element) {
            return;
        }

        ctx.error_at_with_help(
            ctx.t_fmt(
                "vue/no-v-text-v-html-on-component.message",
                &[("directive", directive.name()), ("tag", element.tag())],
            ),
            directive.range(),
            ctx.t("vue/no-v-text-v-html-on-component.help"),
        );
    }
}

impl MarkupRule for NoVTextVHtmlOnComponent {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check(ctx.lint(), element, directive);
    }
}

impl Rule for NoVTextVHtmlOnComponent {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check(
            ctx,
            &MarkupElement::new(element),
            &MarkupDirective::from_relief(directive),
        );
    }
}

fn is_component_like_tag(ctx: &LintContext<'_>, element: &MarkupElement<'_>) -> bool {
    // The dynamic-component element renders whatever its `is` prop resolves
    // to, so it must be classified by that prop instead of its tag name (and
    // before the `tag_type` check: parser options decide whether `<component>`
    // itself is typed `Element` or `Component`). A static `is` naming a known
    // native tag renders that native element, where v-text / v-html are as
    // safe as on the literal tag (#3211).
    if is_dynamic_component(element) {
        return dynamic_component_may_render_component(ctx, element);
    }

    // The lint parser can retain a native tag type for Vue's explicit native
    // `is="vue:..."` component substitution. Preserve that component boundary.
    if element.is_component()
        || element
            .static_attribute("is")
            .and_then(|attribute| attribute.value())
            .is_some_and(|value| value.starts_with("vue:"))
    {
        return true;
    }

    // Vue recognizes native HTML/SVG/MathML names independently of lexical
    // namespace. A root `<g>` or a chart's SVG slot receives its namespace at
    // runtime; it still compiles to an element, never a component lookup.
    // Unknown lowercase tags remain components, as do the explicit component
    // substitutions already classified above.
    !is_native_tag(element.tag())
}

#[cfg(test)]
mod tests;
