//! Element transformation.
//!
//! Handles `ElementNode` transformation including plain elements, components,
//! slots, and template elements. Also provides template string generation
//! and static analysis helpers.

use vize_carton::{append, cstr, Box, String, Vec};

use crate::ir::{
    BlockIRNode, ChildRefIRNode, ComponentKind, CreateComponentIRNode, IREffect, IRProp, IRSlot,
    NextRefIRNode, OperationNode, SetTextIRNode, SlotOutletIRNode,
};
use vize_atelier_core::{
    ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

use super::{
    context::TransformContext,
    control::{transform_for_node, transform_if_node},
    directive::transform_directive,
    text::{transform_interpolation, transform_text, transform_text_children},
    transform_children,
};

/// Transform element node
pub(crate) fn transform_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    // Template elements don't consume an ID - they just wrap children
    if el.tag_type == ElementType::Template {
        for child in el.children.iter() {
            match child {
                TemplateChildNode::Element(child_el) => {
                    transform_element(ctx, child_el, block);
                }
                TemplateChildNode::Text(text) => {
                    transform_text(ctx, text, block);
                }
                TemplateChildNode::Interpolation(interp) => {
                    transform_interpolation(ctx, interp, block);
                }
                TemplateChildNode::If(if_node) => {
                    transform_if_node(ctx, if_node, block);
                }
                TemplateChildNode::For(for_node) => {
                    transform_for_node(ctx, for_node, block);
                }
                _ => {}
            }
        }
        return;
    }

    // Check if this element has non-static children that require
    // deferred ID allocation (so inner templates/IDs come first).
    let has_control_flow_children = el.tag_type == ElementType::Element
        && el
            .children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::If(_) | TemplateChildNode::For(_)));
    let has_dynamic_element_children = el.tag_type == ElementType::Element
        && !has_control_flow_children
        && el.children.iter().any(
            |c| matches!(c, TemplateChildNode::Element(child_el) if !is_static_element(child_el)),
        );

    if has_dynamic_element_children {
        // Dynamic element children: allocate child IDs first, then parent ID.
        // Use child/next navigation instead of separate templates.
        transform_element_with_dynamic_children(ctx, el, block);
        return;
    }

    if has_control_flow_children {
        // Control flow children (v-if/v-for): defer parent ID and template
        // allocation until after children, so inner IDs/templates come first.
        transform_element_with_control_flow_children(ctx, el, block);
        return;
    }

    // Components handle their own ID allocation (slots consume IDs before the component)
    // Also handle <component :is="..."> (dynamic component) which the parser classifies as Element
    if el.tag_type == ElementType::Component || el.tag.as_str() == "component" {
        transform_component(ctx, el, block);
        return;
    }

    let element_id = ctx.next_id();

    match el.tag_type {
        ElementType::Element => {
            let template = generate_element_template(el);

            // Process props and events
            for prop in el.props.iter() {
                match prop {
                    PropNode::Directive(dir) => {
                        transform_directive(ctx, dir, element_id, el, block);
                    }
                    PropNode::Attribute(_attr) => {
                        // Static attributes are included in the template
                    }
                }
            }

            // Check if we have mixed text and interpolation children
            let has_text_or_interpolation = el.children.iter().any(|c| {
                matches!(
                    c,
                    TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
                )
            });
            let has_interpolation = el
                .children
                .iter()
                .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

            if has_interpolation && has_text_or_interpolation {
                // Collect all text parts and interpolations together
                transform_text_children(ctx, &el.children, element_id, block);
            }

            // Register template (no deferred children to process)
            ctx.add_template(element_id, template);
        }
        ElementType::Component => {
            let mut props = Vec::new_in(ctx.allocator);
            let slots = Vec::new_in(ctx.allocator);

            // Process props (v-bind and v-on directives, and static attributes)
            for prop in el.props.iter() {
                match prop {
                    PropNode::Directive(dir) => {
                        if dir.name.as_str() == "bind" {
                            // v-bind -> prop
                            if let Some(ref arg) = dir.arg {
                                if let ExpressionNode::Simple(key_exp) = arg {
                                    let key_node = SimpleExpressionNode::new(
                                        key_exp.content.clone(),
                                        key_exp.is_static,
                                        key_exp.loc.clone(),
                                    );
                                    let key = Box::new_in(key_node, ctx.allocator);

                                    let mut values = Vec::new_in(ctx.allocator);
                                    if let Some(ref exp) = dir.exp {
                                        if let ExpressionNode::Simple(val_exp) = exp {
                                            let val_node = SimpleExpressionNode::new(
                                                val_exp.content.clone(),
                                                val_exp.is_static,
                                                val_exp.loc.clone(),
                                            );
                                            values.push(Box::new_in(val_node, ctx.allocator));
                                        }
                                    }

                                    props.push(IRProp {
                                        key,
                                        values,
                                        is_component: true,
                                    });
                                }
                            }
                        } else if dir.name.as_str() == "on" {
                            // v-on -> onXxx prop
                            if let Some(ref arg) = dir.arg {
                                if let ExpressionNode::Simple(event_exp) = arg {
                                    let event_name = event_exp.content.as_str();
                                    let on_name = if event_name.is_empty() {
                                        String::from("on")
                                    } else {
                                        let mut s = String::from("on");
                                        let mut chars = event_name.chars();
                                        if let Some(c) = chars.next() {
                                            s.push(c.to_ascii_uppercase());
                                        }
                                        for c in chars {
                                            s.push(c);
                                        }
                                        s
                                    };

                                    let key_node = SimpleExpressionNode::new(
                                        on_name,
                                        true,
                                        event_exp.loc.clone(),
                                    );
                                    let key = Box::new_in(key_node, ctx.allocator);

                                    let mut values = Vec::new_in(ctx.allocator);
                                    if let Some(ref exp) = dir.exp {
                                        if let ExpressionNode::Simple(val_exp) = exp {
                                            let val_node = SimpleExpressionNode::new(
                                                val_exp.content.clone(),
                                                val_exp.is_static,
                                                val_exp.loc.clone(),
                                            );
                                            values.push(Box::new_in(val_node, ctx.allocator));
                                        }
                                    }

                                    props.push(IRProp {
                                        key,
                                        values,
                                        is_component: true,
                                    });
                                }
                            }
                        } else if dir.name.as_str() == "model" {
                            // v-model -> modelValue + onUpdate:modelValue props
                            let binding = if let Some(ref exp) = dir.exp {
                                match exp {
                                    ExpressionNode::Simple(s) => s.content.clone(),
                                    _ => String::from(""),
                                }
                            } else {
                                String::from("")
                            };

                            // Determine prop name from argument (default: "modelValue")
                            let prop_name = dir
                                .arg
                                .as_ref()
                                .map(|arg| match arg {
                                    ExpressionNode::Simple(s) => s.content.clone(),
                                    _ => String::from("modelValue"),
                                })
                                .unwrap_or_else(|| String::from("modelValue"));

                            // Add modelValue prop
                            let key_node = SimpleExpressionNode::new(
                                prop_name.clone(),
                                true,
                                SourceLocation::STUB,
                            );
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            let val_node = SimpleExpressionNode::new(
                                binding.clone(),
                                false,
                                SourceLocation::STUB,
                            );
                            values.push(Box::new_in(val_node, ctx.allocator));
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });

                            // Add onUpdate:propName event prop
                            let event_key = {
                                let mut s = String::from("onUpdate:");
                                s.push_str(prop_name.as_str());
                                s
                            };
                            let event_key_node =
                                SimpleExpressionNode::new(event_key, true, SourceLocation::STUB);
                            let event_key_box = Box::new_in(event_key_node, ctx.allocator);
                            // Handler: _value => (_ctx.binding = _value)
                            // Mark as static so generate won't add _ctx. prefix
                            let handler_content = {
                                let mut s = String::from("__RAW__() => _value => (_ctx.");
                                s.push_str(binding.as_str());
                                s.push_str(" = _value)");
                                s
                            };
                            let handler_node = SimpleExpressionNode::new(
                                handler_content,
                                true,
                                SourceLocation::STUB,
                            );
                            let mut handler_values = Vec::new_in(ctx.allocator);
                            handler_values.push(Box::new_in(handler_node, ctx.allocator));
                            props.push(IRProp {
                                key: event_key_box,
                                values: handler_values,
                                is_component: true,
                            });

                            // Add modifiers prop if present
                            if !dir.modifiers.is_empty() {
                                let mod_key_name = if prop_name == "modelValue" {
                                    String::from("modelModifiers")
                                } else {
                                    let mut s = prop_name.clone();
                                    s.push_str("Modifiers");
                                    s
                                };
                                let mod_key_node = SimpleExpressionNode::new(
                                    mod_key_name,
                                    true,
                                    SourceLocation::STUB,
                                );
                                let mod_key = Box::new_in(mod_key_node, ctx.allocator);
                                // Build modifiers object content
                                let mut mod_content = String::from("__RAW__() => ({ ");
                                for (i, m) in dir.modifiers.iter().enumerate() {
                                    if i > 0 {
                                        mod_content.push_str(", ");
                                    }
                                    mod_content.push_str(m.content.as_str());
                                    mod_content.push_str(": true");
                                }
                                mod_content.push_str(" })");
                                let mod_val_node = SimpleExpressionNode::new(
                                    mod_content,
                                    true,
                                    SourceLocation::STUB,
                                );
                                let mut mod_values = Vec::new_in(ctx.allocator);
                                mod_values.push(Box::new_in(mod_val_node, ctx.allocator));
                                props.push(IRProp {
                                    key: mod_key,
                                    values: mod_values,
                                    is_component: true,
                                });
                            }
                        }
                    }
                    PropNode::Attribute(attr) => {
                        // Static attribute -> prop
                        let key_node = SimpleExpressionNode::new(
                            attr.name.clone(),
                            true,
                            SourceLocation::STUB,
                        );
                        let key = Box::new_in(key_node, ctx.allocator);

                        let mut values = Vec::new_in(ctx.allocator);
                        if let Some(ref value) = attr.value {
                            let val_node = SimpleExpressionNode::new(
                                value.content.clone(),
                                true,
                                SourceLocation::STUB,
                            );
                            values.push(Box::new_in(val_node, ctx.allocator));
                        }

                        props.push(IRProp {
                            key,
                            values,
                            is_component: true,
                        });
                    }
                }
            }

            let create_component = CreateComponentIRNode {
                id: element_id,
                tag: el.tag.clone(),
                props,
                slots,
                asset: true,
                once: false,
                dynamic_slots: false,
                kind: crate::ir::ComponentKind::Regular,
                is_expr: None,
                v_show: None,
            };

            block
                .operation
                .push(OperationNode::CreateComponent(create_component));
        }
        ElementType::Slot => {
            // Slot outlet handling
            let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
            let slot_outlet = SlotOutletIRNode {
                id: element_id,
                name: Box::new_in(name_exp, ctx.allocator),
                props: Vec::new_in(ctx.allocator),
                fallback: None,
            };

            block.operation.push(OperationNode::SlotOutlet(slot_outlet));
        }
        ElementType::Template => {
            // Handled at top of function, unreachable
            unreachable!("Template elements handled at top of transform_element");
        }
    }

    block.returns.push(element_id);
}

/// Generate element template string (recursively includes static children)
pub(crate) fn generate_element_template(el: &ElementNode<'_>) -> String {
    let mut template = cstr!("<{}", el.tag);

    // Collect dynamic binding names to skip their static counterparts
    let dynamic_attrs: vize_carton::FxHashSet<&str> = el
        .props
        .iter()
        .filter_map(|p| {
            if let PropNode::Directive(dir) = p {
                if dir.name.as_str() == "bind" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(key) = arg {
                            return Some(key.content.as_str());
                        }
                    }
                }
            }
            None
        })
        .collect();

    // Add static attributes (skip those overridden by dynamic bindings)
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if dynamic_attrs.contains(attr.name.as_str()) {
                continue;
            }
            if let Some(ref value) = attr.value {
                append!(template, " {}=\"{}\"", attr.name, value.content);
            } else {
                append!(template, " {}", attr.name);
            }
        }
    }

    if el.is_self_closing || is_void_element(&el.tag) {
        template.push('>');
    } else {
        template.push('>');

        // Check if there are any interpolations - if so, use a space placeholder
        let has_interpolation = el
            .children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

        if has_interpolation {
            // Use single space as placeholder for interpolation text content
            template.push(' ');
        } else {
            // Recursively add static children (text and static elements)
            for child in el.children.iter() {
                match child {
                    TemplateChildNode::Text(text) => {
                        template.push_str(&escape_html_text(&text.content));
                    }
                    TemplateChildNode::Element(child_el) => {
                        // Include child elements in template
                        template.push_str(&generate_element_template(child_el));
                    }
                    _ => {
                        // Other dynamic content is handled elsewhere
                    }
                }
            }
        }

        append!(template, "</{}>", el.tag);
    }

    template
}

/// Escape HTML special characters in text content (vuejs/core #14310)
pub(crate) fn escape_html_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

/// Check if an element is static (no dynamic directives)
pub(crate) fn is_static_element(el: &ElementNode<'_>) -> bool {
    // Check if any prop is a directive (dynamic)
    for prop in el.props.iter() {
        if matches!(prop, PropNode::Directive(_)) {
            return false;
        }
    }

    // Check if any child is dynamic
    for child in el.children.iter() {
        match child {
            TemplateChildNode::Interpolation(_) => return false,
            TemplateChildNode::Element(child_el) => {
                if !is_static_element(child_el) {
                    return false;
                }
            }
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => return false,
            _ => {}
        }
    }

    true
}

/// Transform an element that has control flow children (v-if/v-for).
/// The parent element ID is allocated AFTER children so inner IDs come first.
fn transform_element_with_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let template = generate_element_template(el);

    // Save the current returns length so we can remove any returns added by children.
    // Child control flow nodes (v-if/v-for) push their IDs to block.returns,
    // but for an element with control flow children, only the parent element
    // should be returned.
    let returns_before = block.returns.len();

    // Process children FIRST (before allocating parent ID)
    for child in el.children.iter() {
        match child {
            TemplateChildNode::Interpolation(_) | TemplateChildNode::Text(_) => {
                // Text/interpolation handled after element_id is known
            }
            TemplateChildNode::Element(child_el) => {
                if !is_static_element(child_el) {
                    transform_element(ctx, child_el, block);
                }
            }
            TemplateChildNode::If(if_node) => {
                transform_if_node(ctx, if_node, block);
            }
            TemplateChildNode::For(for_node) => {
                transform_for_node(ctx, for_node, block);
            }
            _ => {}
        }
    }

    // Remove any returns added by children — they are operations within this
    // element, not top-level returns.
    block.returns.truncate(returns_before);

    // NOW allocate parent ID (after all children have consumed their IDs)
    let element_id = ctx.next_id();

    // Process props and events
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                transform_directive(ctx, dir, element_id, el, block);
            }
            PropNode::Attribute(_attr) => {}
        }
    }

    // Handle text content if needed
    let has_text_or_interpolation = el.children.iter().any(|c| {
        matches!(
            c,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });
    let has_interpolation = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

    if has_interpolation && has_text_or_interpolation {
        transform_text_children(ctx, &el.children, element_id, block);
    }

    // Register template AFTER children
    ctx.add_template(element_id, template);

    block.returns.push(element_id);
}

/// Transform an element that has dynamic element children.
/// Child IDs are allocated before the parent ID, and ChildRef/NextRef
/// operations are used instead of separate templates for each child.
fn transform_element_with_dynamic_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    // Collect dynamic element children (those that need child/next refs)
    let mut dynamic_child_indices: std::vec::Vec<usize> = std::vec::Vec::new();
    for (i, child) in el.children.iter().enumerate() {
        if let TemplateChildNode::Element(child_el) = child {
            if !is_static_element(child_el) {
                dynamic_child_indices.push(i);
            }
        }
    }

    // Allocate IDs for dynamic children first
    let child_ids: std::vec::Vec<usize> = dynamic_child_indices
        .iter()
        .map(|_| ctx.next_id())
        .collect();

    // Now allocate parent ID (will be higher than all child IDs)
    let parent_id = ctx.next_id();

    // Generate template (includes all children inline)
    let template = generate_element_template(el);

    // Process parent props
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                transform_directive(ctx, dir, parent_id, el, block);
            }
            PropNode::Attribute(_attr) => {}
        }
    }

    // Generate ChildRef/NextRef operations for dynamic children
    // and collect SetText operations into a single combined effect
    let mut prev_child_id: Option<usize> = None;
    let mut combined_effect_ops = Vec::new_in(ctx.allocator);

    for (idx, &child_index) in dynamic_child_indices.iter().enumerate() {
        let child_id = child_ids[idx];

        if idx == 0 {
            block
                .operation
                .push(OperationNode::ChildRef(ChildRefIRNode {
                    child_id,
                    parent_id,
                }));
        } else {
            let prev_dyn_index = dynamic_child_indices[idx - 1];
            let mut offset = 0usize;
            for i in (prev_dyn_index + 1)..=child_index {
                if matches!(el.children[i], TemplateChildNode::Element(_)) {
                    offset += 1;
                }
            }
            block.operation.push(OperationNode::NextRef(NextRefIRNode {
                child_id,
                prev_id: prev_child_id.unwrap(),
                offset,
            }));
        }
        prev_child_id = Some(child_id);

        // Collect child's text content into combined effect
        if let TemplateChildNode::Element(child_el) = &el.children[child_index] {
            let has_interpolation = child_el
                .children
                .iter()
                .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));
            let has_text_or_interpolation = child_el.children.iter().any(|c| {
                matches!(
                    c,
                    TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
                )
            });

            if has_interpolation && has_text_or_interpolation {
                let mut values = Vec::new_in(ctx.allocator);
                for c in child_el.children.iter() {
                    match c {
                        TemplateChildNode::Text(text) => {
                            let exp = SimpleExpressionNode::new(
                                text.content.clone(),
                                true,
                                SourceLocation::STUB,
                            );
                            values.push(Box::new_in(exp, ctx.allocator));
                        }
                        TemplateChildNode::Interpolation(interp) => {
                            if let ExpressionNode::Simple(simple) = &interp.content {
                                let exp = SimpleExpressionNode::new(
                                    simple.content.clone(),
                                    simple.is_static,
                                    simple.loc.clone(),
                                );
                                values.push(Box::new_in(exp, ctx.allocator));
                            }
                        }
                        _ => {}
                    }
                }
                if !values.is_empty() {
                    combined_effect_ops.push(OperationNode::SetText(SetTextIRNode {
                        element: child_id,
                        values,
                    }));
                }
            }
        }
    }

    // Add the combined effect with all SetText operations
    if !combined_effect_ops.is_empty() {
        block.effect.push(IREffect {
            operations: combined_effect_ops,
        });
    }

    // Register template for parent
    ctx.add_template(parent_id, template);

    block.returns.push(parent_id);
}

/// Transform a component element (handles slots, built-in components, dynamic components, v-show)
fn transform_component<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let tag = el.tag.as_str();
    let kind = match tag {
        "Teleport" => ComponentKind::Teleport,
        "KeepAlive" => ComponentKind::KeepAlive,
        "Suspense" => ComponentKind::Suspense,
        "component" => ComponentKind::Dynamic,
        _ => ComponentKind::Regular,
    };

    let mut props = Vec::new_in(ctx.allocator);
    let mut slots = Vec::new_in(ctx.allocator);
    let mut v_show_exp: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut is_expr: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut has_dynamic_slot = false;

    // Check for v-slot on the component itself (scoped default slot)
    let mut has_v_slot_on_component = false;
    let mut slot_props_expr: Option<String> = None;
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            if dir.name.as_str() == "slot" {
                has_v_slot_on_component = true;
                if let Some(ref exp) = dir.exp {
                    if let ExpressionNode::Simple(s) = exp {
                        slot_props_expr = Some(s.content.clone());
                    }
                }
            }
        }
    }

    // Process props
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                if dir.name.as_str() == "slot" {
                    continue;
                }
                if dir.name.as_str() == "show" {
                    if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(s) = exp {
                            let node = SimpleExpressionNode::new(
                                s.content.clone(),
                                s.is_static,
                                s.loc.clone(),
                            );
                            v_show_exp = Some(Box::new_in(node, ctx.allocator));
                        }
                    }
                    continue;
                }
                if dir.name.as_str() == "bind" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(key_exp) = arg {
                            if kind == ComponentKind::Dynamic && key_exp.content.as_str() == "is" {
                                if let Some(ref exp) = dir.exp {
                                    if let ExpressionNode::Simple(val_exp) = exp {
                                        let node = SimpleExpressionNode::new(
                                            val_exp.content.clone(),
                                            val_exp.is_static,
                                            val_exp.loc.clone(),
                                        );
                                        is_expr = Some(Box::new_in(node, ctx.allocator));
                                    }
                                }
                                continue;
                            }
                            let key_node = SimpleExpressionNode::new(
                                key_exp.content.clone(),
                                key_exp.is_static,
                                key_exp.loc.clone(),
                            );
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            if let Some(ref exp) = dir.exp {
                                if let ExpressionNode::Simple(val_exp) = exp {
                                    let val_node = SimpleExpressionNode::new(
                                        val_exp.content.clone(),
                                        val_exp.is_static,
                                        val_exp.loc.clone(),
                                    );
                                    values.push(Box::new_in(val_node, ctx.allocator));
                                }
                            }
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });
                        }
                    }
                } else if dir.name.as_str() == "on" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(event_exp) = arg {
                            let event_name = event_exp.content.as_str();
                            let on_name = if event_name.is_empty() {
                                String::from("on")
                            } else {
                                let mut s = String::from("on");
                                let mut chars = event_name.chars();
                                if let Some(c) = chars.next() {
                                    s.push(c.to_ascii_uppercase());
                                }
                                for c in chars {
                                    s.push(c);
                                }
                                s
                            };
                            let key_node =
                                SimpleExpressionNode::new(on_name, true, event_exp.loc.clone());
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            if let Some(ref exp) = dir.exp {
                                if let ExpressionNode::Simple(val_exp) = exp {
                                    let val_node = SimpleExpressionNode::new(
                                        val_exp.content.clone(),
                                        val_exp.is_static,
                                        val_exp.loc.clone(),
                                    );
                                    values.push(Box::new_in(val_node, ctx.allocator));
                                }
                            }
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });
                        }
                    }
                } else if dir.name.as_str() == "model" {
                    transform_component_v_model(ctx, dir, &mut props);
                }
            }
            PropNode::Attribute(attr) => {
                let key_node =
                    SimpleExpressionNode::new(attr.name.clone(), true, SourceLocation::STUB);
                let key = Box::new_in(key_node, ctx.allocator);
                let mut values = Vec::new_in(ctx.allocator);
                if let Some(ref value) = attr.value {
                    let val_node = SimpleExpressionNode::new(
                        value.content.clone(),
                        true,
                        SourceLocation::STUB,
                    );
                    values.push(Box::new_in(val_node, ctx.allocator));
                }
                props.push(IRProp {
                    key,
                    values,
                    is_component: true,
                });
            }
        }
    }

    // Process children to create slots
    if has_v_slot_on_component {
        let slot_block = transform_children(ctx, &el.children);
        let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
        let fn_exp = slot_props_expr.map(|expr| {
            let node = SimpleExpressionNode::new(expr, false, SourceLocation::STUB);
            Box::new_in(node, ctx.allocator)
        });
        slots.push(IRSlot {
            name: Box::new_in(name_exp, ctx.allocator),
            fn_exp,
            block: slot_block,
        });
    } else if !el.children.is_empty() {
        let has_named_slots = el.children.iter().any(|c| {
            if let TemplateChildNode::Element(child_el) = c {
                child_el.tag_type == ElementType::Template
                    && child_el
                        .props
                        .iter()
                        .any(|p| matches!(p, PropNode::Directive(d) if d.name.as_str() == "slot"))
            } else {
                false
            }
        });

        if has_named_slots {
            for child in el.children.iter() {
                if let TemplateChildNode::Element(child_el) = child {
                    if child_el.tag_type == ElementType::Template {
                        for prop in child_el.props.iter() {
                            if let PropNode::Directive(dir) = prop {
                                if dir.name.as_str() == "slot" {
                                    let (slot_name, is_static_name) = if let Some(ref arg) = dir.arg
                                    {
                                        match arg {
                                            ExpressionNode::Simple(exp) => {
                                                (exp.content.clone(), exp.is_static)
                                            }
                                            _ => (String::from("default"), true),
                                        }
                                    } else {
                                        (String::from("default"), true)
                                    };
                                    if !is_static_name {
                                        has_dynamic_slot = true;
                                    }
                                    let fn_exp = dir.exp.as_ref().and_then(|exp| match exp {
                                        ExpressionNode::Simple(s) => {
                                            let node = SimpleExpressionNode::new(
                                                s.content.clone(),
                                                false,
                                                SourceLocation::STUB,
                                            );
                                            Some(Box::new_in(node, ctx.allocator))
                                        }
                                        _ => None,
                                    });
                                    let slot_block = transform_children(ctx, &child_el.children);
                                    let _template_id = ctx.next_id(); // consume ID for template wrapper
                                    let name_exp = SimpleExpressionNode::new(
                                        slot_name,
                                        is_static_name,
                                        SourceLocation::STUB,
                                    );
                                    slots.push(IRSlot {
                                        name: Box::new_in(name_exp, ctx.allocator),
                                        fn_exp,
                                        block: slot_block,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let slot_block = transform_children(ctx, &el.children);
            let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
            slots.push(IRSlot {
                name: Box::new_in(name_exp, ctx.allocator),
                fn_exp: None,
                block: slot_block,
            });
        }
    }

    let element_id = ctx.next_id();

    let create_component = CreateComponentIRNode {
        id: element_id,
        tag: el.tag.clone(),
        props,
        slots,
        asset: kind == ComponentKind::Regular || kind == ComponentKind::Suspense,
        once: false,
        dynamic_slots: has_dynamic_slot,
        kind,
        is_expr,
        v_show: v_show_exp,
    };

    block
        .operation
        .push(OperationNode::CreateComponent(create_component));
    block.returns.push(element_id);
}

/// Transform v-model on component (helper for transform_component)
fn transform_component_v_model<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &vize_atelier_core::DirectiveNode<'a>,
    props: &mut Vec<'a, IRProp<'a>>,
) {
    let binding = if let Some(ref exp) = dir.exp {
        match exp {
            ExpressionNode::Simple(s) => s.content.clone(),
            _ => String::from(""),
        }
    } else {
        String::from("")
    };
    let prop_name = dir
        .arg
        .as_ref()
        .map(|arg| match arg {
            ExpressionNode::Simple(s) => s.content.clone(),
            _ => String::from("modelValue"),
        })
        .unwrap_or_else(|| String::from("modelValue"));

    let key_node = SimpleExpressionNode::new(prop_name.clone(), true, SourceLocation::STUB);
    let key = Box::new_in(key_node, ctx.allocator);
    let mut values = Vec::new_in(ctx.allocator);
    let val_node = SimpleExpressionNode::new(binding.clone(), false, SourceLocation::STUB);
    values.push(Box::new_in(val_node, ctx.allocator));
    props.push(IRProp {
        key,
        values,
        is_component: true,
    });

    let event_key = {
        let mut s = String::from("onUpdate:");
        s.push_str(prop_name.as_str());
        s
    };
    let event_key_node = SimpleExpressionNode::new(event_key, true, SourceLocation::STUB);
    let event_key_box = Box::new_in(event_key_node, ctx.allocator);
    let handler_content = {
        let mut s = String::from("__RAW__() => _value => (_ctx.");
        s.push_str(binding.as_str());
        s.push_str(" = _value)");
        s
    };
    let handler_node = SimpleExpressionNode::new(handler_content, true, SourceLocation::STUB);
    let mut handler_values = Vec::new_in(ctx.allocator);
    handler_values.push(Box::new_in(handler_node, ctx.allocator));
    props.push(IRProp {
        key: event_key_box,
        values: handler_values,
        is_component: true,
    });

    if !dir.modifiers.is_empty() {
        let mod_key_name = if prop_name == "modelValue" {
            String::from("modelModifiers")
        } else {
            let mut s = prop_name.clone();
            s.push_str("Modifiers");
            s
        };
        let mod_key_node = SimpleExpressionNode::new(mod_key_name, true, SourceLocation::STUB);
        let mod_key = Box::new_in(mod_key_node, ctx.allocator);
        let mut mod_content = String::from("__RAW__() => ({ ");
        for (i, m) in dir.modifiers.iter().enumerate() {
            if i > 0 {
                mod_content.push_str(", ");
            }
            mod_content.push_str(m.content.as_str());
            mod_content.push_str(": true");
        }
        mod_content.push_str(" })");
        let mod_val_node = SimpleExpressionNode::new(mod_content, true, SourceLocation::STUB);
        let mut mod_values = Vec::new_in(ctx.allocator);
        mod_values.push(Box::new_in(mod_val_node, ctx.allocator));
        props.push(IRProp {
            key: mod_key,
            values: mod_values,
            is_component: true,
        });
    }
}

/// Check if an element is a void (self-closing) HTML element
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
