//! Walking the lowered JSX tree for the editor document.
//!
//! Mirrors `vize_canon`'s batch `jsx_codegen::collect` so the editor's virtual
//! TypeScript matches the type-checker's byte-for-byte.

use vize_atelier_jsx::StyleExprSpan;
use vize_relief::{
    ExpressionNode, RootNode, TemplateChildNode,
    elements::PropNode,
    expressions::{CompoundExpressionChild, CompoundExpressionNode},
};

use super::{JsxEmit, JsxExpr, component, slot};

pub(super) fn collect_root_expressions(
    root: &RootNode<'_>,
    out: &mut Vec<JsxEmit>,
    preserve_components: bool,
) {
    for child in &root.children {
        collect_child(child, out, preserve_components, None);
    }
}

pub(super) fn collect_style_expressions(style_exprs: &[StyleExprSpan], out: &mut Vec<JsxEmit>) {
    for style_expr in style_exprs {
        if let Some(expr) = jsx_expr(&style_expr.content, style_expr.start, style_expr.end) {
            out.push(JsxEmit::Expr(expr));
        }
    }
}

/// Collect one child. `host` is the enclosing component's tag expression, so a
/// scoped slot can type its parameter from that component's declared `$slots`
/// (#4042). It is set for a component's children and *forwarded* through the
/// structural `v-if`/`v-for` arms, because JSX control flow inside a component's
/// children lowers into those nodes, so a synthesized `<template v-slot>` can sit
/// under them and still belong to the same component.
pub(super) fn collect_child(
    child: &TemplateChildNode<'_>,
    out: &mut Vec<JsxEmit>,
    preserve_components: bool,
    host: Option<&JsxExpr>,
) {
    match child {
        TemplateChildNode::Element(element) => {
            if let Some(host) = host
                && let Some(scope) = slot::collect(element, host, preserve_components)
            {
                out.push(JsxEmit::SlotScope(scope));
                return;
            }
            let semantic_component = preserve_components
                .then(|| component::collect(element))
                .flatten();
            let has_semantic_component = semantic_component.is_some();
            let slot_host = semantic_component
                .as_ref()
                .map(|component| component.tag().clone());
            if let Some(component) = semantic_component {
                out.push(JsxEmit::Component(component));
            }
            for prop in &element.props {
                if !has_semantic_component || !component::captures_prop(element, prop) {
                    collect_prop(prop, out, preserve_components);
                }
            }
            for child in &element.children {
                collect_child(child, out, preserve_components, slot_host.as_ref());
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
                    collect_child(child, out, preserve_components, host);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            if let Some(condition) = &branch.condition {
                collect_expression(condition, out);
            }
            for child in &branch.children {
                collect_child(child, out, preserve_components, host);
            }
        }
        TemplateChildNode::For(node) => {
            let Some(source) = expr_of(&node.source) else {
                for child in &node.children {
                    collect_child(child, out, preserve_components, host);
                }
                return;
            };
            let mut body = Vec::new();
            for child in &node.children {
                collect_child(child, &mut body, preserve_components, host);
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

/// Collect one prop.
///
/// `preserve_components` distinguishes the two callers: the generated
/// type-check document (`true`), which must stay byte-for-byte identical to the
/// batch generator, and the structural walk behind semantic tokens and hover
/// (`false`), which wants every authored expression range including binding
/// patterns.
fn collect_prop(prop: &PropNode<'_>, out: &mut Vec<JsxEmit>, preserve_components: bool) {
    match prop {
        // Static `class="a"` style attributes carry only literal text.
        PropNode::Attribute(_) => {}
        PropNode::Directive(directive) => {
            match directive.name.as_str() {
                "model" => {
                    if let Some(exp) = &directive.exp
                        && let Some(target) = expr_of(exp)
                    {
                        out.push(JsxEmit::ModelTarget(target));
                    }
                }
                // A `v-slot` expression is a binding *pattern*, not a readable
                // value. A scoped slot whose host is known is re-emitted as its
                // own scope by [`slot::collect`], so reaching here in a generated
                // document means no host was available (a native-mode dashed tag
                // carrying `v-slots`, which is not a semantic component), where
                // re-emitting the pattern as a read would fabricate `TS2304`
                // (#4042). The structural walk still reports the pattern's range.
                "slot" if preserve_components => {}
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
            push_expr(&simple.content, &simple.loc, out);
        }
        ExpressionNode::Compound(compound) => collect_compound(compound, out),
    }
}

fn collect_compound(compound: &CompoundExpressionNode<'_>, out: &mut Vec<JsxEmit>) {
    for child in &compound.children {
        match child {
            CompoundExpressionChild::Simple(simple) => {
                if !simple.is_static {
                    push_expr(&simple.content, &simple.loc, out);
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
            jsx_expr(&simple.content, simple.loc.span.start, simple.loc.span.end)
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

/// The static text of a static simple [`ExpressionNode`] (e.g. a `v-slot` name).
pub(super) fn static_text(expression: &ExpressionNode<'_>) -> Option<String> {
    match expression {
        ExpressionNode::Simple(simple) if simple.is_static => Some(simple.content.to_string()),
        _ => None,
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
