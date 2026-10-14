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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use dynamic_component::{dynamic_component_may_render_component, is_dynamic_component};
use vize_l0::{is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_relief::{DirectiveNode, ElementNode, ElementType};

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

        if !is_component_like_tag(ctx, element) {
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

fn is_component_like_tag(ctx: &LintContext<'_>, element: &ElementNode<'_>) -> bool {
    // The dynamic-component element renders whatever its `is` prop resolves
    // to, so it must be classified by that prop instead of its tag name (and
    // before the `tag_type` check: parser options decide whether `<component>`
    // itself is typed `Element` or `Component`). A static `is` naming a known
    // native tag renders that native element, where v-text / v-html are as
    // safe as on the literal tag (#3211).
    if is_dynamic_component(element) {
        return dynamic_component_may_render_component(ctx, element);
    }

    if element.tag_type == ElementType::Component {
        return true;
    }

    // Like Vue's compiler and the reference rule, any tag outside the known
    // names of its namespace resolves as a component, including a
    // lowercase single word such as a mini-program `<view>`.
    // The namespace comes from the enclosing `<svg>` / `<math>` (the template
    // parser also tags a root-level SVG name such as `<view>` as SVG).
    let tag = element.tag;
    let inside = |root: &str| {
        ctx.has_ancestor(|parent| parent.tag.as_str() == root)
            && !ctx.has_ancestor(|parent| parent.tag.as_str() == "foreignObject")
    };
    if tag == "svg" || inside("svg") {
        !is_svg_tag(tag)
    } else if tag == "math" || inside("math") {
        !is_math_ml_tag(tag)
    } else {
        !is_html_tag(tag)
    }
}

#[cfg(test)]
mod tests;
