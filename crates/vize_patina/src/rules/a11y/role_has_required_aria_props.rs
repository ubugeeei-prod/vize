//! a11y/role-has-required-aria-props
//!
//! Require elements with ARIA roles to have all required ARIA properties.
//!
//! Some ARIA roles require specific ARIA attributes to be present.
//! For example, `role="checkbox"` requires `aria-checked` on a non-native element.
//! Native checkbox and radio inputs expose their checked state without it.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div role="checkbox">Check</div>
//! <div role="slider">Slider</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div role="checkbox" aria-checked="false">Check</div>
//! <div role="slider" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100">Slider</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

use super::helpers::{get_required_aria_props, get_static_or_bound_literal_attribute_value};
use super::markup_helpers::{
    get_static_markup_attribute_value, get_static_or_bound_literal_markup_value,
    get_static_or_bound_literal_markup_value_ignore_ascii_case, is_vue_is_component,
};

static META: RuleMeta = RuleMeta {
    name: "a11y/role-has-required-aria-props",
    description: "Require ARIA roles to have required properties",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require ARIA roles to have required properties
#[derive(Default)]
pub struct RoleHasRequiredAriaProps;

fn has_aria_attribute_or_binding(element: &ElementNode, name: &str) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name.eq_ignore_ascii_case(name),
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(
                    &dir.arg,
                    Some(ExpressionNode::Simple(s))
                        if s.is_static && s.content.eq_ignore_ascii_case(name)
                )
        }
    })
}

fn is_focusable_separator(element: &ElementNode) -> bool {
    if has_aria_attribute_or_binding(element, "tabindex") {
        return true;
    }
    if matches!(
        element.tag,
        "button" | "input" | "select" | "textarea" | "summary"
    ) {
        return true;
    }
    if matches!(element.tag, "a" | "area") && has_aria_attribute_or_binding(element, "href") {
        return true;
    }
    if has_aria_attribute_or_binding(element, "contenteditable") {
        return super::helpers::get_static_attribute_value(element, "contenteditable")
            != Some("false");
    }
    false
}

/// A static attribute, or a bind with a static argument.
fn has_markup_attribute_or_binding(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= match binding.kind() {
            MarkupBindingKind::Attribute => binding.is_unqualified_arg_eq_ignore_ascii_case(name),
            MarkupBindingKind::Bind => binding.is_static_unqualified_arg_eq_ignore_ascii_case(name),
            _ => false,
        };
    });
    found
}

fn is_focusable_markup_separator(element: &MarkupElement<'_>) -> bool {
    if has_markup_attribute_or_binding(element, "tabindex")
        || ["button", "input", "select", "textarea", "summary"]
            .iter()
            .any(|tag| element.is_unqualified_tag_exact(tag))
        || ((element.is_unqualified_tag_exact("a") || element.is_unqualified_tag_exact("area"))
            && has_markup_attribute_or_binding(element, "href"))
    {
        return true;
    }
    has_markup_attribute_or_binding(element, "contenteditable")
        && get_static_markup_attribute_value(element, "contenteditable") != Some("false")
}

impl MarkupRule for RoleHasRequiredAriaProps {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if matches!(
            element.kind(),
            MarkupElementKind::Component | MarkupElementKind::Slot | MarkupElementKind::Template
        ) || is_vue_is_component(element)
        {
            return;
        }
        let Some(role) =
            get_static_or_bound_literal_markup_value_ignore_ascii_case(element, "role")
        else {
            return;
        };
        let native_checked = element.is_unqualified_tag_exact("input")
            && matches!(
                get_static_or_bound_literal_markup_value(element, "type"),
                Some("checkbox" | "radio")
            );
        let mut seen = Vec::new();
        for token in role.split_whitespace() {
            let token = token.to_ascii_lowercase();
            if token.is_empty() || seen.iter().any(|seen| seen == &token) {
                continue;
            }
            seen.push(token.clone());
            if token == "separator" && !is_focusable_markup_separator(element) {
                continue;
            }
            report_missing(
                ctx.lint(),
                &token,
                element.range(),
                native_checked,
                |prop| has_markup_attribute_or_binding(element, prop),
            );
        }
    }
}

/// ARIA in HTML forbids aria-checked on native checkbox/radio inputs:
/// their checked state already supplies the required role state.
fn report_missing(
    ctx: &mut LintContext<'_>,
    role: &str,
    range: ByteRange,
    native_checked: bool,
    has_prop: impl Fn(&str) -> bool,
) {
    let required_props = get_required_aria_props(role);
    for &required_prop in required_props {
        if (required_prop == "aria-checked" && native_checked) || has_prop(required_prop) {
            continue;
        }
        let message = ctx.t_fmt(
            "a11y/role-has-required-aria-props.message",
            &[("role", role), ("prop", required_prop)],
        );
        let help = ctx.t_fmt(
            "a11y/role-has-required-aria-props.help",
            &[("role", role), ("props", &required_props.join(", "))],
        );
        ctx.warn_at_with_help(message, range, help);
    }
}

impl Rule for RoleHasRequiredAriaProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component
            || matches!(element.tag_type, ElementType::Slot | ElementType::Template)
            || element.props.iter().any(|prop| {
                matches!(prop, PropNode::Attribute(attr)
                    if attr.name.eq_ignore_ascii_case("is")
                        && attr.value.as_ref().is_some_and(|value| value.content.starts_with("vue:")))
            })
        {
            return;
        }

        let Some(role) = get_static_or_bound_literal_attribute_value(element, "role") else {
            return;
        };
        let native_checked = element.tag == "input"
            && matches!(
                get_static_or_bound_literal_attribute_value(element, "type"),
                Some("checkbox" | "radio")
            );
        let range = crate::ir::ByteRange::new(element.loc.span.start, element.loc.span.end);
        let mut seen = Vec::new();
        for token in role.split_whitespace() {
            let token = token.to_ascii_lowercase();
            if token.is_empty() || seen.iter().any(|seen| seen == &token) {
                continue;
            }
            seen.push(token.clone());
            if token == "separator" && !is_focusable_separator(element) {
                continue;
            }
            report_missing(ctx, &token, range, native_checked, |prop| {
                has_aria_attribute_or_binding(element, prop)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoleHasRequiredAriaProps;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(RoleHasRequiredAriaProps));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_checkbox_with_checked() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="checkbox" aria-checked="false">Check</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_checkbox_with_dynamic_checked() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="checkbox" :aria-checked="checked">Check</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn object_binding_does_not_hide_a_missing_required_property() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<input role="combobox" aria-expanded="false" v-bind="fieldProps" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn native_checked_inputs_do_not_require_aria_checked() {
        let linter = create_linter();
        for source in [
            r#"<input type="checkbox" role="switch" />"#,
            r#"<input type="checkbox" role="menuitemcheckbox" />"#,
            r#"<input :type="'checkbox'" role="switch" />"#,
            r#"<input type="radio" role="menuitemradio" />"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 0, "{source}");
        }
    }

    #[test]
    fn non_native_or_dynamic_input_types_still_require_aria_checked() {
        let linter = create_linter();
        for source in [
            r#"<div role="switch"></div>"#,
            r#"<input type="text" role="switch" />"#,
            r#"<input :type="kind" role="switch" />"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 1, "{source}");
        }
    }

    #[test]
    fn wai_aria_12_role_requirements_are_complete() {
        let linter = create_linter();
        for source in [
            r#"<input role="combobox" aria-expanded="false" />"#,
            r#"<div role="menuitemcheckbox" tabindex="-1">Bold</div>"#,
            r#"<div role="menuitemradio" tabindex="-1">Left</div>"#,
            r#"<li role="treeitem">Node</li>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 1, "{source}");
        }
    }

    #[test]
    fn test_invalid_checkbox_missing_checked() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="checkbox">Check</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_bound_literal_checkbox_missing_checked() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :role="'checkbox'">Check</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_slider_missing_valuenow() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="slider">Slider</div>"#, "test.vue");
        // slider requires aria-valuenow
        assert!(result.warning_count >= 1);
    }

    #[test]
    fn test_valid_no_required_props() {
        let linter = create_linter();
        // "button" role has no required ARIA props
        let result = linter.lint_template(r#"<div role="button">Button</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyCheckbox role="checkbox">Check</MyCheckbox>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn static_separators_do_not_require_a_value() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<hr role="separator" /><div role="separator"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn a_slot_outlet_is_not_an_element_with_a_role() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<slot role="checkbox" /><slot :role="'checkbox'" /><slot name="item" role="switch" /><div role="checkbox">fallback</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn role_tokens_are_split_trimmed_and_lowercased() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="switch checkbox">a</div><div role="SLIDER">b</div><div role=" checkbox ">c</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 4);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            [
                "Elements with role=\"switch\" must have the 'aria-checked' attribute",
                "Elements with role=\"checkbox\" must have the 'aria-checked' attribute",
                "Elements with role=\"slider\" must have the 'aria-valuenow' attribute",
                "Elements with role=\"checkbox\" must have the 'aria-checked' attribute",
            ]
        );
    }

    #[test]
    fn an_object_bind_does_not_supply_a_missing_aria_property() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="checkbox" v-bind="attrs"></div><div role="slider" v-bind="$attrs"></div><div role="option" v-bind="{ id: 'x' }"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 3);
    }

    #[test]
    fn an_uppercase_aria_attribute_satisfies_the_role() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div ROLE="checkbox" ARIA-CHECKED="false">a</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn vue_is_substitution_is_not_a_native_role_host() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div is="vue:my-item" role="checkbox">b</div><template v-if="ok" role="checkbox">x</template>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn focusable_separators_require_a_value() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="separator" tabindex="0"></div><div role="separator" :tabindex="index"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }
}
