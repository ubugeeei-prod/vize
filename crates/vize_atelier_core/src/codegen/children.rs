//! Children, text, comment, and interpolation generation functions.

use crate::rendu::{RenduChildren, RenduOp};
use crate::{Position, RuntimeHelper, TemplateChildNode, TextNode};

use super::children_static::{
    generate_cached_static_children_array, generate_cached_static_element,
    generate_cached_static_vnode, is_static_cacheable_element,
};
use super::context::CodegenContext;
use super::helpers::escape_js_string;
use super::interpolation::push_interpolation_value;
use super::node::{dispatch_rendu_op, generate_node};

/// Generate children array
pub fn generate_children(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    generate_children_inner(ctx, children, false);
}

/// Generate children, forcing array form with createTextVNode (for withDirectives elements)
pub fn generate_children_force_array(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    generate_children_inner(ctx, children, true);
}

/// Emit `,`-separated, newline-prefixed children into an already-open array,
/// driven by the Rendu op stream (#1756).
///
/// The caller emits the surrounding `[` / `]` and manages indentation; each
/// child is dispatched via [`dispatch_rendu_op`] from its op. Directive
/// comments are skipped, exactly as the previous `is_directive_comment` filter
/// did, so component/slot fallback bodies emit the same child set in the same
/// order.
pub(crate) fn emit_children_array_body(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
) {
    for (i, (op, node)) in RenduChildren::new(children).rendered().enumerate() {
        if i > 0 {
            ctx.push(",");
        }
        ctx.newline();
        dispatch_rendu_op(ctx, op, node);
    }
}

/// Check if a child node is a directive comment that should be stripped.
///
/// Classified through the Rendu plate rather than matching the Relief variant
/// directly: `RenduOp::Comment` already carries the `is_directive` fact, so the
/// children traversal reads the same render-semantic classification DOM node
/// dispatch uses. Byte-for-byte equivalent — `is_directive` is exactly
/// `comment.directive.is_some()`.
#[inline]
pub(crate) fn is_directive_comment(child: &TemplateChildNode<'_>) -> bool {
    matches!(
        RenduOp::from_template_child(child),
        RenduOp::Comment {
            is_directive: true,
            ..
        }
    )
}

/// Check if a child renders as inline text (a text literal or interpolation),
/// classified through the Rendu plate (#1756) — reads `RenduOp::Text` /
/// `RenduOp::Interpolation` so text-run grouping uses the same render-semantic
/// classification the node dispatch does. Byte-equivalent.
#[inline]
fn is_text_like(child: &TemplateChildNode<'_>) -> bool {
    matches!(
        RenduOp::from_template_child(child),
        RenduOp::Text { .. } | RenduOp::Interpolation { .. }
    )
}

/// Emit a quoted text literal through the Rendu plate.
///
/// The text content and its source anchor are read from `RenduOp::Text`
/// (#1756) rather than the Relief node. Byte-for-byte equivalent — the op
/// borrows `text.content` and `text.loc`.
fn emit_text_literal(ctx: &mut CodegenContext, text: &TextNode) {
    let RenduOp::Text { content, span } = RenduOp::from_text(text) else {
        unreachable!("from_text returns a Text op");
    };
    ctx.push("\"");
    ctx.record_mapping(&span.start);
    ctx.push(&escape_js_string(content));
    ctx.push("\"");
}

fn generate_children_inner(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    force_array: bool,
) {
    // Filter out directive comments — they are invisible to codegen
    let effective: Vec<&TemplateChildNode<'_>> = children
        .iter()
        .filter(|c| !is_directive_comment(c))
        .collect();

    if effective.is_empty() {
        ctx.push("null");
        return;
    }

    // Check if single text/interpolation child can be inlined (unless forced to array)
    if !force_array && effective.len() == 1 {
        match effective[0] {
            TemplateChildNode::Text(text) => {
                emit_text_literal(ctx, text);
                return;
            }
            TemplateChildNode::Interpolation(interp) => {
                push_interpolation_value(ctx, interp);
                return;
            }
            _ => {}
        }
    }

    // Check if all children are text/interpolation - if so, use string concatenation (unless forced to array)
    let all_text_or_interp = effective.iter().all(|child| is_text_like(child));

    if !force_array && all_text_or_interp {
        // Generate concatenated expression: "text" + _toDisplayString(expr) + "more"
        for (i, child) in effective.iter().enumerate() {
            if i > 0 {
                ctx.push(" + ");
            }
            match child {
                TemplateChildNode::Text(text) => {
                    emit_text_literal(ctx, text);
                }
                TemplateChildNode::Interpolation(interp) => {
                    push_interpolation_value(ctx, interp);
                }
                _ => {}
            }
        }
        return;
    }

    let can_cache_static =
        ctx.static_cache && !ctx.in_v_for && !ctx.has_slot_params() && !ctx.in_cached_static;
    if !force_array
        && can_cache_static
        && !effective.is_empty()
        && effective
            .iter()
            .all(|child| is_static_cacheable_element(child))
    {
        generate_cached_static_children_array(ctx, &effective);
        return;
    }

    ctx.push("[");
    ctx.indent();

    // Group consecutive text/interpolation nodes for merging into single createTextVNode calls
    let mut i = 0;
    let mut first_output = true;
    while i < effective.len() {
        if is_text_like(effective[i]) {
            // Find the run of consecutive text/interpolation nodes
            let start = i;
            while i < effective.len() && is_text_like(effective[i]) {
                i += 1;
            }
            let run = &effective[start..i];

            if !first_output {
                ctx.push(",");
            }
            ctx.newline();
            first_output = false;

            // Check if run has any interpolation (needs TEXT patch flag)
            let has_interp = run
                .iter()
                .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

            let create_text = ctx.helper(RuntimeHelper::CreateText);
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.push(create_text);

            // Single space text: _createTextVNode() with no args (Vue convention)
            let is_single_space = !has_interp
                && run.len() == 1
                && matches!(run[0], TemplateChildNode::Text(t) if t.content == " ");
            if is_single_space {
                ctx.push("()");
                continue;
            }

            ctx.push("(");

            if has_interp {
                // Merge text + interpolation: "text" + _toDisplayString(expr)
                // (a raw `{{{ … }}}` interpolation is concatenated unescaped).
                for (j, child) in run.iter().enumerate() {
                    if j > 0 {
                        ctx.push(" + ");
                    }
                    match child {
                        TemplateChildNode::Text(text) => {
                            emit_text_literal(ctx, text);
                        }
                        TemplateChildNode::Interpolation(interp) => {
                            push_interpolation_value(ctx, interp);
                        }
                        _ => {}
                    }
                }
                ctx.push(", 1 /* TEXT */)");
            } else {
                // Only static text nodes
                for (j, child) in run.iter().enumerate() {
                    if j > 0 {
                        ctx.push(" + ");
                    }
                    if let TemplateChildNode::Text(text) = child {
                        emit_text_literal(ctx, text);
                    }
                }
                ctx.push(")");
            }
        } else {
            if !first_output {
                ctx.push(",");
            }
            ctx.newline();
            first_output = false;
            if !force_array && can_cache_static && is_static_cacheable_element(effective[i]) {
                if let TemplateChildNode::Element(el) = effective[i] {
                    generate_cached_static_element(ctx, el);
                }
            } else if ctx.in_cached_static && is_static_cacheable_element(effective[i]) {
                // Plain descendant inside an already-cached static subtree.
                if let TemplateChildNode::Element(el) = effective[i] {
                    generate_cached_static_vnode(ctx, el, false);
                }
            } else {
                generate_node(ctx, effective[i]);
            }
            i += 1;
        }
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("]");
}

/// Generate text node
pub fn generate_text(ctx: &mut CodegenContext, content: &str, start: &Position) {
    let helper = ctx.helper(RuntimeHelper::CreateText);
    ctx.use_helper(RuntimeHelper::CreateText);
    ctx.push(helper);
    // Single space text: _createTextVNode() with no args (Vue convention)
    if content == " " {
        ctx.push("()");
    } else {
        ctx.push("(\"");
        // Anchor the generated string literal back to the text node's source
        // position, just inside the opening quote. No-op without `source_map`.
        ctx.record_mapping(start);
        ctx.push(&escape_js_string(content));
        ctx.push("\")");
    }
}

/// Generate comment node
///
/// Directive comments (`@vize:` prefix) are stripped from output.
pub fn generate_comment(ctx: &mut CodegenContext, content: &str, start: &Position) {
    let helper = ctx.helper(RuntimeHelper::CreateComment);
    ctx.use_helper(RuntimeHelper::CreateComment);
    ctx.push(helper);
    ctx.push("(\"");
    // Anchor the generated comment string back to the comment node's source
    // position, just inside the opening quote. No-op without `source_map`.
    ctx.record_mapping(start);
    ctx.push(&escape_js_string(content));
    ctx.push("\")");
}
