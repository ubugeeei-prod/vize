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
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

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

    fn traversal(&self) -> crate::markup::MarkupTraversal {
        crate::markup::MarkupTraversal::Bindings
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if !matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) || !binding.is_static_unqualified_arg_eq_ignore_ascii_case("tabindex")
        {
            return;
        }
        let positive = binding
            .literal_string_value()
            .is_some_and(crate::attribute_policy::positive_tabindex)
            || binding
                .expression()
                .is_some_and(crate::attribute_policy::positive_bound_tabindex_literal);
        if positive {
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
            let location = match prop {
                PropNode::Attribute(attr)
                    if attr.name == "tabindex"
                        && attr.value.as_ref().is_some_and(|value| {
                            crate::attribute_policy::positive_tabindex(value.content)
                        }) =>
                {
                    &attr.loc
                }
                PropNode::Directive(directive)
                    if directive.name == "bind"
                        && matches!(
                            directive.arg.as_ref(),
                            Some(ExpressionNode::Simple(argument))
                                if argument.content == "tabindex"
                        )
                        && matches!(
                            directive.exp.as_ref(),
                            Some(ExpressionNode::Simple(expression))
                                if crate::attribute_policy::positive_bound_tabindex_literal(
                                    expression.content
                                )
                        ) =>
                {
                    &directive.loc
                }
                _ => continue,
            };
            ctx.warn_with_help(
                ctx.t("a11y/tabindex-no-positive.message"),
                location,
                ctx.t("a11y/tabindex-no-positive.help"),
            );
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

    #[test]
    fn reports_eslint_plugin_vuejs_accessibility_numeric_values() {
        let result = create_linter().lint_template(
            r#"<div>
                <div tabindex="1.5">decimal</div>
                <div tabindex=" 1 ">padded</div>
                <div :tabindex="2">bound number</div>
                <div :tabindex="'2'">bound string</div>
                <div v-bind:tabindex="2">longhand</div>
                <MyButton :tabindex="2">component</MyButton>
            </div>"#,
            "test.vue",
        );

        assert_eq!(result.warning_count, 6, "{:?}", result.diagnostics);
    }
}
