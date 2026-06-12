//! a11y/tabindex-no-positive
//!
//! Disallow positive tabindex values.
//!
//! Positive tabindex values disrupt the natural tab order and can make
//! navigation confusing for keyboard users. Use 0 or -1 instead.
//!
//! Based on eslint-plugin-vuejs-accessibility tabindex-no-positive rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "a11y/tabindex-no-positive",
    description: "Disallow positive tabindex values",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow positive tabindex values
#[derive(Default)]
pub struct TabindexNoPositive;

impl TabindexNoPositive {
    /// Whether a statically-written `tabindex` value parses to a positive int.
    fn is_positive(value: &str) -> bool {
        value.parse::<i32>().is_ok_and(|num| num > 0)
    }
}

/// Markup-IR entry point for `a11y/tabindex-no-positive`.
///
/// Inspects the statically-written `tabindex` value on both backends: a Vue
/// `tabindex="1"` and a JSX `tabIndex="1"` both surface a static binding value
/// of `"1"`. A JSX numeric-expression `tabIndex={1}` has no static string value,
/// so — exactly like the legacy template path, which only reads static
/// attribute text — it is not flagged here.
impl MarkupRule for TabindexNoPositive {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if !binding.arg_name_eq("tabindex") {
            return;
        }
        let Some(value) = binding.static_value() else {
            return;
        };
        if Self::is_positive(value) {
            let message = ctx.lint().t("a11y/tabindex-no-positive.message");
            let help = ctx.lint().t("a11y/tabindex-no-positive.help");
            ctx.lint().warn_at_with_help(message, binding.range(), help);
        }
    }
}

impl Rule for TabindexNoPositive {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        for prop in &element.props {
            if let PropNode::Attribute(attr) = prop
                && attr.name == "tabindex"
                && let Some(value) = &attr.value
                && let Ok(num) = value.content.parse::<i32>()
                && num > 0
            {
                ctx.warn_with_help(
                    ctx.t("a11y/tabindex-no-positive.message"),
                    &attr.loc,
                    ctx.t("a11y/tabindex-no-positive.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TabindexNoPositive;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TabindexNoPositive));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_zero() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div tabindex="0">Focusable</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_negative() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div tabindex="-1">Programmatic focus</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_positive() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div tabindex="1">Bad focus order</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_large_positive() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div tabindex="99">Very bad</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
