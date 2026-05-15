//! vue/valid-v-model
//!
//! Enforce valid `v-model` directives.
//!
//! `v-model` must:
//! - Have an expression
//! - Be on a valid element (input, select, textarea, or component)
//! - Not have invalid modifiers
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-model="foo"></div>
//! <input v-model>
//! ```
//!
//! ### Valid
//! ```vue
//! <input v-model="foo">
//! <select v-model="selected"></select>
//! <textarea v-model="text"></textarea>
//! <MyComponent v-model="value" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::is_html_tag;
use vize_relief::ast::{DirectiveNode, ElementNode, ElementType, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-model",
    description: "Enforce valid `v-model` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-model directives
pub struct ValidVModel;

/// Elements that can use v-model
const VALID_V_MODEL_ELEMENTS: &[&str] = &["input", "select", "textarea"];

/// Valid modifiers for v-model
const VALID_MODIFIERS: &[&str] = &["lazy", "number", "trim"];

impl Rule for ValidVModel {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "model" {
            return;
        }

        // Check 1: v-model must have an expression
        let has_expression = match &directive.exp {
            Some(exp) => !is_empty_expression(exp),
            None => false,
        };

        if !has_expression {
            ctx.error_with_help(
                ctx.t("vue/valid-v-model.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-model.help"),
            );
            return;
        }

        // Check 2: v-model must be on valid elements
        let tag = element.tag.as_str().to_lowercase();
        let is_component = is_component_like_tag(element);
        let is_valid_element = VALID_V_MODEL_ELEMENTS.contains(&tag.as_str()) || is_component;

        if !is_valid_element {
            ctx.error_with_help(
                ctx.t_fmt("vue/valid-v-model.invalid_element", &[("tag", &tag)]),
                &directive.loc,
                ctx.t("vue/valid-v-model.help"),
            );
            return;
        }

        // Check 3: Validate modifiers (only for native elements)
        if !is_component {
            for modifier in directive.modifiers.iter() {
                let mod_name = modifier.content.as_str();
                if !VALID_MODIFIERS.contains(&mod_name) {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-model.missing_expression"),
                        &modifier.loc,
                        ctx.t("vue/valid-v-model.help"),
                    );
                }
            }
        }
    }
}

fn is_component_like_tag(element: &ElementNode<'_>) -> bool {
    if element.tag_type == ElementType::Component {
        return true;
    }

    let tag = element.tag.as_str();
    tag.contains('-') && !is_html_tag(tag)
}

/// Check if expression is empty
fn is_empty_expression(exp: &ExpressionNode) -> bool {
    match exp {
        ExpressionNode::Simple(s) => s.content.trim().is_empty(),
        ExpressionNode::Compound(c) => c.children.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVModel;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVModel));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_model_input() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-model="foo">"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_model_select() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<select v-model="selected"></select>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_model_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-model.trim="foo">"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_model_on_kebab_case_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a-rate v-model:value="score" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_model_on_custom_element_like_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-widget v-model="value"></my-widget>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_model_on_div() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-model="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_model_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-model>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
