//! a11y/no-autofocus
//!
//! Disallow the use of the `autofocus` attribute.
//!
//! The autofocus attribute can cause usability issues for sighted and
//! non-sighted users by moving focus unexpectedly, disrupting the
//! natural reading order and navigation flow.
//!
//! Based on eslint-plugin-vuejs-accessibility no-autofocus rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "a11y/no-autofocus",
    description: "Disallow the use of the autofocus attribute",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow the use of the autofocus attribute
#[derive(Default)]
pub struct NoAutofocus;

/// Markup-IR entry point for `a11y/no-autofocus`.
///
/// Flags any `autofocus` binding — static (`autofocus` / `autofocus="true"`),
/// `v-bind` (`:autofocus`), or JSX (`autofocus={…}` / `autoFocus={…}`). The
/// normalized [`MarkupBinding`] view answers "is there an autofocus prop?"
/// identically on both backends; `arg_name_eq` is ASCII-case-insensitive, so
/// the JSX `autoFocus` casing matches too. Components are exempt on both sides.
impl MarkupRule for NoAutofocus {
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
        ) || !binding.arg_name_eq("autofocus")
        {
            return;
        }
        let message = ctx.lint().t("a11y/no-autofocus.message");
        let help = ctx.lint().t("a11y/no-autofocus.help");
        ctx.lint().warn_at_with_help(message, binding.range(), help);
    }
}

impl Rule for NoAutofocus {
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
                PropNode::Attribute(attr) if attr.name == "autofocus" => {
                    ctx.warn_with_help(
                        ctx.t("a11y/no-autofocus.message"),
                        &attr.loc,
                        ctx.t("a11y/no-autofocus.help"),
                    );
                }
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(
                            &dir.arg,
                            Some(ExpressionNode::Simple(arg)) if arg.content == "autofocus"
                        ) =>
                {
                    ctx.warn_with_help(
                        ctx.t("a11y/no-autofocus.message"),
                        &dir.loc,
                        ctx.t("a11y/no-autofocus.help"),
                    );
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoAutofocus;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoAutofocus));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_autofocus() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="text" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_has_autofocus() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="text" autofocus />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_has_bound_autofocus() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="text" :autofocus="true" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
