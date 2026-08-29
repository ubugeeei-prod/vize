//! html/no-consecutive-br
//!
//! Warn against consecutive `<br>` elements. Multiple `<br>` tags typically
//! indicate that `<p>` elements or CSS margins should be used instead.
//! Based on markuplint's `no-consecutive-br` rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <p>First paragraph<br><br>Second paragraph</p>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <p>First paragraph</p>
//!   <p>Second paragraph</p>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupNode, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "html/no-consecutive-br",
    description: "Disallow consecutive <br> elements",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoConsecutiveBr;

impl NoConsecutiveBr {
    fn check_markup_children<'a>(ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.is_component() {
            return;
        }

        let mut last_br_seen = false;

        element.walk_children(&mut |child| match child {
            MarkupNode::Element(child) if child.is_unqualified_tag_exact("br") => {
                if last_br_seen {
                    let message = ctx.lint().t("html/no-consecutive-br.message");
                    let help = ctx.lint().t("html/no-consecutive-br.help");
                    ctx.lint().warn_at_with_help(message, child.range(), help);
                }
                last_br_seen = true;
            }
            MarkupNode::Text(text) => {
                if text.is_significant() {
                    last_br_seen = false;
                }
            }
            MarkupNode::Comment(_) => {}
            _ => {
                last_br_seen = false;
            }
        });
    }
}

impl MarkupRule for NoConsecutiveBr {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_markup_children(ctx, element);
    }
}

impl Rule for NoConsecutiveBr {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        let mut last_br_seen = false;

        for child in &element.children {
            match child {
                TemplateChildNode::Element(el) if el.tag == "br" => {
                    if last_br_seen {
                        let message = ctx.t("html/no-consecutive-br.message");
                        let help = ctx.t("html/no-consecutive-br.help");
                        ctx.warn_with_help(message, &el.loc, help);
                    }
                    last_br_seen = true;
                }
                TemplateChildNode::Text(text) => {
                    // Only whitespace text between <br> tags doesn't reset
                    if !text.content.trim().is_empty() {
                        last_br_seen = false;
                    }
                }
                TemplateChildNode::Comment(_) => {
                    // Comments don't reset
                }
                _ => {
                    last_br_seen = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoConsecutiveBr;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoConsecutiveBr));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_single_br() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>line1<br>line2</p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_br_separated_by_content() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>line1<br>middle<br>line2</p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_br() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>text</p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_consecutive_br() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>text<br><br>more</p>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_consecutive_br_with_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>text<br> <br>more</p>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_three_consecutive_br() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>text<br><br><br>more</p>"#, "test.vue");
        // Second and third <br> both trigger
        assert_eq!(result.warning_count, 2);
    }
}
