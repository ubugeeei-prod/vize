//! vue/no-child-content
//!
//! Disallow element's child contents which would be overwritten by a directive
//! like `v-html` or `v-text`.
//!
//! When using `v-html` or `v-text`, the child content of the element is ignored
//! by Vue and replaced with the directive's value. Having child content alongside
//! these directives is misleading.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-html="content">child</div>
//! <div v-text="content">child</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-html="content"></div>
//! <div v-text="content"></div>
//! <div>child</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupContext, MarkupElement, MarkupNode, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::authored_closing;

static META: RuleMeta = RuleMeta {
    name: "vue/no-child-content",
    description: "Disallow child content when using v-html or v-text",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
/// Implements the `vue/no-child-content` lint rule.
pub struct NoChildContent;

impl NoChildContent {
    /// The first of `v-html` / `v-text` on the element ("html" or "text").
    fn has_v_html_or_v_text(element: &MarkupElement<'_>) -> Option<&'static str> {
        let mut found = None;
        element.walk_directives(&mut |directive| {
            if found.is_none() {
                found = match directive.name() {
                    "html" => Some("html"),
                    "text" => Some("text"),
                    _ => None,
                };
            }
        });
        found
    }

    fn has_child_content(element: &MarkupElement<'_>) -> bool {
        let mut found = false;
        element.walk_children(&mut |child| {
            found |= match child {
                MarkupNode::Text(text) => text.is_significant(),
                MarkupNode::Element(_)
                | MarkupNode::Comment(_)
                | MarkupNode::Interpolation(_)
                | MarkupNode::If(_)
                | MarkupNode::For(_) => true,
                MarkupNode::Other(_) => false,
            };
        });
        found
    }

    fn check(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if let Some(directive_name) = Self::has_v_html_or_v_text(element)
            && Self::has_child_content(element)
        {
            let open = element.range();
            let range = authored_closing::find(ctx.source, open.end, element.tag())
                .map_or(open, |closing| ByteRange::new(open.end, closing.start));
            ctx.error_at_with_help(
                ctx.t_fmt(
                    "vue/no-child-content.message",
                    &[("directive", directive_name)],
                ),
                range,
                ctx.t("vue/no-child-content.help"),
            );
        }
    }
}

impl MarkupRule for NoChildContent {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check(ctx.lint(), element);
    }
}

impl Rule for NoChildContent {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoChildContent;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoChildContent));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_html_no_children() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html="content"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_text_no_children() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text="content"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_directive() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>child content</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_html_with_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html="content">child</div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_text_with_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text="content">child</div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_with_element() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-html="content"><span>child</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_v_html_whitespace_only() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html="content">   </div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn reports_only_the_authored_child_content() {
        let source = "<div v-html=\"content\">\n  <span>child</span>\n</div>";
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.error_count, 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            &source[diagnostic.start as usize..diagnostic.end as usize],
            "\n  <span>child</span>\n"
        );
    }
}
