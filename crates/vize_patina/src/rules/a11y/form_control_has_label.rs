//! a11y/form-control-has-label
//!
//! Require form controls to have associated labels.
//!
//! Form controls (input, select, textarea) must have associated labels
//! for screen reader users. This can be via <label>, aria-label, or
//! aria-labelledby.
//!
//! Based on eslint-plugin-vuejs-accessibility form-control-has-label rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::helpers::string_literal_value;

static META: RuleMeta = RuleMeta {
    name: "a11y/form-control-has-label",
    description: "Require form controls to have associated labels",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require form controls to have associated labels
#[derive(Default)]
pub struct FormControlHasLabel;

impl FormControlHasLabel {
    /// Check if an element is a form control that needs a label
    fn is_form_control(element: &MarkupElement<'_>) -> bool {
        element.is_unqualified_tag_exact("input")
            || element.is_unqualified_tag_exact("select")
            || element.is_unqualified_tag_exact("textarea")
    }

    /// Check if the input type doesn't need a label (hidden, submit, etc.)
    fn is_exempt_input_type(element: &MarkupElement<'_>) -> bool {
        if !element.is_unqualified_tag_exact("input") {
            return false;
        }

        let Some(input_type) = Self::get_static_or_bound_literal_attribute_value(element, "type")
        else {
            return false;
        };

        matches!(
            input_type,
            "hidden" | "submit" | "reset" | "button" | "image"
        )
    }

    /// Check if element has aria-label or aria-labelledby
    fn has_aria_label(element: &MarkupElement<'_>) -> bool {
        Self::has_non_empty_static_attribute(element, "aria-label")
            || Self::has_non_empty_static_attribute(element, "aria-labelledby")
            || Self::has_bound_attribute(element, "aria-label")
            || Self::has_bound_attribute(element, "aria-labelledby")
    }

    /// Check if element has an id (potentially used by a label)
    fn has_id(element: &MarkupElement<'_>) -> bool {
        Self::has_non_empty_static_attribute(element, "id")
            || Self::has_bound_attribute(element, "id")
    }

    /// Check if element has a placeholder (weak but sometimes acceptable)
    fn has_placeholder(element: &MarkupElement<'_>) -> bool {
        Self::has_non_empty_static_attribute(element, "placeholder")
    }

    /// Check if element has a title attribute
    fn has_title(element: &MarkupElement<'_>) -> bool {
        Self::has_non_empty_static_attribute(element, "title")
    }

    fn has_bound_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Bind && binding.is_unqualified_arg_exact(name) {
                found = true;
            }
        });
        found
    }

    fn has_non_empty_static_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact(name)
                && binding
                    .static_value()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                found = true;
            }
        });
        found
    }

    fn get_static_or_bound_literal_attribute_value<'a>(
        element: &MarkupElement<'a>,
        name: &str,
    ) -> Option<&'a str> {
        let mut result = None;
        element.walk_bindings(&mut |binding| {
            if result.is_some() {
                return;
            }

            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact(name)
            {
                result = Some(binding.static_value());
                return;
            }

            if binding.kind() == MarkupBindingKind::Bind
                && binding.is_unqualified_arg_exact(name)
                && let Some(value) = binding.expression().and_then(string_literal_value)
            {
                result = Some(Some(value));
            }
        });
        result.flatten()
    }

    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        has_label_ancestor: bool,
    ) {
        if !Self::is_form_control(element) {
            return;
        }

        // Skip inputs that don't need labels
        if Self::is_exempt_input_type(element) {
            return;
        }

        // Check for various label methods
        let has_label = Self::has_aria_label(element)
            || Self::has_id(element)
            || Self::has_title(element)
            || has_label_ancestor;

        if !has_label {
            let help = if Self::has_placeholder(element) {
                ctx.t("a11y/form-control-has-label.help_placeholder")
            } else {
                ctx.t("a11y/form-control-has-label.help")
            };

            ctx.warn_at_with_help(
                ctx.t_fmt(
                    "a11y/form-control-has-label.message",
                    &[("tag", element.tag())],
                ),
                element.range(),
                help,
            );
        }
    }
}

impl MarkupRule for FormControlHasLabel {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let has_label_ancestor =
            ctx.has_ancestor(|parent| parent.is_unqualified_tag_exact("label"));
        Self::check_element(ctx.lint(), element, has_label_ancestor);
    }
}

impl Rule for FormControlHasLabel {
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
        let has_label_ancestor = ctx.has_ancestor(|parent| parent.tag.as_str() == "label");
        Self::check_element(ctx, &MarkupElement::new(element), has_label_ancestor);
    }
}

#[cfg(test)]
mod tests {
    use super::FormControlHasLabel;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(FormControlHasLabel));
        Linter::with_registry(registry)
    }

    fn assert_template_warnings(source: &str, expected: usize) {
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, expected, "{:?}", result.diagnostics);
    }

    fn assert_jsx_warnings(source: &str, expected: usize) {
        let result = create_linter().lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(result.warning_count, expected, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_valid_with_id() {
        assert_template_warnings(r#"<input type="text" id="name" />"#, 0);
    }

    #[test]
    fn test_valid_with_aria_label() {
        assert_template_warnings(r#"<input type="text" aria-label="Name" />"#, 0);
    }

    #[test]
    fn test_valid_hidden_input() {
        assert_template_warnings(r#"<input type="hidden" value="token" />"#, 0);
    }

    #[test]
    fn test_valid_bound_literal_hidden_input() {
        assert_template_warnings(r#"<input :type="'hidden'" value="token" />"#, 0);
    }

    #[test]
    fn test_valid_submit_button() {
        assert_template_warnings(r#"<input type="submit" value="Submit" />"#, 0);
    }

    #[test]
    fn test_valid_inside_label() {
        assert_template_warnings(r#"<label>Name <input type="text" /></label>"#, 0);
    }

    #[test]
    fn test_valid_inside_label_nested_span() {
        assert_template_warnings(
            r#"<label><span><input type="checkbox" /></span></label>"#,
            0,
        );
    }

    #[test]
    fn test_invalid_no_label() {
        assert_template_warnings(r#"<input type="text" />"#, 1);
    }

    #[test]
    fn test_invalid_select_no_label() {
        assert_template_warnings(r#"<select><option>A</option></select>"#, 1);
    }

    #[test]
    fn test_invalid_textarea_no_label() {
        assert_template_warnings(r#"<textarea></textarea>"#, 1);
    }

    #[test]
    fn test_valid_jsx_with_id() {
        assert_jsx_warnings(r#"const Field = () => <input type="text" id="name" />;"#, 0);
    }

    #[test]
    fn test_valid_jsx_with_bound_aria_label() {
        assert_jsx_warnings(
            r#"const Field = () => <input type="text" aria-label={label} />;"#,
            0,
        );
    }

    #[test]
    fn test_valid_jsx_bound_literal_hidden_input() {
        assert_jsx_warnings(
            r#"const Field = () => <input type={'hidden'} value={token} />;"#,
            0,
        );
    }

    #[test]
    fn test_valid_jsx_inside_label_nested_span() {
        assert_jsx_warnings(
            r#"const Field = () => <label><span><input type="checkbox" /></span></label>;"#,
            0,
        );
    }

    #[test]
    fn test_invalid_jsx_no_label() {
        assert_jsx_warnings(r#"const Field = () => <input type="text" />;"#, 1);
    }

    #[test]
    fn test_invalid_jsx_dynamic_title_is_not_label() {
        assert_jsx_warnings(
            r#"const Field = () => <input type="text" title={title} />;"#,
            1,
        );
    }
}
