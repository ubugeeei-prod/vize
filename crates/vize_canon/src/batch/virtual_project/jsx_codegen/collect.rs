//! Walking the lowered JSX tree and gathering every re-emitted unit.
//!
//! Split out of [`super`] so the render pass and the tree walk stay separately
//! readable. The walk collects, in source order, every dynamic (non-static)
//! expression's source text and byte range, plus the two structured forms that
//! need their own scope (`v-for` bodies and scoped-slot bodies) and the semantic
//! component calls that carry the props contract.

use vize_carton::{String as CompactString, ToCompactString};
use vize_relief::{
    ExpressionNode, RootNode, TemplateChildNode,
    elements::PropNode,
    expressions::{CompoundExpressionChild, CompoundExpressionNode},
};

use super::{JsxEmit, JsxExpr, component, slot};
use vize_atelier_jsx::StyleExprSpan;

pub(super) fn collect_root_expressions(root: &RootNode<'_>, out: &mut Vec<JsxEmit>) {
    for child in &root.children {
        collect_child(child, out, None);
    }
}

/// Append a component's `<style scoped>` template-literal interpolations as
/// plain reads.
///
/// The style block is extracted out of the rendered tree (#1495), so its
/// `${expr}` interpolations are recovered separately on
/// [`LoweredRoot::scoped_style_exprs`](vize_atelier_jsx::LoweredRoot). Each
/// references script values in the component scope (`props`, setup-scope
/// bindings, the `Ctx` context), so re-emitting it through the same sink and
/// scope as the root's JSX expressions type-checks it, with its mapping pointing
/// diagnostics back at the original `${…}` byte range (#1497).
///
/// CSS `v-bind(expr)` references are *not* handled here (see the deferral note in
/// the module docs): they live in the cooked CSS text whose offsets no longer map
/// to source bytes, so recovering their spans needs dedicated extraction infra.
pub(super) fn collect_style_expressions(style_exprs: &[StyleExprSpan], out: &mut Vec<JsxEmit>) {
    for style_expr in style_exprs {
        if let Some(expr) = jsx_expr(&style_expr.content, style_expr.start, style_expr.end) {
            out.push(JsxEmit::Expr(expr));
        }
    }
}

/// Collect one child. `host` is the enclosing component's tag expression, which
/// lets a scoped slot type its parameter from that component's declared
/// `$slots`. It is set for a component's children and *forwarded* through the
/// structural `v-if`/`v-for` arms, because JSX control flow inside a component's
/// children lowers into those nodes, so a synthesized `<template v-slot>` can
/// sit under them and still belong to the same component.
pub(super) fn collect_child(
    child: &TemplateChildNode<'_>,
    out: &mut Vec<JsxEmit>,
    host: Option<&JsxExpr>,
) {
    match child {
        TemplateChildNode::Element(element) => {
            // A scoped slot binds its parameter pattern over the slot body, so
            // it owns both the pattern and the body; collecting them through the
            // ordinary prop/child walk would re-emit the pattern as a bare read
            // and evaluate the body outside the scope it introduces (#4042).
            if let Some(host) = host
                && let Some(scope) = slot::collect(element, host)
            {
                out.push(JsxEmit::SlotScope(scope));
                return;
            }
            let semantic_component = component::collect(element);
            let has_semantic_component = semantic_component.is_some();
            let slot_host = semantic_component
                .as_ref()
                .map(|component| component.tag().clone());
            if let Some(component) = semantic_component {
                out.push(JsxEmit::Component(component));
            }
            for prop in &element.props {
                if !has_semantic_component || !component::captures_prop(element, prop) {
                    collect_prop(prop, out);
                }
            }
            for child in &element.children {
                collect_child(child, out, slot_host.as_ref());
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
                    collect_child(child, out, host);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            if let Some(condition) = &branch.condition {
                collect_expression(condition, out);
            }
            for child in &branch.children {
                collect_child(child, out, host);
            }
        }
        TemplateChildNode::For(node) => {
            // The loop body is re-emitted *inside* the `.map()` callback so its
            // aliases (`value`, `key`) bind with their inferred element types,
            // both fixing a spurious "Cannot find name '<alias>'" and checking
            // the body against the real type. The `source` is the iterated value.
            let Some(source) = expr_of(&node.source) else {
                // A static/empty source cannot be iterated meaningfully; fall
                // back to just walking the body so nothing is silently dropped.
                for child in &node.children {
                    collect_child(child, out, host);
                }
                return;
            };
            let mut body = Vec::new();
            for child in &node.children {
                collect_child(child, &mut body, host);
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

pub(super) fn collect_prop(prop: &PropNode<'_>, out: &mut Vec<JsxEmit>) {
    match prop {
        // Static `class="a"` style attributes carry only literal text.
        PropNode::Attribute(_) => {}
        PropNode::Directive(directive) => {
            // `v-model`'s value expression is the binding target: re-emit it as
            // an assignment so a `const`/`readonly`/non-lvalue binding is reported
            // at the binding. Other directive values (`v-show`, `v-if`, custom
            // `v-x:arg={…}`, `v-on` handlers, bound attributes) are plain reads.
            match directive.name {
                "model" => {
                    if let Some(exp) = &directive.exp
                        && let Some(target) = expr_of(exp)
                    {
                        out.push(JsxEmit::ModelTarget(target));
                    }
                }
                // A `v-slot` expression is a binding *pattern*, not a readable
                // value. A scoped slot whose host is known is re-emitted as its
                // own scope by [`slot::collect`], so reaching here means no host
                // was available (e.g. a native-mode dashed tag carrying
                // `v-slots`, which is not a semantic component), and re-emitting
                // the pattern as a read would fabricate `TS2304` (#4042).
                "slot" => {}
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

pub(super) fn collect_expression(expression: &ExpressionNode<'_>, out: &mut Vec<JsxEmit>) {
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

/// Build a [`JsxExpr`] from a dynamic simple [`ExpressionNode`], or `None` when
/// the expression is static or trims to empty (e.g. a directive with no value).
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
                content: content.to_compact_string(),
                start: simple.loc.span.start,
                end: simple.loc.span.end,
            })
        }
        ExpressionNode::Compound(_) => None,
    }
}

/// The static text of a static simple [`ExpressionNode`] (e.g. a `v-slot` name).
pub(super) fn static_text(expression: &ExpressionNode<'_>) -> Option<CompactString> {
    match expression {
        ExpressionNode::Simple(simple) if simple.is_static => {
            Some(simple.content.to_compact_string())
        }
        _ => None,
    }
}

/// Trim `content` and pair it with its byte range, or `None` when empty.
pub(super) fn jsx_expr(content: &str, start: u32, end: u32) -> Option<JsxExpr> {
    let content = content.trim();
    (!content.is_empty()).then(|| JsxExpr {
        content: content.to_compact_string(),
        start,
        end,
    })
}
