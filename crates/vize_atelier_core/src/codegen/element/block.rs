//! Block element generation.
//!
//! Generates elements that open a new block scope using `openBlock()` and
//! `createElementBlock()` / `createBlock()` for efficient patching.

use crate::ast::{
    ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
};
use crate::transforms::v_memo::{get_memo_exp, has_v_memo};

use super::{
    super::{
        children::{generate_children, generate_children_force_array, is_directive_comment},
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
    directives::{
        generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
    },
    helpers::{
        has_custom_directives, has_renderable_props, has_vmodel_directive, has_vshow_directive,
        is_is_prop, is_renderable_prop, is_whitespace_or_comment,
    },
    v_once::generate_v_once_element,
};
use vize_carton::ToCompactString;

/// Generate element as a block
pub fn generate_element_block(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    // Check for v-once directive - handle it specially
    if super::helpers::has_v_once(el) {
        generate_v_once_element(ctx, el);
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

    // Check for custom directives
    let has_custom_dirs = has_custom_directives(el);
    if has_custom_dirs {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.use_helper(RuntimeHelper::ResolveDirective);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }

    // Check for v-model directive on native elements (only if no custom directives)
    let has_vmodel = has_vmodel_directive(el) && !has_custom_dirs;
    if has_vmodel {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }

    // Check for v-show directive (only if no custom directives or vmodel)
    let has_vshow = has_vshow_directive(el) && !has_vmodel && !has_custom_dirs;
    if has_vshow {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.use_helper(RuntimeHelper::VShow);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }

    // Slots are not blocks - handle them separately
    if el.tag_type == ElementType::Slot {
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

        // Generate fallback content if present
        if !el.children.is_empty() {
            ctx.push(", {}, () => [");
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
        } else {
            ctx.push(")");
        }

        // Close withMemo wrapper if v-memo was present (unlikely but safe)
        if let Some(cache_index) = memo_cache_index {
            ctx.push(", _cache, ");
            ctx.push(&cache_index.to_compact_string());
            ctx.push(")");
        }
        return;
    }

    // v-memo on components: use createVNode without block wrapper
    let is_memo_component = memo_cache_index.is_some() && el.tag_type == ElementType::Component;

    if !is_memo_component {
        // Track helpers for preamble
        ctx.use_helper(RuntimeHelper::OpenBlock);

        // Open block wrapper
        ctx.push("(");
        ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
        ctx.push("(), ");
    }

    match el.tag_type {
        ElementType::Element => {
            ctx.use_helper(RuntimeHelper::CreateElementBlock);
            ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
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
            } else if has_renderable_props(el) {
                ctx.push(", ");
                generate_props(ctx, &el.props);
            } else if !el.children.is_empty() || has_patch_info {
                ctx.push(", null");
            }

            // Generate children
            // When props are hoisted and only TEXT flag is set, omit the patch flag
            // (Vue optimizes block elements with hoisted static props)
            // v-memo elements don't need patch flags (memo handles change detection)
            let should_emit_patch_flag = if memo_cache_index.is_some() {
                false
            } else if let Some(flag) = patch_flag {
                !(el.hoisted_props_index.is_some() && flag == 1)
            } else {
                false
            };
            let effective_has_patch_info = has_patch_info && should_emit_patch_flag;
            if !el.children.is_empty() {
                ctx.push(", ");
                // Custom directives and v-memo require array children with createTextVNode
                if has_custom_dirs || memo_cache_index.is_some() {
                    generate_children_force_array(ctx, &el.children);
                } else {
                    generate_children(ctx, &el.children);
                }
            } else if effective_has_patch_info {
                ctx.push(", null");
            }

            // Generate patch flag
            if should_emit_patch_flag {
                if let Some(flag) = patch_flag {
                    ctx.push(", ");
                    ctx.push(&flag.to_compact_string());
                    ctx.push(" /* ");
                    let flag_name = patch_flag_name(flag);
                    ctx.push(&flag_name);
                    ctx.push(" */");
                }
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

            ctx.push("))");

            // Close withDirectives for custom directives
            if has_custom_dirs {
                generate_custom_directives_closing(ctx, el);
            }

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
            if is_memo_component {
                ctx.use_helper(RuntimeHelper::CreateVNode);
                ctx.push(ctx.helper(RuntimeHelper::CreateVNode));
            } else {
                ctx.use_helper(RuntimeHelper::CreateBlock);
                ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
            }
            ctx.push("(");

            // Check for dynamic component (<component :is="..."> or <Component is="...">)
            let is_dynamic_component = el.tag == "component";
            let (dynamic_is, static_is) = if is_dynamic_component {
                // Check for :is="..." (dynamic binding)
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
                // Check for is="..." (static attribute)
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
                // Dynamic component: resolveDynamicComponent(_ctx.component)
                ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                ctx.push("(");
                generate_expression(ctx, is_exp);
                ctx.push(")");
            } else if let Some(component_name) = static_is {
                // Static is: resolveDynamicComponent("componentName")
                ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                ctx.push("(\"");
                ctx.push(component_name);
                ctx.push("\")");
            } else if let Some(builtin) = is_builtin_component(&el.tag) {
                // Check for built-in components (Teleport, KeepAlive, Suspense)
                ctx.use_helper(builtin);
                ctx.push(ctx.helper(builtin));
            } else if ctx.is_component_in_bindings(&el.tag) {
                // In inline mode, components are directly in scope (imported at module level)
                // In function mode, use $setup.ComponentName to access setup bindings
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

            // For components with slot children, remove TEXT flag (1) since text is inside slot
            if has_slot_children(el) {
                if let Some(flag) = patch_flag {
                    let new_flag = flag & !1; // Remove TEXT flag
                    patch_flag = if new_flag > 0 { Some(new_flag) } else { None };
                }
            }

            // Add DYNAMIC_SLOTS flag (1024) if component has dynamic slots
            // KeepAlive always gets DYNAMIC_SLOTS
            if el.tag == "KeepAlive" || el.tag == "keep-alive" || has_dynamic_slots_flag(el) {
                let dynamic_slots_flag = 1024;
                patch_flag = Some(patch_flag.unwrap_or(0) | dynamic_slots_flag);
            }

            let has_patch_info = patch_flag.is_some() || dynamic_props.is_some();

            // Generate props (only if there are renderable props, not just v-show)
            // For dynamic components (<component :is="...">), filter out the `is` prop
            let effective_has_props = if is_dynamic_component {
                // Check if there are renderable props besides `is`
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
                // Teleport, KeepAlive: pass children as array, not slot object
                // (whitespace-only children are skipped to match Vue's behavior)
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
                    // KeepAlive: dynamic components (<component :is="...">) are
                    // rendered as blocks so KeepAlive can track their identity
                    if is_keep_alive {
                        if let TemplateChildNode::Element(child_el) = child {
                            if child_el.tag_type == ElementType::Component
                                && child_el.tag == "component"
                            {
                                generate_element_block(ctx, child_el);
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

            if is_memo_component {
                ctx.push(")");
            } else {
                ctx.push("))");
            }

            // Close withDirectives for custom directives on component
            if has_custom_dirs {
                generate_custom_directives_closing(ctx, el);
            }

            // Close withDirectives for v-show on component
            if has_vshow {
                generate_vshow_closing(ctx, el);
            }
        }
        ElementType::Slot => {
            // Slots don't use blocks - use renderSlot directly
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

            // Generate fallback content if present
            if !el.children.is_empty() {
                ctx.push(", {}, () => [");
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
            } else {
                ctx.push(")");
            }
        }
        ElementType::Template => {
            ctx.use_helper(RuntimeHelper::CreateElementBlock);
            ctx.use_helper(RuntimeHelper::Fragment);
            ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
            ctx.push("(");
            ctx.push(ctx.helper(RuntimeHelper::Fragment));
            ctx.push(", null, ");
            generate_children(ctx, &el.children);
            ctx.push("))");
        }
    }

    // Close withMemo wrapper if v-memo was present
    if let Some(cache_index) = memo_cache_index {
        ctx.push(", _cache, ");
        ctx.push(&cache_index.to_compact_string());
        ctx.push(")");
    }
}
