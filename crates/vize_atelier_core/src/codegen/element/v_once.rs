//! v-once element generation.
//!
//! Generates cache-wrapped elements for the `v-once` directive, which
//! renders elements once and caches them for subsequent renders.

use crate::ast::{
    ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
};

use super::super::{
    context::CodegenContext,
    expression::generate_expression,
    helpers::{escape_js_string, to_valid_asset_identifier},
    node::generate_node,
    patch_flag::{calculate_element_patch_info, patch_flag_name},
    props::is_supported_directive,
};
use vize_carton::ToCompactString;

/// Generate v-once element with cache wrapper
pub fn generate_v_once_element(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let cache_index = ctx.next_cache_index();

    ctx.use_helper(RuntimeHelper::SetBlockTracking);

    // _cache[0] || (...)
    ctx.push("_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] || (");
    ctx.indent();
    ctx.newline();

    // _setBlockTracking(-1, true),
    ctx.push(ctx.helper(RuntimeHelper::SetBlockTracking));
    ctx.push("(-1, true),");
    ctx.newline();

    // (_cache[0] = _createElementVNode(...)).cacheIndex = 0,
    ctx.push("(_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] = ");

    // Generate the element content
    if el.tag_type == ElementType::Component {
        ctx.use_helper(RuntimeHelper::CreateVNode);
        ctx.use_helper(RuntimeHelper::ResolveComponent);
        ctx.push(ctx.helper(RuntimeHelper::CreateVNode));
        ctx.push("(");
        ctx.push(&to_valid_asset_identifier("component", &el.tag));
        ctx.push(")");
    } else {
        ctx.use_helper(RuntimeHelper::CreateElementVNode);
        ctx.push(ctx.helper(RuntimeHelper::CreateElementVNode));
        ctx.push("(\"");
        ctx.push(&el.tag);
        ctx.push("\"");

        // Generate props (excluding v-once)
        let has_props = el.props.iter().any(|p| match p {
            PropNode::Directive(dir) => dir.name != "once" && is_supported_directive(dir),
            PropNode::Attribute(_) => true,
        });

        if has_props {
            ctx.push(", ");
            generate_v_once_props(ctx, el);
        } else if !el.children.is_empty() {
            ctx.push(", null");
        }

        // Generate children
        if !el.children.is_empty() {
            ctx.push(", [");
            ctx.indent();
            for (i, child) in el.children.iter().enumerate() {
                if i > 0 {
                    ctx.push(",");
                }
                ctx.newline();
                generate_v_once_child(ctx, child);
            }
            ctx.deindent();
            ctx.newline();
            ctx.push("]");
        }

        // v-once still needs patch flag for dynamic bindings (class/style)
        let (patch_flag, _) = calculate_element_patch_info(
            el,
            ctx.options.binding_metadata.as_ref(),
            ctx.cache_handlers_in_current_scope(),
        );
        if let Some(flag) = patch_flag {
            // Only emit CLASS/STYLE flags for v-once, ignore PROPS
            let filtered_flag = flag & (2 | 4); // CLASS | STYLE
            if filtered_flag > 0 {
                if el.children.is_empty() && !has_props {
                    ctx.push(", null");
                }
                ctx.push(", ");
                ctx.push(&filtered_flag.to_compact_string());
                ctx.push(" /* ");
                let flag_name = patch_flag_name(filtered_flag);
                ctx.push(&flag_name);
                ctx.push(" */");
            }
        }
        ctx.push(")");
    }

    ctx.push(").cacheIndex = ");
    ctx.push(&cache_index.to_compact_string());
    ctx.push(",");
    ctx.newline();

    // _setBlockTracking(1),
    ctx.push(ctx.helper(RuntimeHelper::SetBlockTracking));
    ctx.push("(1),");
    ctx.newline();

    // _cache[0]
    ctx.push("_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("]");

    ctx.deindent();
    ctx.newline();
    ctx.push(")");
}

/// Generate props for v-once element (excludes v-once directive)
pub fn generate_v_once_props(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    ctx.push("{");
    ctx.indent();

    let mut first = true;
    for prop in &el.props {
        match prop {
            PropNode::Directive(dir) if dir.name == "once" => continue,
            PropNode::Directive(dir) if dir.name == "bind" => {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    if !first {
                        ctx.push(",");
                    }
                    first = false;
                    ctx.newline();

                    if arg.content == "class" {
                        ctx.use_helper(RuntimeHelper::NormalizeClass);
                        ctx.push("class: ");
                        ctx.push(ctx.helper(RuntimeHelper::NormalizeClass));
                        ctx.push("(");
                        if let Some(exp) = &dir.exp {
                            generate_expression(ctx, exp);
                        }
                        ctx.push(")");
                    } else if arg.content == "style" {
                        ctx.use_helper(RuntimeHelper::NormalizeStyle);
                        ctx.push("style: ");
                        ctx.push(ctx.helper(RuntimeHelper::NormalizeStyle));
                        ctx.push("(");
                        if let Some(exp) = &dir.exp {
                            generate_expression(ctx, exp);
                        }
                        ctx.push(")");
                    } else {
                        ctx.push(&arg.content);
                        ctx.push(": ");
                        if let Some(exp) = &dir.exp {
                            generate_expression(ctx, exp);
                        }
                    }
                }
            }
            PropNode::Attribute(attr) => {
                if !first {
                    ctx.push(",");
                }
                first = false;
                ctx.newline();
                ctx.push(&attr.name);
                ctx.push(": ");
                if let Some(value) = &attr.value {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(&value.content));
                    ctx.push("\"");
                } else {
                    ctx.push("true");
                }
            }
            _ => {}
        }
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

/// Generate child node for v-once (uses createTextVNode instead of interpolation)
pub fn generate_v_once_child(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>) {
    match node {
        TemplateChildNode::Text(text) => {
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.push(ctx.helper(RuntimeHelper::CreateText));
            ctx.push("(\"");
            ctx.push(&escape_js_string(&text.content));
            ctx.push("\")");
        }
        TemplateChildNode::Interpolation(interp) => {
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.use_helper(RuntimeHelper::ToDisplayString);
            ctx.push(ctx.helper(RuntimeHelper::CreateText));
            ctx.push("(");
            ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
            ctx.push("(");
            generate_expression(ctx, &interp.content);
            ctx.push("), 1 /* TEXT */)");
        }
        _ => generate_node(ctx, node),
    }
}
