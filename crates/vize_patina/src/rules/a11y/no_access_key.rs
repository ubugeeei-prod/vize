//! a11y/no-access-key
//!
//! Disallow the use of the `accesskey` attribute.
//!
//! Access keys are keyboard shortcuts that can conflict with browser
//! and assistive technology shortcuts, creating an inconsistent
//! experience across platforms and devices.
//!
//! Based on eslint-plugin-vuejs-accessibility no-access-key rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "a11y/no-access-key",
    description: "Disallow the use of the accesskey attribute",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow the use of the accesskey attribute
#[derive(Default)]
pub struct NoAccessKey;

/// Markup-IR entry point for `a11y/no-access-key`.
///
/// Flags any `accesskey` binding — static (`accesskey="h"`), `v-bind`
/// (`:accesskey`), or JSX (`accessKey={…}`). Reasoning over the normalized
/// [`MarkupBinding`] view lets one rule body handle both backends; the
/// ASCII-case-insensitive `arg_name_eq` absorbs the JSX `accessKey` casing.
/// Components are exempt on both sides.
impl MarkupRule for NoAccessKey {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if element.is_component() {
            return;
        }
        if !matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) || !binding.arg_name_eq("accesskey")
        {
            return;
        }
        let message = ctx.lint().t("a11y/no-access-key.message");
        let help = ctx.lint().t("a11y/no-access-key.help");
        ctx.lint().warn_at_with_help(message, binding.range(), help);
    }
}

impl Rule for NoAccessKey {
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

        for prop in &element.props {
            match prop {
                PropNode::Attribute(attr) if attr.name == "accesskey" => {
                    ctx.warn_with_help(
                        ctx.t("a11y/no-access-key.message"),
                        &attr.loc,
                        ctx.t("a11y/no-access-key.help"),
                    );
                }
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(
                            &dir.arg,
                            Some(ExpressionNode::Simple(arg)) if arg.content == "accesskey"
                        ) =>
                {
                    ctx.warn_with_help(
                        ctx.t("a11y/no-access-key.message"),
                        &dir.loc,
                        ctx.t("a11y/no-access-key.help"),
                    );
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoAccessKey;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoAccessKey));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_accesskey() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_has_accesskey() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div accesskey="h">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_has_bound_accesskey() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :accesskey="'h'">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
