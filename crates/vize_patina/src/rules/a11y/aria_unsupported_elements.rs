//! a11y/aria-unsupported-elements
//!
//! Disallow ARIA attributes and role on elements that do not support them.
//!
//! Certain HTML elements like `<meta>`, `<html>`, `<script>`, and `<style>`
//! do not support ARIA attributes or the `role` attribute because they are
//! not rendered visually or do not have semantic meaning in the accessibility tree.
//!
//! Based on eslint-plugin-jsx-a11y aria-unsupported-elements rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, ElementType, ExpressionNode, PropNode, SourceLocation};

use super::helpers;

static META: RuleMeta = RuleMeta {
    name: "a11y/aria-unsupported-elements",
    description: "Disallow ARIA attributes on elements that do not support them",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow ARIA attributes on elements that do not support them
#[derive(Default)]
pub struct AriaUnsupportedElements;

impl Rule for AriaUnsupportedElements {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        if !helpers::ARIA_UNSUPPORTED_ELEMENTS.contains(&element.tag.as_str()) {
            return;
        }

        for prop in &element.props {
            match prop {
                PropNode::Attribute(attr) if Self::is_aria_or_role(attr.name.as_str()) => {
                    self.report_unsupported_attr(ctx, element, attr.name.as_str(), &attr.loc);
                }
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && Self::is_aria_or_role(arg.content.as_str()) =>
                {
                    self.report_unsupported_attr(ctx, element, arg.content.as_str(), &dir.loc);
                }
                _ => {}
            }
        }
    }
}

impl AriaUnsupportedElements {
    fn is_aria_or_role(name: &str) -> bool {
        name.starts_with("aria-") || name == "role"
    }

    fn report_unsupported_attr(
        &self,
        ctx: &mut LintContext<'_>,
        element: &ElementNode<'_>,
        attr: &str,
        loc: &SourceLocation,
    ) {
        ctx.error_with_help(
            ctx.t_fmt(
                "a11y/aria-unsupported-elements.message",
                &[("tag", element.tag.as_str()), ("attr", attr)],
            ),
            loc,
            ctx.t("a11y/aria-unsupported-elements.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::AriaUnsupportedElements;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AriaUnsupportedElements));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_div_with_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div aria-label="content"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_meta_with_aria_hidden() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<meta aria-hidden="true" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_script_with_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<script role="presentation"></script>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_meta_with_bound_aria_hidden() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<meta :aria-hidden="hidden" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_script_with_bound_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<script :role="role"></script>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
