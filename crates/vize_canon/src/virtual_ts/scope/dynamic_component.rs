//! `<component :is="expr">` whose `expr` is not a component identifier.
//!
//! Croquis records such a usage under a synthetic name (see
//! `vize_croquis::drawer::dynamic_component_alias`); the template scope
//! declares that name as a `const` initialized with the authored expression,
//! so the usage's props, listeners and slots are checked against
//! `typeof <alias>` exactly like a named component's. A union of constructors
//! (`cond ? Foo : Bar`) then types each prop from the members that declare it,
//! which is how `vue-tsc` checks the same element.

use vize_carton::{String, append};
use vize_croquis::{Croquis, drawer::is_dynamic_component_alias};
use vize_relief::{ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode};

/// A named value of the built-in `component`, not an authored registry tag.
/// Match the existing Croquis retained-AST authority without reparsing.
pub(crate) fn is_owned_named_dynamic_component(
    template_ast: Option<&RootNode<'_>>,
    usage: &vize_croquis::croquis::ComponentUsage,
) -> bool {
    owned_named_dynamic_component_ast(template_ast, usage).is_some()
}

pub(super) fn owned_named_dynamic_component_ast<'a>(
    template_ast: Option<&'a RootNode<'a>>,
    usage: &vize_croquis::croquis::ComponentUsage,
) -> Option<&'a oxc_ast::ast::Expression<'a>> {
    let ExpressionNode::Simple(expression) = is_expression_node(template_ast?, usage.start)? else {
        return None;
    };
    let js = expression.js_ast?;
    (js.raw == expression.content
        && usage.name == expression.content.trim()
        && matches!(
            js.ast,
            oxc_ast::ast::Expression::Identifier(_)
                | oxc_ast::ast::Expression::StaticMemberExpression(_)
        ))
    .then_some(js.ast)
}

/// Prove the usage is the generated alias of its authored `<component :is>`.
/// Reserved-looking authored static tags must still receive registry checks.
pub(crate) fn is_owned_dynamic_component_alias(
    template_ast: Option<&RootNode<'_>>,
    usage: &vize_croquis::croquis::ComponentUsage,
) -> bool {
    is_dynamic_component_alias(usage.name.as_str())
        && usage.name == vize_croquis::drawer::dynamic_component_alias(usage.start)
        && template_ast.is_some_and(|root| is_expression(root, usage.start).is_some())
}

pub(super) fn emit_dynamic_component_aliases(
    ts: &mut String,
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
) {
    let Some(root) = template_ast else {
        return;
    };
    for usage in vize_croquis::facts::component_usage_list(summary) {
        if !is_dynamic_component_alias(usage.name.as_str()) {
            continue;
        }
        let Some(expression) = is_expression(root, usage.start) else {
            continue;
        };
        // The authored `v-bind` statement owns the expression's own
        // diagnostics; the alias only has to carry its type.
        append!(
            *ts,
            "  // @ts-ignore Inference-only alias; the authored v-bind statement owns diagnostics.\n  const {} = ({});\n",
            usage.name,
            expression.trim()
        );
    }
}

/// The `:is` expression of the `<component>` element starting at `start`.
fn is_expression<'a>(root: &'a RootNode<'a>, start: u32) -> Option<&'a str> {
    match is_expression_node(root, start)? {
        ExpressionNode::Simple(simple) => Some(simple.content),
        ExpressionNode::Compound(compound) => Some(compound.loc.span.slice(root.source)),
    }
}

fn is_expression_node<'a>(root: &'a RootNode<'a>, start: u32) -> Option<&'a ExpressionNode<'a>> {
    let mut found = None;
    for child in root.children.iter() {
        visit(child, start, &mut found);
        if found.is_some() {
            break;
        }
    }
    found
}

fn visit<'a>(
    node: &'a TemplateChildNode<'a>,
    start: u32,
    found: &mut Option<&'a ExpressionNode<'a>>,
) {
    match node {
        TemplateChildNode::Element(element) => {
            if element.loc.span.start == start {
                *found = (element.tag == "component")
                    .then(|| is_directive_expression(element))
                    .flatten();
                return;
            }
            for child in element.children.iter() {
                visit(child, start, found);
                if found.is_some() {
                    return;
                }
            }
        }
        TemplateChildNode::If(node) => {
            for branch in node.branches.iter() {
                for child in branch.children.iter() {
                    visit(child, start, found);
                    if found.is_some() {
                        return;
                    }
                }
            }
        }
        TemplateChildNode::For(node) => {
            for child in node.children.iter() {
                visit(child, start, found);
                if found.is_some() {
                    return;
                }
            }
        }
        _ => {}
    }
}

fn is_directive_expression<'a>(element: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    element.props.iter().find_map(|prop| {
        let PropNode::Directive(directive) = prop else {
            return None;
        };
        if directive.name != "bind"
            || !matches!(
                directive.arg.as_ref(),
                Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == "is"
            )
        {
            return None;
        }
        directive.exp.as_ref()
    })
}
