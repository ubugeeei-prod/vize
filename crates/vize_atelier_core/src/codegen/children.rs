//! Children, text, comment, and interpolation generation functions.

use crate::ast::*;
use crate::transforms::hoist_static::is_static_node;

use super::context::CodegenContext;
use super::element::helpers::has_renderable_props;
use super::expression::generate_expression;
use super::helpers::escape_js_string;
use super::node::generate_node;
use super::props::generate_props;
use vize_carton::ToCompactString;

/// Generate children array
pub fn generate_children(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    generate_children_inner(ctx, children, false);
}

/// Generate children, forcing array form with createTextVNode (for withDirectives elements)
pub fn generate_children_force_array(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    generate_children_inner(ctx, children, true);
}

/// Check if a child node is a directive comment that should be stripped.
#[inline]
pub(crate) fn is_directive_comment(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Comment(c) if c.directive.is_some())
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
                ctx.push("\"");
                ctx.push(&escape_js_string(&text.content));
                ctx.push("\"");
                return;
            }
            TemplateChildNode::Interpolation(interp) => {
                let helper = ctx.helper(RuntimeHelper::ToDisplayString);
                ctx.use_helper(RuntimeHelper::ToDisplayString);
                ctx.push(helper);
                ctx.push("(");
                generate_expression(ctx, &interp.content);
                ctx.push(")");
                return;
            }
            _ => {}
        }
    }

    // Check if all children are text/interpolation - if so, use string concatenation (unless forced to array)
    let all_text_or_interp = effective.iter().all(|child| {
        matches!(
            child,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });

    if !force_array && all_text_or_interp {
        // Generate concatenated expression: "text" + _toDisplayString(expr) + "more"
        for (i, child) in effective.iter().enumerate() {
            if i > 0 {
                ctx.push(" + ");
            }
            match child {
                TemplateChildNode::Text(text) => {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(&text.content));
                    ctx.push("\"");
                }
                TemplateChildNode::Interpolation(interp) => {
                    let helper = ctx.helper(RuntimeHelper::ToDisplayString);
                    ctx.use_helper(RuntimeHelper::ToDisplayString);
                    ctx.push(helper);
                    ctx.push("(");
                    generate_expression(ctx, &interp.content);
                    ctx.push(")");
                }
                _ => {}
            }
        }
        return;
    }

    let can_cache_static = ctx.static_cache && !ctx.in_v_for && !ctx.has_slot_params();
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
        let is_text_like = matches!(
            effective[i],
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        );

        if is_text_like {
            // Find the run of consecutive text/interpolation nodes
            let start = i;
            while i < effective.len()
                && matches!(
                    effective[i],
                    TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
                )
            {
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
                let to_display = ctx.helper(RuntimeHelper::ToDisplayString);
                ctx.use_helper(RuntimeHelper::ToDisplayString);
                for (j, child) in run.iter().enumerate() {
                    if j > 0 {
                        ctx.push(" + ");
                    }
                    match child {
                        TemplateChildNode::Text(text) => {
                            ctx.push("\"");
                            ctx.push(&escape_js_string(&text.content));
                            ctx.push("\"");
                        }
                        TemplateChildNode::Interpolation(interp) => {
                            ctx.push(to_display);
                            ctx.push("(");
                            generate_expression(ctx, &interp.content);
                            ctx.push(")");
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
                        ctx.push("\"");
                        ctx.push(&escape_js_string(&text.content));
                        ctx.push("\"");
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

fn is_static_cacheable_element(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Element(_)) && is_static_node(child)
}

fn generate_cached_static_children_array(
    ctx: &mut CodegenContext,
    children: &[&TemplateChildNode<'_>],
) {
    let cache_index = ctx.next_cache_index();
    ctx.push("[...(_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] || (_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] = [");
    ctx.indent();

    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            ctx.push(",");
        }
        ctx.newline();
        if let TemplateChildNode::Element(el) = child {
            generate_cached_static_vnode(ctx, el);
        }
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("]))]");
}

fn generate_cached_static_element(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let cache_index = ctx.next_cache_index();
    ctx.push("_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] || (_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] = ");
    generate_cached_static_vnode(ctx, el);
    ctx.push(")");
}

fn generate_cached_static_vnode(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    ctx.use_helper(RuntimeHelper::CreateElementVNode);
    ctx.push(ctx.helper(RuntimeHelper::CreateElementVNode));
    ctx.push("(\"");
    ctx.push(&el.tag);
    ctx.push("\"");

    if has_renderable_props(el) {
        ctx.push(", ");
        ctx.props_is_plain_element = true;
        generate_props(ctx, &el.props);
        ctx.props_is_plain_element = false;
    } else {
        ctx.push(", null");
    }

    if !el.children.is_empty() {
        ctx.push(", ");
        generate_children(ctx, &el.children);
    } else {
        ctx.push(", null");
    }

    ctx.push(", -1 /* CACHED */)");
}

/// Generate text node
pub fn generate_text(ctx: &mut CodegenContext, text: &TextNode) {
    let helper = ctx.helper(RuntimeHelper::CreateText);
    ctx.use_helper(RuntimeHelper::CreateText);
    ctx.push(helper);
    // Single space text: _createTextVNode() with no args (Vue convention)
    if text.content == " " {
        ctx.push("()");
    } else {
        ctx.push("(\"");
        ctx.push(&escape_js_string(&text.content));
        ctx.push("\")");
    }
}

/// Generate comment node
///
/// Directive comments (`@vize:` prefix) are stripped from output.
pub fn generate_comment(ctx: &mut CodegenContext, comment: &CommentNode) {
    // Strip @vize: directive comments from build output
    if comment.directive.is_some() {
        return;
    }
    let helper = ctx.helper(RuntimeHelper::CreateComment);
    ctx.use_helper(RuntimeHelper::CreateComment);
    ctx.push(helper);
    ctx.push("(\"");
    ctx.push(&escape_js_string(&comment.content));
    ctx.push("\")");
}

/// Generate interpolation
pub fn generate_interpolation(ctx: &mut CodegenContext, interp: &InterpolationNode<'_>) {
    let helper = ctx.helper(RuntimeHelper::ToDisplayString);
    ctx.use_helper(RuntimeHelper::ToDisplayString);
    ctx.push(helper);
    ctx.push("(");
    generate_expression(ctx, &interp.content);
    ctx.push(")");
}
