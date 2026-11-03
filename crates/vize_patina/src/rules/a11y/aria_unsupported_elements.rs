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
use crate::ir::ByteRange;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode, SourceLocation};

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

impl MarkupRule for AriaUnsupportedElements {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.kind() == MarkupElementKind::Component
            || !helpers::ARIA_UNSUPPORTED_ELEMENTS
                .iter()
                .any(|tag| element.is_unqualified_tag_exact(tag))
        {
            return;
        }
        element.walk_bindings(&mut |binding| {
            let attr = match binding.kind() {
                MarkupBindingKind::Attribute => binding.arg_name(),
                MarkupBindingKind::Bind => binding
                    .arg_name()
                    .filter(|arg| binding.is_static_unqualified_arg_exact(arg)),
                _ => None,
            };
            if let Some(attr) = attr
                && Self::is_aria_or_role(attr)
            {
                Self::report_at(ctx.lint(), element.tag(), attr, binding.range());
            }
        });
    }
}

impl Rule for AriaUnsupportedElements {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        if !helpers::ARIA_UNSUPPORTED_ELEMENTS.contains(&element.tag) {
            return;
        }

        for prop in &element.props {
            match prop {
                PropNode::Attribute(attr) if Self::is_aria_or_role(attr.name) => {
                    self.report_unsupported_attr(ctx, element, attr.name, &attr.loc);
                }
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && arg.is_static
                        && Self::is_aria_or_role(arg.content) =>
                {
                    self.report_unsupported_attr(ctx, element, arg.content, &dir.loc);
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
        Self::report_at(
            ctx,
            element.tag,
            attr,
            ByteRange::new(loc.span.start, loc.span.end),
        );
    }

    fn report_at(ctx: &mut LintContext<'_>, tag: &str, attr: &str, range: ByteRange) {
        let message = ctx.t_fmt(
            "a11y/aria-unsupported-elements.message",
            &[("tag", tag), ("attr", attr)],
        );
        let help = ctx.t("a11y/aria-unsupported-elements.help");
        ctx.error_at_with_help(message, range, help);
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

    #[test]
    fn test_valid_script_with_dynamic_role_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<script :[role]="role"></script>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
