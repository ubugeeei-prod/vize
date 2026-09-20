//! Structural JSX expression ranges for semantic tokens and hover.

use vize_atelier_jsx::StyleExprSpan;
use vize_relief::{
    ExpressionNode, RootNode, TemplateChildNode,
    elements::PropNode,
    expressions::{CompoundExpressionChild, CompoundExpressionNode},
};

use super::{JsxEmit, JsxExpr};

pub(super) fn collect_root_expressions(root: &RootNode<'_>, out: &mut Vec<JsxEmit>) {
    for child in &root.children {
        collect_child(child, out);
    }
}

pub(super) fn collect_style_expressions(style_exprs: &[StyleExprSpan], out: &mut Vec<JsxEmit>) {
    for style_expr in style_exprs {
        if let Some(expr) = jsx_expr(&style_expr.content, style_expr.start, style_expr.end) {
            out.push(JsxEmit::Expr(expr));
        }
    }
}

pub(super) fn collect_child(child: &TemplateChildNode<'_>, out: &mut Vec<JsxEmit>) {
    match child {
        TemplateChildNode::Element(element) => {
            for prop in &element.props {
                collect_prop(prop, out);
            }
            for child in &element.children {
                collect_child(child, out);
            }
        }
        TemplateChildNode::Interpolation(interpolation) => {
            collect_expression(&interpolation.content, out);
        }
        TemplateChildNode::CompoundExpression(compound) => {
            collect_compound(compound, out);
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                if let Some(condition) = &branch.condition {
                    collect_expression(condition, out);
                }
                for child in &branch.children {
                    collect_child(child, out);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            if let Some(condition) = &branch.condition {
                collect_expression(condition, out);
            }
            for child in &branch.children {
                collect_child(child, out);
            }
        }
        TemplateChildNode::For(node) => {
            let Some(source) = expr_of(&node.source) else {
                for child in &node.children {
                    collect_child(child, out);
                }
                return;
            };
            let mut body = Vec::new();
            for child in &node.children {
                collect_child(child, &mut body);
            }
            out.push(JsxEmit::ForScope {
                source,
                value_alias: node.value_alias.as_ref().and_then(alias_expr),
                key_alias: node.key_alias.as_ref().and_then(alias_expr),
                body,
            });
        }
        TemplateChildNode::TextCall(node) => {
            collect_text_call(&node.content, out);
        }
        TemplateChildNode::Text(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Hoisted(_) => {}
    }
}

fn collect_text_call(content: &vize_relief::TextCallContent<'_>, out: &mut Vec<JsxEmit>) {
    use vize_relief::TextCallContent;
    match content {
        TextCallContent::Interpolation(interpolation) => {
            collect_expression(&interpolation.content, out);
        }
        TextCallContent::Compound(compound) => collect_compound(compound, out),
        TextCallContent::Text(_) => {}
    }
}

fn collect_prop(prop: &PropNode<'_>, out: &mut Vec<JsxEmit>) {
    match prop {
        // Static `class="a"` style attributes carry only literal text.
        PropNode::Attribute(_) => {}
        PropNode::Directive(directive) => {
            match directive.name {
                "model" => {
                    if let Some(exp) = &directive.exp
                        && let Some(target) = expr_of(exp)
                    {
                        out.push(JsxEmit::ModelTarget(target));
                    }
                }
                _ => {
                    if let Some(exp) = &directive.exp {
                        collect_expression(exp, out);
                    }
                }
            }
            if let Some(arg) = &directive.arg {
                collect_expression(arg, out);
            }
        }
    }
}

fn collect_expression(expression: &ExpressionNode<'_>, out: &mut Vec<JsxEmit>) {
    match expression {
        ExpressionNode::Simple(simple) => {
            if simple.is_static {
                return;
            }
            push_expr(simple.content, &simple.loc, out);
        }
        ExpressionNode::Compound(compound) => collect_compound(compound, out),
    }
}

fn collect_compound(compound: &CompoundExpressionNode<'_>, out: &mut Vec<JsxEmit>) {
    for child in &compound.children {
        match child {
            CompoundExpressionChild::Simple(simple) => {
                if !simple.is_static {
                    push_expr(simple.content, &simple.loc, out);
                }
            }
            CompoundExpressionChild::Compound(compound) => collect_compound(compound, out),
            CompoundExpressionChild::Interpolation(interpolation) => {
                collect_expression(&interpolation.content, out);
            }
            CompoundExpressionChild::Text(_)
            | CompoundExpressionChild::String(_)
            | CompoundExpressionChild::Symbol(_) => {}
        }
    }
}

fn push_expr(content: &str, loc: &vize_relief::SourceLocation, out: &mut Vec<JsxEmit>) {
    if let Some(expr) = jsx_expr(content, loc.span.start, loc.span.end) {
        out.push(JsxEmit::Expr(expr));
    }
}

pub(super) fn expr_of(expression: &ExpressionNode<'_>) -> Option<JsxExpr> {
    match expression {
        ExpressionNode::Simple(simple) if !simple.is_static => {
            jsx_expr(simple.content, simple.loc.span.start, simple.loc.span.end)
        }
        _ => None,
    }
}

/// The source text of a binding pattern stored as a simple expression (a
/// `v-for` alias or a `v-slot` scope pattern), or `None` when absent.
pub(super) fn alias_expr(alias: &ExpressionNode<'_>) -> Option<JsxExpr> {
    match alias {
        ExpressionNode::Simple(simple) => {
            let content = simple.content.trim();
            (!content.is_empty()).then(|| JsxExpr {
                content: content.to_string(),
                start: simple.loc.span.start,
                end: simple.loc.span.end,
            })
        }
        ExpressionNode::Compound(_) => None,
    }
}

pub(super) fn jsx_expr(content: &str, start: u32, end: u32) -> Option<JsxExpr> {
    let content = content.trim();
    (!content.is_empty()).then(|| JsxExpr {
        content: content.to_string(),
        start,
        end,
    })
}
