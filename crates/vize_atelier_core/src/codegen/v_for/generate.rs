//! v-for item generation.
//!
//! Generates code for individual items within a v-for loop,
//! including props merging, key handling, and block wrapping.

use crate::{
    ast::*,
    transforms::v_memo::{get_memo_exp, has_v_memo},
};

use super::super::{
    children::{generate_children, generate_children_force_array},
    context::CodegenContext,
    element::{generate_vshow_closing, has_vshow_directive},
    expression::generate_expression,
    helpers::{escape_js_string, is_builtin_component},
    node::generate_node,
    patch_flag::{calculate_element_patch_info, patch_flag_name},
    slots::{generate_slots, has_slot_children},
};
use super::helpers::{get_element_key, has_other_props, should_skip_prop};
use vize_carton::ToCompactString;

/// Generate item for v-for (as block, not regular vnode)
pub fn generate_for_item(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>, is_stable: bool) {
    match node {
        TemplateChildNode::Element(el) => {
            let key_exp = get_element_key(el);
            let is_template = el.tag_type == ElementType::Template;
            let is_component = el.tag_type == ElementType::Component;
            let prev_skip_scope_id = ctx.skip_scope_id;

            // Check for v-memo directive on for item (skip if already handled by v-for)
            let memo_exp = if !ctx.skip_v_memo && has_v_memo(el) {
                get_memo_exp(el)
            } else {
                None
            };

            if let Some(memo_exp) = memo_exp {
                ctx.use_helper(RuntimeHelper::WithMemo);
                ctx.push(ctx.helper(RuntimeHelper::WithMemo));
                ctx.push("(");
                generate_expression(ctx, memo_exp);
                ctx.push(", () => ");
            }

            // Check for v-show directive
            let has_vshow = has_vshow_directive(el);
            if has_vshow {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.use_helper(RuntimeHelper::VShow);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            // Components: skip scope_id in props -- Vue runtime applies it via __scopeId
            if is_component {
                ctx.skip_scope_id = true;
            }

            if is_stable {
                // Stable fragment: use createElementVNode without block wrapper
                ctx.use_helper(RuntimeHelper::CreateElementVNode);
                ctx.push(ctx.helper(RuntimeHelper::CreateElementVNode));
                ctx.push("(\"");
                ctx.push(&el.tag);
                ctx.push("\"");

                // Props with key and all other props
                generate_for_item_props(ctx, el, key_exp);

                // Children
                if !el.children.is_empty() {
                    ctx.push(", ");
                    generate_children(ctx, &el.children);
                }

                // Add TEXT patch flag if has interpolation
                let has_interpolation = el
                    .children
                    .iter()
                    .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));
                if has_interpolation {
                    ctx.push(", 1 /* TEXT */");
                }

                ctx.push(")");

                // Close withDirectives for v-show
                if has_vshow {
                    generate_vshow_closing(ctx, el);
                }
            } else {
                // Dynamic list: wrap in block
                ctx.use_helper(RuntimeHelper::OpenBlock);
                ctx.push("(");
                ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
                ctx.push("(), ");

                // Template with single child element optimization:
                // unwrap the template and generate the child directly as a block
                let unwrapped_child: Option<&ElementNode<'_>> =
                    if is_template && el.children.len() == 1 {
                        if let TemplateChildNode::Element(ref child_el) = el.children[0] {
                            if child_el.tag_type == ElementType::Element {
                                Some(child_el)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                let gen_is_template = is_template && unwrapped_child.is_none();

                if is_component {
                    // Component: use createBlock
                    ctx.use_helper(RuntimeHelper::CreateBlock);
                    ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
                    ctx.push("(");
                    // Handle dynamic component
                    if el.tag == "component" || el.tag == "Component" {
                        let dynamic_is = el.props.iter().find_map(|p| {
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
                        let static_is = el.props.iter().find_map(|p| {
                            if let PropNode::Attribute(attr) = p {
                                if attr.name == "is" {
                                    return attr.value.as_ref().map(|v| v.content.as_str());
                                }
                            }
                            None
                        });
                        if let Some(is_exp) = dynamic_is {
                            ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                            ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                            ctx.push("(");
                            generate_expression(ctx, is_exp);
                            ctx.push(")");
                        } else if let Some(name) = static_is {
                            ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                            ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                            ctx.push("(\"");
                            ctx.push(name);
                            ctx.push("\")");
                        } else {
                            ctx.push("_component_component");
                        }
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
                } else if gen_is_template {
                    // Template with multiple children: use Fragment
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.use_helper(RuntimeHelper::Fragment);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(");
                    ctx.push(ctx.helper(RuntimeHelper::Fragment));
                } else if let Some(child_el) = unwrapped_child {
                    // Template with single child: unwrap to child element
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(\"");
                    ctx.push(&child_el.tag);
                    ctx.push("\"");
                } else {
                    // Regular element
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(\"");
                    ctx.push(&el.tag);
                    ctx.push("\"");
                }

                // Props with key and all other props
                // For unwrapped template child, use child's props with template's key
                let props_el = unwrapped_child.unwrap_or(el);
                generate_for_item_props(ctx, props_el, key_exp);

                // Children
                let children_el = unwrapped_child.unwrap_or(el);
                if !children_el.children.is_empty() {
                    ctx.push(", ");
                    if is_component && has_slot_children(children_el) {
                        // Component children must be compiled as slot functions,
                        // not raw children. Otherwise Vue warns:
                        // "Non-function value encountered for default slot"
                        generate_slots(ctx, children_el);
                    } else if gen_is_template {
                        // Template children are array
                        ctx.push("[");
                        ctx.indent();
                        for (i, child) in children_el.children.iter().enumerate() {
                            if i > 0 {
                                ctx.push(",");
                            }
                            ctx.newline();
                            generate_node(ctx, child);
                        }
                        ctx.deindent();
                        ctx.newline();
                        ctx.push("]");
                    } else if ctx.skip_v_memo {
                        // v-for + v-memo: force array form for children
                        generate_children_force_array(ctx, &children_el.children);
                    } else {
                        generate_children(ctx, &children_el.children);
                    }
                }

                // Add patch flag
                if is_component {
                    let (mut patch_flag, dynamic_props) = calculate_element_patch_info(
                        el,
                        ctx.options.binding_metadata.as_ref(),
                        ctx.options.cache_handlers,
                    );
                    // Remove TEXT flag for components with slot children (text is inside slot)
                    if has_slot_children(el) {
                        if let Some(flag) = patch_flag {
                            let new_flag = flag & !1;
                            patch_flag = if new_flag > 0 { Some(new_flag) } else { None };
                        }
                    }
                    // Inside v-for, component slots are always dynamic
                    if ctx.in_v_for && has_slot_children(el) {
                        let dynamic_slots_flag = 1024;
                        patch_flag = Some(patch_flag.unwrap_or(0) | dynamic_slots_flag);
                    }
                    if el.children.is_empty() && (patch_flag.is_some() || dynamic_props.is_some()) {
                        ctx.push(", null");
                    }
                    if let Some(flag) = patch_flag {
                        ctx.push(", ");
                        ctx.push(&flag.to_compact_string());
                        ctx.push(" /* ");
                        ctx.push(&patch_flag_name(flag));
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
                } else if gen_is_template {
                    ctx.push(", 64 /* STABLE_FRAGMENT */");
                } else if !ctx.skip_v_memo {
                    // Skip patch flags for v-memo elements (memo handles reactivity)
                    let flag_el = unwrapped_child.unwrap_or(el);
                    let (patch_flag, dynamic_props) = calculate_element_patch_info(
                        flag_el,
                        ctx.options.binding_metadata.as_ref(),
                        ctx.options.cache_handlers,
                    );
                    if let Some(flag) = patch_flag {
                        ctx.push(", ");
                        ctx.push(&flag.to_compact_string());
                        ctx.push(" /* ");
                        ctx.push(&patch_flag_name(flag));
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
                }

                ctx.push("))");

                // Close withDirectives for v-show
                if has_vshow {
                    generate_vshow_closing(ctx, el);
                }
            }

            // Close withMemo wrapper for v-for + v-memo
            if memo_exp.is_some() {
                ctx.push(", _cache, ");
                if let Some(key) = key_exp {
                    generate_expression(ctx, key);
                } else {
                    ctx.push("0");
                }
                ctx.push(")");
            }

            ctx.skip_scope_id = prev_skip_scope_id;
        }
        _ => generate_node(ctx, node),
    }
}

/// Generate props for v-for item, including key and all other props
pub(crate) fn generate_for_item_props(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    key_exp: Option<&ExpressionNode<'_>>,
) {
    let has_other = has_other_props(el);
    // For component elements, skip_scope_id suppresses the attribute.
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };

    if key_exp.is_none() && !has_other && scope_id.is_none() {
        ctx.push(", null");
        return;
    }

    ctx.push(", ");

    if !has_other {
        // Only key (and optionally scope_id), no other props
        if let Some(key) = key_exp {
            ctx.push("{ key: ");
            generate_expression(ctx, key);
            if let Some(ref sid) = scope_id {
                ctx.push(", \"");
                ctx.push(sid);
                ctx.push("\": \"\"");
            }
            ctx.push(" }");
        } else if let Some(ref sid) = scope_id {
            // No key, no other props, but has scope_id
            ctx.push("{ \"");
            ctx.push(sid);
            ctx.push("\": \"\" }");
        }
        return;
    }

    // Check for v-bind/v-on object spreads (v-bind="obj", v-on="handlers")
    let has_vbind_spread = super::super::props::has_vbind_object(&el.props);
    let has_von_spread = super::super::props::has_von_object(&el.props);

    if has_vbind_spread || has_von_spread {
        generate_for_item_props_merged(
            ctx,
            el,
            key_exp,
            &scope_id,
            has_vbind_spread,
            has_von_spread,
        );
        return;
    }

    // Detect static class/style that need to be merged with dynamic :class/:style
    let static_class = el.props.iter().find_map(|p| {
        if let PropNode::Attribute(attr) = p {
            if attr.name == "class" {
                return attr.value.as_ref().map(|v| v.content.as_str());
            }
        }
        None
    });

    let static_style = el.props.iter().find_map(|p| {
        if let PropNode::Attribute(attr) = p {
            if attr.name == "style" {
                return attr.value.as_ref().map(|v| v.content.as_str());
            }
        }
        None
    });

    let has_dynamic_class = el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
                    return exp.content == "class";
                }
            }
        }
        false
    });

    let has_dynamic_style = el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
                    return exp.content == "style";
                }
            }
        }
        false
    });

    let skip_static_class = static_class.is_some() && has_dynamic_class;
    let skip_static_style = static_style.is_some() && has_dynamic_style;

    let merge_static_class = if skip_static_class {
        static_class
    } else {
        None
    };
    let merge_static_style = if skip_static_style {
        static_style
    } else {
        None
    };

    if let Some(key) = key_exp {
        // Merge key with other props
        ctx.push("{");
        ctx.indent();
        ctx.newline();
        ctx.push("key: ");
        generate_expression(ctx, key);

        for prop in el.props.iter() {
            if should_skip_prop(prop) {
                continue;
            }
            if skip_static_class {
                if let PropNode::Attribute(attr) = prop {
                    if attr.name == "class" {
                        continue;
                    }
                }
            }
            if skip_static_style {
                if let PropNode::Attribute(attr) = prop {
                    if attr.name == "style" {
                        continue;
                    }
                }
            }
            ctx.push(",");
            ctx.newline();
            generate_single_prop(ctx, prop, merge_static_class, merge_static_style);
        }

        if let Some(ref sid) = scope_id {
            ctx.push(",");
            ctx.newline();
            ctx.push("\"");
            ctx.push(sid);
            ctx.push("\": \"\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    } else {
        // No key, generate props directly (skipping v-for directive)
        ctx.push("{");
        let mut first = true;
        for prop in el.props.iter() {
            if should_skip_prop(prop) {
                continue;
            }
            if skip_static_class {
                if let PropNode::Attribute(attr) = prop {
                    if attr.name == "class" {
                        continue;
                    }
                }
            }
            if skip_static_style {
                if let PropNode::Attribute(attr) = prop {
                    if attr.name == "style" {
                        continue;
                    }
                }
            }
            if !first {
                ctx.push(",");
            }
            ctx.push(" ");
            generate_single_prop(ctx, prop, merge_static_class, merge_static_style);
            first = false;
        }

        if let Some(ref sid) = scope_id {
            if !first {
                ctx.push(",");
            }
            ctx.push(" \"");
            ctx.push(sid);
            ctx.push("\": \"\"");
        }

        ctx.push(" }");
    }
}

/// Generate props using _mergeProps when v-bind/v-on object spreads are present.
fn generate_for_item_props_merged(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    key_exp: Option<&ExpressionNode<'_>>,
    scope_id: &Option<vize_carton::String>,
    has_vbind_spread: bool,
    has_von_spread: bool,
) {
    ctx.use_helper(RuntimeHelper::MergeProps);
    ctx.push(ctx.helper(RuntimeHelper::MergeProps));
    ctx.push("(");

    let mut first_merge_arg = true;

    if has_vbind_spread {
        super::super::props::generate_vbind_object_exp(ctx, &el.props);
        first_merge_arg = false;
    }

    if has_von_spread {
        if !first_merge_arg {
            ctx.push(", ");
        }
        super::super::props::generate_von_object_exp(ctx, &el.props);
        first_merge_arg = false;
    }

    let has_remaining = key_exp.is_some()
        || scope_id.is_some()
        || el.props.iter().any(|p| {
            if should_skip_prop(p) {
                return false;
            }
            if let PropNode::Directive(dir) = p {
                if dir.arg.is_none() && (dir.name == "bind" || dir.name == "on") {
                    return false;
                }
            }
            true
        });

    if has_remaining {
        if !first_merge_arg {
            ctx.push(", ");
        }
        ctx.push("{");
        ctx.indent();
        let mut first_prop = true;

        if let Some(key) = key_exp {
            ctx.newline();
            ctx.push("key: ");
            generate_expression(ctx, key);
            first_prop = false;
        }

        for prop in el.props.iter() {
            if should_skip_prop(prop) {
                continue;
            }
            if let PropNode::Directive(dir) = prop {
                if dir.arg.is_none() && (dir.name == "bind" || dir.name == "on") {
                    continue;
                }
            }
            if !first_prop {
                ctx.push(",");
            }
            ctx.newline();
            generate_single_prop(ctx, prop, None, None);
            first_prop = false;
        }

        if let Some(ref sid) = scope_id {
            if !first_prop {
                ctx.push(",");
            }
            ctx.newline();
            ctx.push("\"");
            ctx.push(sid);
            ctx.push("\": \"\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    }

    ctx.push(")");
}

/// Generate a single prop (attribute or directive)
fn generate_single_prop(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_class: Option<&str>,
    static_style: Option<&str>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let needs_quotes = !super::super::helpers::is_valid_js_identifier(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = &attr.value {
                ctx.push("\"");
                ctx.push(&escape_js_string(&value.content));
                ctx.push("\"");
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            super::super::props::generate_directive_prop_with_static(
                ctx,
                dir,
                static_class,
                static_style,
            );
        }
    }
}
