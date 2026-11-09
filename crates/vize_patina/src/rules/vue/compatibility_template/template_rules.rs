//! Vue-template-specific compatibility rules.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::Expression;
use vize_l0::{String, cstr, is_native_tag};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

macro_rules! meta {
    ($name:literal, $description:literal) => {
        RuleMeta {
            name: $name,
            description: $description,
            category: RuleCategory::Recommended,
            fixable: false,
            default_severity: Severity::Warning,
        }
    };
}

static NO_LITERALS_IN_TEMPLATE_META: RuleMeta = meta!(
    "vue/no-literals-in-template",
    "Disallow object, array and function literals in v-bind"
);

/// `vue/no-literals-in-template`.
pub struct NoLiteralsInTemplate;

impl Rule for NoLiteralsInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &NO_LITERALS_IN_TEMPLATE_META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "bind"
            || static_argument(directive)
                .is_some_and(|argument| matches!(argument, "class" | "style"))
        {
            return;
        }
        let Some(ExpressionNode::Simple(expression)) = &directive.exp else {
            return;
        };
        let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
            return;
        };
        if matches!(
            parsed.ast.get_inner_expression(),
            Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ArrowFunctionExpression(_)
        ) {
            ctx.warn_with_help(
                "Unexpected literal in a template binding",
                &expression.loc,
                "Move the literal to a script binding so Vue can reuse the same value",
            );
        }
    }
}

static NO_RESTRICTED_V_BIND_META: RuleMeta = meta!(
    "vue/no-restricted-v-bind",
    "Disallow restricted v-bind arguments"
);

/// `vue/no-restricted-v-bind`; the default rejects `:v-*` attributes.
pub struct NoRestrictedVBind;

impl Rule for NoRestrictedVBind {
    fn meta(&self) -> &'static RuleMeta {
        &NO_RESTRICTED_V_BIND_META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name == "bind"
            && static_argument(directive).is_some_and(|argument| argument.starts_with("v-"))
        {
            ctx.warn_with_help(
                "Using `:v-*` is not allowed",
                &directive.loc,
                "Remove `:` and use the attribute as a directive",
            );
        }
    }
}

static PREFER_V_MODEL_META: RuleMeta = meta!(
    "vue/prefer-v-model",
    "Prefer v-model over matching bind and update listeners"
);

/// `vue/prefer-v-model`.
pub struct PreferVModel;

impl Rule for PreferVModel {
    fn meta(&self) -> &'static RuleMeta {
        &PREFER_V_MODEL_META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if is_native_tag(element.tag) {
            return;
        }
        for bind in element.props.iter().filter_map(|prop| match prop {
            PropNode::Directive(directive)
                if directive.name == "bind" && directive.modifiers.is_empty() =>
            {
                Some(directive)
            }
            _ => None,
        }) {
            let Some(prop_name) = static_argument(bind) else {
                continue;
            };
            let Some(bind_expression) = simple_expression(bind.exp.as_ref()) else {
                continue;
            };
            let update_name = cstr!("update:{}", prop_name);
            let matching = element.props.iter().find_map(|prop| match prop {
                PropNode::Directive(directive)
                    if directive.name == "on"
                        && directive.modifiers.is_empty()
                        && static_argument(directive) == Some(update_name.as_str()) =>
                {
                    Some(directive)
                }
                _ => None,
            });
            let Some(handler) =
                matching.and_then(|directive| simple_expression(directive.exp.as_ref()))
            else {
                continue;
            };
            if mirrors_assignment(bind_expression, handler) {
                ctx.warn_with_help(
                    cstr!(
                        "Prefer `v-model` over `:{}` and `@update:{}`",
                        prop_name,
                        prop_name
                    ),
                    &bind.loc,
                    "Replace the matching bind and update listener with v-model",
                );
            }
        }
    }
}

static V_IF_ELSE_KEY_META: RuleMeta = meta!(
    "vue/v-if-else-key",
    "Require keys on repeated conditional components"
);

/// `vue/v-if-else-key`.
pub struct VIfElseKey;

impl Rule for VIfElseKey {
    fn meta(&self) -> &'static RuleMeta {
        &V_IF_ELSE_KEY_META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_conditional_siblings(ctx, &element.children);
    }
}

fn static_argument<'a>(directive: &'a DirectiveNode<'a>) -> Option<&'a str> {
    match &directive.arg {
        Some(ExpressionNode::Simple(argument)) if argument.is_static => Some(argument.content),
        _ => None,
    }
}

fn simple_expression<'a>(expression: Option<&'a ExpressionNode<'a>>) -> Option<&'a str> {
    match expression {
        Some(ExpressionNode::Simple(expression)) => Some(expression.content),
        _ => None,
    }
}

fn mirrors_assignment(binding: &str, handler: &str) -> bool {
    let binding = compact_js(binding);
    let handler = compact_js(handler);
    handler == cstr!("{}=$event", binding)
        || handler.split_once("=>").is_some_and(|(parameter, body)| {
            let parameter = parameter.trim_matches(['(', ')']);
            body == cstr!("{}={}", binding, parameter)
        })
}

fn compact_js(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn has_key(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "key",
        PropNode::Directive(directive) => {
            directive.name == "bind" && static_argument(directive) == Some("key")
        }
    })
}

fn check_conditional_siblings(ctx: &mut LintContext<'_>, children: &[TemplateChildNode<'_>]) {
    let elements: Vec<_> = children
        .iter()
        .filter_map(|child| match child {
            TemplateChildNode::Element(element)
                if !is_native_tag(element.tag)
                    && !matches!(element.tag, "component" | "slot" | "template")
                    && has_conditional_directive(element) =>
            {
                Some(element.as_ref())
            }
            _ => None,
        })
        .collect();
    for element in elements.iter().filter(|element| !has_key(element)) {
        if elements
            .iter()
            .filter(|sibling| sibling.tag == element.tag)
            .count()
            > 1
        {
            ctx.warn_with_help(
                cstr!(
                    "Conditionally rendered repeated component `{}` should have a key",
                    element.tag
                ),
                &element.loc,
                "Add a unique `key` or `:key` to each conditional branch",
            );
        }
    }
}

fn has_conditional_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(prop, PropNode::Directive(directive)
            if matches!(directive.name, "if" | "else-if" | "else"))
    })
}
