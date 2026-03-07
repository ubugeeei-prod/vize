//! Inline (non-block) element generation.
//!
//! Generates elements that are not block roots, using `createElementVNode()`
//! and `createVNode()` instead of their block counterparts.

use crate::{
    ast::*,
    transforms::v_memo::{get_memo_exp, has_v_memo},
};

use super::{
    super::{
        children::{generate_children, is_directive_comment},
        context::CodegenContext,
        expression::generate_expression,
        helpers::is_builtin_component,
        node::generate_node,
        patch_flag::{
            calculate_element_patch_info, calculate_element_patch_info_skip_is, patch_flag_name,
        },
        props::generate_props,
        slots::{generate_slots, has_dynamic_slots_flag, has_slot_children},
    },
    directives::{generate_vmodel_closing, generate_vshow_closing},
    helpers::{
        has_renderable_props, has_vmodel_directive, has_vshow_directive, is_is_prop,
        is_renderable_prop, is_whitespace_or_comment,
    },
};
use vize_carton::ToCompactString;

/// Generate element code (non-block)
pub fn generate_element(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    // Check for v-once directive - handle it specially with cache
    if super::helpers::has_v_once(el) {
        super::v_once::generate_v_once_element(ctx, el);
        return;
    }

    // Check for v-memo directive - wrap with memoization
    let memo_cache_index = if has_v_memo(el) {
        if let Some(memo_exp) = get_memo_exp(el) {
            let cache_index = ctx.next_cache_index();
            ctx.use_helper(RuntimeHelper::WithMemo);
            ctx.push(ctx.helper(RuntimeHelper::WithMemo));
            ctx.push("(");
            // Generate the memo deps expression with proper _ctx. prefixing
            generate_expression(ctx, memo_exp);
            ctx.push(", () => ");
            Some(cache_index)
        } else {
            None
        }
    } else {
        None
    };

    match el.tag_type {
        ElementType::Element => {
            // Check for v-model directive on native elements (only if no v-show)
            let has_vmodel = has_vmodel_directive(el);
            if has_vmodel {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            // Check for v-show directive (only if no v-model)
            let has_vshow = has_vshow_directive(el) && !has_vmodel;
            if has_vshow {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.use_helper(RuntimeHelper::VShow);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            ctx.push_pure();
            let helper = ctx.helper(RuntimeHelper::CreateElementVNode);
            ctx.use_helper(RuntimeHelper::CreateElementVNode);
            ctx.push(helper);
            ctx.push("(\"");
            ctx.push(&el.tag);
            ctx.push("\"");

            // Calculate patch flag and dynamic props
            let (patch_flag, dynamic_props) = calculate_element_patch_info(
                el,
                ctx.options.binding_metadata.as_ref(),
                ctx.options.cache_handlers,
            );
            let has_patch_info = patch_flag.is_some() || dynamic_props.is_some();

            // Generate props (only if there are renderable props, not just v-show)
            // If props are hoisted, use the hoisted reference
            if let Some(hoisted_index) = el.hoisted_props_index {
                ctx.push(", _hoisted_");
                ctx.push(&hoisted_index.to_compact_string());
            } else if super::helpers::has_renderable_props(el) {
                ctx.push(", ");
                generate_props(ctx, &el.props);
            } else if !el.children.is_empty() || has_patch_info {
                ctx.push(", null");
            }

            // Generate children
            if !el.children.is_empty() {
                ctx.push(", ");
                generate_children(ctx, &el.children);
            } else if has_patch_info {
                ctx.push(", null");
            }

            // Generate patch flag
            if let Some(flag) = patch_flag {
                ctx.push(", ");
                ctx.push(&flag.to_compact_string());
                ctx.push(" /* ");
                let flag_name = patch_flag_name(flag);
                ctx.push(&flag_name);
                ctx.push(" */");
            }

            // Generate dynamic props
            if let Some(props) = dynamic_props {
                ctx.push(", [");
                for (i, prop) in props.iter().enumerate() {
                    if i > 0 {
                        ctx.push(", ");
                    }
                    ctx.push("\"");
                    ctx.push(prop);
                    ctx.push("\"");
                }
                ctx.push("]");
            }

            ctx.push(")");

            // Close withDirectives for v-model
            if has_vmodel {
                generate_vmodel_closing(ctx, el);
            }

            // Close withDirectives for v-show
            if has_vshow {
                generate_vshow_closing(ctx, el);
            }
        }
        ElementType::Component => {
            // Support v-show on non-block components:
            // _withDirectives(_createVNode(...), [[_vShow, expr]])
            let has_vshow = has_vshow_directive(el);
            if has_vshow {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.use_helper(RuntimeHelper::VShow);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            ctx.push_pure();
            let helper = ctx.helper(RuntimeHelper::CreateVNode);
            ctx.use_helper(RuntimeHelper::CreateVNode);
            ctx.push(helper);
            ctx.push("(");

            // Check for dynamic component (<component :is="..."> or <Component is="...">)
            let is_dynamic_component = el.tag == "component" || el.tag == "Component";
            let (dynamic_is, static_is) = if is_dynamic_component {
                let dynamic = el.props.iter().find_map(|p| {
                    if let PropNode::Directive(dir) = p {
                        if dir.name == "bind" {
                            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                                if arg.content == "is" {
                                    return dir.exp.as_ref();
                                }
                            }
                        }
                    }
                    None
                });
                let static_val = el.props.iter().find_map(|p| {
                    if let PropNode::Attribute(attr) = p {
                        if attr.name == "is" {
                            return attr.value.as_ref().map(|v| v.content.as_str());
                        }
                    }
                    None
                });
                (dynamic, static_val)
            } else {
                (None, None)
            };

            if let Some(is_exp) = dynamic_is {
                ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                ctx.push("(");
                generate_expression(ctx, is_exp);
                ctx.push(")");
            } else if let Some(component_name) = static_is {
                ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                ctx.push("(\"");
                ctx.push(component_name);
                ctx.push("\")");
            } else if let Some(builtin) = is_builtin_component(&el.tag) {
                ctx.use_helper(builtin);
                ctx.push(ctx.helper(builtin));
            } else if ctx.is_component_in_bindings(&el.tag) {
                if !ctx.options.inline {
                    ctx.push("$setup.");
                }
                ctx.push(&el.tag);
            } else {
                ctx.push("_component_");
                ctx.push(&el.tag.replace('-', "_"));
            }

            // Calculate patch flag and dynamic props for component
            // For dynamic components, skip the :is binding from patch flag calculation
            let (mut patch_flag, dynamic_props) = if is_dynamic_component {
                calculate_element_patch_info_skip_is(
                    el,
                    ctx.options.binding_metadata.as_ref(),
                    ctx.options.cache_handlers,
                )
            } else {
                calculate_element_patch_info(
                    el,
                    ctx.options.binding_metadata.as_ref(),
                    ctx.options.cache_handlers,
                )
            };

            // Slot content is patched through the slot object, so the component vnode
            // itself should not keep the TEXT flag.
            if has_slot_children(el) {
                if let Some(flag) = patch_flag {
                    let new_flag = flag & !1;
                    patch_flag = if new_flag > 0 { Some(new_flag) } else { None };
                }
            }

            // KeepAlive always needs DYNAMIC_SLOTS. Other components need it when
            // slot structure is dynamic.
            if el.tag == "KeepAlive" || el.tag == "keep-alive" || has_dynamic_slots_flag(el) {
                patch_flag = Some(patch_flag.unwrap_or(0) | 1024);
            }

            let has_patch_info = patch_flag.is_some() || dynamic_props.is_some();

            // Generate props -- for dynamic components, filter out the `is` prop
            let effective_has_props = if is_dynamic_component {
                el.props
                    .iter()
                    .any(|p| !is_is_prop(p) && is_renderable_prop(p))
            } else {
                has_renderable_props(el)
            };
            if effective_has_props {
                ctx.push(", ");
                if is_dynamic_component {
                    ctx.skip_is_prop = true;
                }
                // Components: skip scope_id in props -- Vue runtime applies it via __scopeId
                let prev_skip_scope_id = ctx.skip_scope_id;
                ctx.skip_scope_id = true;
                generate_props(ctx, &el.props);
                ctx.skip_scope_id = prev_skip_scope_id;
                ctx.skip_is_prop = false;
            } else if !el.children.is_empty() || has_patch_info {
                ctx.push(", null");
            }

            // Generate children/slots - use slot generation for component children
            if has_slot_children(el) {
                ctx.push(", ");
                generate_slots(ctx, el);
            } else if el.children.iter().any(|c| !is_whitespace_or_comment(c)) {
                let is_keep_alive = matches!(el.tag.as_str(), "KeepAlive" | "keep-alive");
                ctx.push(", [");
                ctx.indent();
                let filtered: Vec<_> = el
                    .children
                    .iter()
                    .filter(|c| !is_directive_comment(c))
                    .collect();
                for (i, child) in filtered.iter().enumerate() {
                    if i > 0 {
                        ctx.push(",");
                    }
                    ctx.newline();
                    if is_keep_alive {
                        if let TemplateChildNode::Element(child_el) = child {
                            if child_el.tag_type == ElementType::Component
                                && child_el.tag == "component"
                            {
                                super::block::generate_element_block(ctx, child_el);
                                continue;
                            }
                        }
                    }
                    generate_node(ctx, child);
                }
                ctx.deindent();
                ctx.newline();
                ctx.push("]");
            } else if has_patch_info {
                ctx.push(", null");
            }

            if let Some(flag) = patch_flag {
                ctx.push(", ");
                ctx.push(&flag.to_compact_string());
                ctx.push(" /* ");
                let flag_name = patch_flag_name(flag);
                ctx.push(&flag_name);
                ctx.push(" */");
            }

            if let Some(props) = dynamic_props {
                ctx.push(", [");
                for (i, prop) in props.iter().enumerate() {
                    if i > 0 {
                        ctx.push(", ");
                    }
                    ctx.push("\"");
                    ctx.push(prop);
                    ctx.push("\"");
                }
                ctx.push("]");
            }

            ctx.push(")");

            // Close withDirectives for v-show on component
            if has_vshow {
                generate_vshow_closing(ctx, el);
            }
        }
        ElementType::Slot => {
            let helper = ctx.helper(RuntimeHelper::RenderSlot);
            ctx.use_helper(RuntimeHelper::RenderSlot);
            ctx.push(helper);
            ctx.push("(_ctx.$slots, ");
            // Get slot name from props
            let slot_name = el
                .props
                .iter()
                .find_map(|p| match p {
                    PropNode::Attribute(attr) if attr.name == "name" => {
                        attr.value.as_ref().map(|v| v.content.as_str())
                    }
                    _ => None,
                })
                .unwrap_or("default");
            ctx.push("\"");
            ctx.push(slot_name);
            ctx.push("\"");

            // Generate slot props (excluding 'name' attribute)
            let slot_props: Vec<_> = el
                .props
                .iter()
                .filter(|p| match p {
                    PropNode::Attribute(attr) => attr.name != "name",
                    PropNode::Directive(_) => true,
                })
                .collect();

            // Generate fallback content if present
            // Slots: skip scope_id in props -- not a real rendered element
            let prev_skip_scope_id = ctx.skip_scope_id;
            ctx.skip_scope_id = true;
            if !el.children.is_empty() {
                // If we have children but no props, pass empty object
                if slot_props.is_empty() {
                    ctx.push(", {}");
                } else {
                    ctx.push(", ");
                    generate_props(ctx, &el.props);
                }
                ctx.push(", () => [");
                ctx.skip_scope_id = prev_skip_scope_id;
                ctx.indent();
                let filtered: Vec<_> = el
                    .children
                    .iter()
                    .filter(|c| !is_directive_comment(c))
                    .collect();
                for (i, child) in filtered.iter().enumerate() {
                    if i > 0 {
                        ctx.push(",");
                    }
                    ctx.newline();
                    generate_node(ctx, child);
                }
                ctx.deindent();
                ctx.newline();
                ctx.push("])");
            } else if !slot_props.is_empty() {
                ctx.push(", ");
                generate_props(ctx, &el.props);
                ctx.skip_scope_id = prev_skip_scope_id;
                ctx.push(")");
            } else {
                ctx.skip_scope_id = prev_skip_scope_id;
                ctx.push(")");
            }
        }
        ElementType::Template => {
            // Template elements render their children directly
            let filtered: Vec<_> = el
                .children
                .iter()
                .filter(|c| !is_directive_comment(c))
                .collect();
            if filtered.len() == 1 {
                generate_node(ctx, filtered[0]);
            } else {
                ctx.push("[");
                for (i, child) in filtered.iter().enumerate() {
                    if i > 0 {
                        ctx.push(", ");
                    }
                    generate_node(ctx, child);
                }
                ctx.push("]");
            }
        }
    }

    // Close withMemo wrapper if v-memo was present
    if let Some(cache_index) = memo_cache_index {
        ctx.push(", _cache, ");
        ctx.push(&cache_index.to_compact_string());
        ctx.push(")");
    }
}
