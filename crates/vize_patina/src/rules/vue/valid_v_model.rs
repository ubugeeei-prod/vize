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
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression as OxcExpression;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_relief::{DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode};
use vize_s0::is_native_tag;

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
        if directive.name != "model" {
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

        if let Some(expression_error_key) = model_expression_error_key(&directive.exp) {
            ctx.error_with_help(
                ctx.t(expression_error_key),
                &directive.loc,
                ctx.t("vue/valid-v-model.help"),
            );
        }

        // Check 2: v-model must be on valid elements
        let tag = element.tag.to_lowercase();
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

        // Check 3: v-model cannot read file inputs
        if !is_component && is_static_file_input(element) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-model.unsupported_file_input"),
                &directive.loc,
                ctx.t("vue/valid-v-model.help"),
            );
        }

        // Check 4: Native v-model does not support arguments
        if !is_component && let Some(arg) = &directive.arg {
            let loc = match arg {
                ExpressionNode::Simple(simple) => &simple.loc,
                ExpressionNode::Compound(_) => &directive.loc,
            };
            ctx.error_with_help(
                ctx.t("vue/valid-v-model.unexpected_argument"),
                loc,
                ctx.t("vue/valid-v-model.help"),
            );
        }

        // Check 5: Validate modifiers (only for native elements)
        if !is_component {
            for modifier in directive.modifiers.iter() {
                let mod_name = modifier.content;
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

    let tag = element.tag;
    tag == "component" || !is_native_tag(tag)
}

fn is_static_file_input(element: &ElementNode<'_>) -> bool {
    if !element.tag.eq_ignore_ascii_case("input") {
        return false;
    }

    element.props.iter().any(|prop| {
        let PropNode::Attribute(attr) = prop else {
            return false;
        };
        attr.name.eq_ignore_ascii_case("type")
            && attr
                .value
                .as_ref()
                .is_some_and(|value| value.content.trim().eq_ignore_ascii_case("file"))
    })
}

/// Check if expression is empty
fn is_empty_expression(exp: &ExpressionNode) -> bool {
    match exp {
        ExpressionNode::Simple(s) => s.content.trim().is_empty(),
        ExpressionNode::Compound(c) => c.children.is_empty(),
    }
}

fn model_expression_error_key(exp: &Option<ExpressionNode<'_>>) -> Option<&'static str> {
    let Some(ExpressionNode::Simple(simple)) = exp else {
        return None;
    };
    let source = simple.content.trim();
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let Ok(expression) = Parser::new(&allocator, source, source_type).parse_expression() else {
        return Some("vue/valid-v-model.invalid_expression");
    };

    if expression.span().end as usize != source.len() {
        return Some("vue/valid-v-model.invalid_expression");
    }
    if expression_contains_optional_chain(&expression) {
        return Some("vue/valid-v-model.optional_chain");
    }
    (!is_assignable_model_target(&expression)).then_some("vue/valid-v-model.invalid_expression")
}

fn is_assignable_model_target(expression: &OxcExpression<'_>) -> bool {
    match expression {
        OxcExpression::Identifier(_)
        | OxcExpression::StaticMemberExpression(_)
        | OxcExpression::ComputedMemberExpression(_)
        | OxcExpression::PrivateFieldExpression(_) => true,
        OxcExpression::ParenthesizedExpression(parenthesized) => {
            is_assignable_model_target(&parenthesized.expression)
        }
        OxcExpression::TSNonNullExpression(ts_non_null) => {
            is_assignable_model_target(&ts_non_null.expression)
        }
        OxcExpression::TSAsExpression(ts_as) => is_assignable_model_target(&ts_as.expression),
        OxcExpression::TSSatisfiesExpression(ts_satisfies) => {
            is_assignable_model_target(&ts_satisfies.expression)
        }
        OxcExpression::TSTypeAssertion(ts_assertion) => {
            is_assignable_model_target(&ts_assertion.expression)
        }
        _ => false,
    }
}

fn expression_contains_optional_chain(expression: &OxcExpression<'_>) -> bool {
    match expression {
        OxcExpression::ChainExpression(_) => true,
        OxcExpression::StaticMemberExpression(member) => {
            member.optional || expression_contains_optional_chain(&member.object)
        }
        OxcExpression::ComputedMemberExpression(member) => {
            member.optional
                || expression_contains_optional_chain(&member.object)
                || expression_contains_optional_chain(&member.expression)
        }
        OxcExpression::PrivateFieldExpression(member) => {
            member.optional || expression_contains_optional_chain(&member.object)
        }
        OxcExpression::CallExpression(call) => {
            call.optional || expression_contains_optional_chain(&call.callee)
        }
        OxcExpression::ParenthesizedExpression(parenthesized) => {
            expression_contains_optional_chain(&parenthesized.expression)
        }
        OxcExpression::TSNonNullExpression(ts_non_null) => {
            expression_contains_optional_chain(&ts_non_null.expression)
        }
        OxcExpression::TSAsExpression(ts_as) => {
            expression_contains_optional_chain(&ts_as.expression)
        }
        OxcExpression::TSSatisfiesExpression(ts_satisfies) => {
            expression_contains_optional_chain(&ts_satisfies.expression)
        }
        OxcExpression::TSTypeAssertion(ts_assertion) => {
            expression_contains_optional_chain(&ts_assertion.expression)
        }
        _ => false,
    }
}

#[cfg(test)]
mod assignment_target_tests;

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
    fn test_valid_v_model_on_lowercase_custom_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<multiselect v-model="selected" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_model_on_dynamic_component() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<component :is="overlay" v-model:open="open" />"#,
            "test.vue",
        );
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

    #[test]
    fn test_invalid_v_model_argument_on_native_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-model:foo="value">"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_model_on_file_input() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="file" v-model="file">"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
