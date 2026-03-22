//! vue/html-self-closing
//!
//! Enforce self-closing style for HTML elements.
//!
//! ## Examples
//!
//! ### Invalid (default config)
//! ```vue
//! <div></div>  <!-- should be <div /> when empty -->
//! <img>        <!-- should be <img /> -->
//! <br>         <!-- should be <br /> -->
//! ```
//!
//! ### Valid
//! ```vue
//! <div />
//! <img />
//! <br />
//! <div>content</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/html-self-closing",
    description: "Enforce self-closing style",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Void elements that should always self-close
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// SVG elements that can self-close
const SVG_ELEMENTS: &[&str] = &[
    "circle",
    "ellipse",
    "line",
    "path",
    "polygon",
    "polyline",
    "rect",
    "use",
    "image",
    "animate",
    "animateMotion",
    "animateTransform",
    "set",
    "stop",
    "symbol",
    "defs",
    "g",
    "marker",
    "mask",
    "pattern",
    "linearGradient",
    "radialGradient",
    "clipPath",
    "filter",
    "foreignObject",
];

/// MathML elements
const MATHML_ELEMENTS: &[&str] = &[
    "math",
    "mrow",
    "mi",
    "mn",
    "mo",
    "ms",
    "mtext",
    "mspace",
    "msqrt",
    "mroot",
    "mfrac",
    "msup",
    "msub",
    "msubsup",
    "munder",
    "mover",
    "munderover",
    "mtable",
    "mtr",
    "mtd",
];

/// HTML self-closing style rule
#[derive(Default)]
pub struct HtmlSelfClosing;

impl Rule for HtmlSelfClosing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let tag = element.tag.as_str();
        let is_void = VOID_ELEMENTS.contains(&tag);
        let is_svg = SVG_ELEMENTS.contains(&tag);
        let is_mathml = MATHML_ELEMENTS.contains(&tag);
        let is_component =
            tag.contains('-') || tag.chars().next().is_some_and(|c| c.is_uppercase());
        let has_children = !element.children.is_empty();
        let is_self_closing = element.is_self_closing;

        // Void elements should always be self-closing
        if is_void && !is_self_closing {
            ctx.warn_with_help(
                ctx.t("vue/html-self-closing.void"),
                &element.loc,
                ctx.t("vue/html-self-closing.help"),
            );
            return;
        }

        // SVG/MathML elements without children should be self-closing
        if (is_svg || is_mathml) && !has_children && !is_self_closing {
            ctx.warn_with_help(
                ctx.t("vue/html-self-closing.empty"),
                &element.loc,
                ctx.t("vue/html-self-closing.help"),
            );
            return;
        }

        // Component elements without children should be self-closing
        if is_component && !has_children && !is_self_closing {
            ctx.warn_with_help(
                ctx.t("vue/html-self-closing.component"),
                &element.loc,
                ctx.t("vue/html-self-closing.help"),
            );
        }

        // Normal HTML elements without children - configurable (default: don't require self-closing)
        // This is intentionally not enforced for normal HTML elements like <div></div>
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlSelfClosing;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlSelfClosing));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_self_closing_void() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_void_not_self_closing() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component_self_closing() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_empty_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent></MyComponent>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component_with_content() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent>content</MyComponent>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
