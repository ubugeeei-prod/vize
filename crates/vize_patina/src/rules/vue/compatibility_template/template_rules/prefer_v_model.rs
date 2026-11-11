//! Matching binds and update listeners that can use `v-model`.
//!
//! Mirrors eslint-plugin-vue: on a custom component, `:prop` pairs with the
//! `@update:` listener whose camelized name matches, and the listener must
//! assign its value to the exact bound text (`x = $event` or `(v) => x = v`).

use super::shared::static_argument;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, BindingPattern, Expression, SimpleAssignmentTarget,
    Statement,
};
use oxc_span::GetSpan;
use vize_l0::{String, cstr, is_native_tag};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-v-model",
    description: "Prefer v-model over matching bind and update listeners",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/prefer-v-model`.
pub struct PreferVModel;

impl Rule for PreferVModel {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_custom_component(element) {
            return;
        }
        let directives = || {
            element.props.iter().filter_map(|prop| match prop {
                PropNode::Directive(directive) if directive.modifiers.is_empty() => {
                    Some(directive.as_ref())
                }
                _ => None,
            })
        };
        for bind in directives().filter(|directive| directive.name == "bind") {
            let Some(prop_name) = static_argument(bind) else {
                continue;
            };
            let model_name = camel_case(prop_name);
            let Some((listener, event_name)) = directives().find_map(|directive| {
                let event = update_event(directive)?;
                (camel_case(event) == model_name).then_some((directive, event))
            }) else {
                continue;
            };
            let (Some(ExpressionNode::Simple(bound)), Some(ExpressionNode::Simple(handler))) =
                (bind.exp.as_ref(), listener.exp.as_ref())
            else {
                continue;
            };
            if !mirrors_assignment(ctx, bound, handler) {
                continue;
            }
            let v_model = if model_name == "modelValue" {
                String::from("v-model")
            } else {
                cstr!("v-model:{prop_name}")
            };
            ctx.warn_with_help(
                cstr!("Prefer `{v_model}` over `:{prop_name}` and `@update:{event_name}`"),
                &bind.loc,
                "Replace the matching bind and update listener with v-model",
            );
        }
    }
}

/// eslint-plugin-vue's `isCustomComponent`: `<slot>` is an HTML element there.
fn is_custom_component(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "is",
        PropNode::Directive(directive) => {
            directive.name == "is"
                || (directive.name == "bind" && static_argument(directive) == Some("is"))
        }
    }) || !(is_native_tag(element.tag) || element.tag == "slot")
}

fn update_event<'a>(directive: &'a DirectiveNode<'a>) -> Option<&'a str> {
    if directive.name != "on" {
        return None;
    }
    match &directive.arg {
        Some(ExpressionNode::Simple(argument)) if argument.is_static => {
            argument.content.strip_prefix("update:")
        }
        _ => None,
    }
}

/// `camelCase` of eslint-plugin-vue's casing utilities.
fn camel_case(name: &str) -> String {
    let symbols = name.chars().any(|character| {
        "!\"#%&'()*+,./:;<=>?@[\\]^`{|}".contains(character)
            || character.is_whitespace()
            || matches!(character, '-' | '_')
    });
    if !symbols && !name.starts_with(|character: char| character.is_ascii_lowercase()) {
        let mut characters = name.chars();
        return characters.next().map_or_else(String::default, |first| {
            cstr!("{}{}", first.to_lowercase(), characters.as_str())
        });
    }
    let mut output = String::default();
    let mut characters = name.chars().peekable();
    while let Some(character) = characters.next() {
        if matches!(character, '-' | '_')
            && let Some(next) = characters
                .peek()
                .copied()
                .filter(|next| next.is_ascii_alphanumeric() || *next == '_')
        {
            characters.next();
            output.push(next.to_ascii_uppercase());
        } else {
            output.push(character);
        }
    }
    output
}

/// `x = $event` as the only statement, or `(v) => x = v`, where the assigned
/// target has the same text as the bound expression.
fn mirrors_assignment<'a>(
    ctx: &mut LintContext<'a>,
    bound: &SimpleExpressionNode<'a>,
    handler: &SimpleExpressionNode<'a>,
) -> bool {
    let bound_expression = bound
        .js_ast
        .filter(|ast| ast.raw == bound.content)
        .and_then(|ast| ast.as_expression())
        .map(|expression| expression.ast)
        .or_else(|| ctx.compatibility_expression(bound.loc.span, bound.content));
    let Some(bound_expression) = bound_expression else {
        return false;
    };
    let Some(bound_text) = text(bound.content, bound_expression.without_parentheses().span())
    else {
        return false;
    };
    let Some(handler_program) = ctx.compatibility_handler(handler.loc.span, handler.content) else {
        return false;
    };
    let [Statement::ExpressionStatement(statement)] = handler_program.body.as_slice() else {
        return false;
    };
    let (assignment, value_name) = match statement.expression.without_parentheses() {
        Expression::ArrowFunctionExpression(arrow) => {
            let [parameter] = arrow.params.items.as_slice() else {
                return false;
            };
            let (BindingPattern::BindingIdentifier(parameter_name), None, None) = (
                &parameter.pattern,
                &parameter.initializer,
                &arrow.params.rest,
            ) else {
                return false;
            };
            let Some(Expression::AssignmentExpression(assignment)) =
                arrow.get_expression().map(Expression::without_parentheses)
            else {
                return false;
            };
            (assignment, parameter_name.name.as_str())
        }
        Expression::AssignmentExpression(assignment) => (assignment, "$event"),
        _ => return false,
    };
    assignment.operator == AssignmentOperator::Assign
        && matches!(assignment.right.without_parentheses(), Expression::Identifier(value) if value.name == value_name)
        && target_text(handler.content, &assignment.left) == Some(bound_text)
}

fn target_text<'s>(source: &'s str, target: &AssignmentTarget<'_>) -> Option<&'s str> {
    let span = match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => identifier.span,
        target => match target.as_simple_assignment_target() {
            Some(SimpleAssignmentTarget::TSAsExpression(_)) | None => return None,
            Some(simple) => simple.span(),
        },
    };
    text(source, span)
}

fn text(source: &str, span: oxc_span::Span) -> Option<&str> {
    source.get(span.start as usize..span.end as usize)
}
