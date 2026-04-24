//! Element transformation.
//!
//! Handles `ElementNode` transformation including plain elements, components,
//! slots, and template elements. Also provides template string generation
//! and static analysis helpers.

use vize_carton::{append, cstr, Box, String, Vec};

use crate::ir::{
    BlockIRNode, ChildRefIRNode, ComponentKind, CreateComponentIRNode, IRProp, IRSlot,
    NextRefIRNode, OperationNode, SetTemplateRefIRNode, SlotOutletIRNode,
};
use vize_atelier_core::{
    ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

use super::{
    context::TransformContext,
    control::{
        transform_for_node, transform_for_node_into_parent, transform_if_node,
        transform_if_node_into_parent,
    },
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
        transform_component(ctx, el, block, None, None, None, true);
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

            transform_template_ref(ctx, el, element_id, block);

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
                            // v-bind -> prop, v-bind="obj" -> ordered spread source
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
                            } else if let Some(ref exp) = dir.exp {
                                if let ExpressionNode::Simple(val_exp) = exp {
                                    let key_node =
                                        SimpleExpressionNode::new("$", true, SourceLocation::STUB);
                                    let key = Box::new_in(key_node, ctx.allocator);
                                    let mut values = Vec::new_in(ctx.allocator);
                                    let val_node = SimpleExpressionNode::new(
                                        val_exp.content.clone(),
                                        val_exp.is_static,
                                        val_exp.loc.clone(),
                                    );
                                    values.push(Box::new_in(val_node, ctx.allocator));

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
                            // Handler getter: the Vapor runtime resolves raw
                            // component props lazily before emit invokes it.
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
                parent: None,
                anchor: None,
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
            if is_runtime_only_attr(attr.name.as_str()) {
                continue;
            }
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

    if is_void_element(&el.tag) {
        template.push('>');
    } else if el.is_self_closing {
        append!(template, "></{}>", el.tag);
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
                        if is_template_backed_element(child_el) {
                            template.push_str(&generate_element_template(child_el));
                        }
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
    if !matches!(el.tag_type, ElementType::Element) {
        return false;
    }

    // Template refs require runtime child lookup even when the rest of the
    // subtree is static, so they must not be folded into a purely static path.
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(_) => return false,
            PropNode::Attribute(attr) if is_runtime_only_attr(attr.name.as_str()) => return false,
            _ => {}
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

fn is_template_backed_element(el: &ElementNode<'_>) -> bool {
    matches!(el.tag_type, ElementType::Element)
}

fn transform_template_ref<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let Some(value) = extract_template_ref_value(ctx, el) else {
        return;
    };

    block
        .operation
        .push(OperationNode::SetTemplateRef(SetTemplateRefIRNode {
            element: element_id,
            value,
            ref_for: has_static_ref_for(el),
        }));
}

fn extract_template_ref_value<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) if attr.name.as_str() == "ref" => {
                let value = attr.value.as_ref()?;
                let node =
                    SimpleExpressionNode::new(value.content.clone(), true, value.loc.clone());
                return Some(Box::new_in(node, ctx.allocator));
            }
            PropNode::Directive(dir) if dir.name.as_str() == "bind" => {
                let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref() else {
                    continue;
                };
                if arg.content.as_str() != "ref" {
                    continue;
                }

                let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref() else {
                    continue;
                };
                let node =
                    SimpleExpressionNode::new(exp.content.clone(), exp.is_static, exp.loc.clone());
                return Some(Box::new_in(node, ctx.allocator));
            }
            _ => {}
        }
    }

    None
}

fn has_static_ref_for(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Attribute(attr) if attr.name.as_str() == "ref_for"
        )
    })
}

fn is_runtime_only_attr(name: &str) -> bool {
    matches!(name, "ref" | "ref_for" | "ref_key")
}

/// Transform an element that has control flow children (v-if/v-for).
/// The parent element ID is allocated AFTER children so inner IDs come first.
fn transform_element_with_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let template = generate_element_template(el);
    let dynamic_child_indices = collect_dynamic_child_indices(el);
    let child_ids: std::vec::Vec<usize> = dynamic_child_indices
        .iter()
        .map(|_| ctx.next_id())
        .collect();

    // Allocate the parent after reserving direct dynamic child IDs so child refs
    // still sort before the parent, while keeping all nested wiring anchored to it.
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

    transform_template_ref(ctx, el, element_id, block);

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

    if !dynamic_child_indices.is_empty() {
        transform_dynamic_children_with_ids(
            ctx,
            el,
            element_id,
            block,
            &dynamic_child_indices,
            &child_ids,
        );
    }

    transform_existing_element_control_flow_children(ctx, el, element_id, block);

    // Register template after nested wiring is emitted
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
    let dynamic_child_indices = collect_dynamic_child_indices(el);
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

    transform_template_ref(ctx, el, parent_id, block);

    transform_dynamic_children_with_ids(
        ctx,
        el,
        parent_id,
        block,
        &dynamic_child_indices,
        &child_ids,
    );

    // Register template for parent
    ctx.add_template(parent_id, template);

    block.returns.push(parent_id);
}

fn collect_dynamic_child_indices(el: &ElementNode<'_>) -> std::vec::Vec<usize> {
    let mut dynamic_child_indices = std::vec::Vec::new();
    for (i, child) in el.children.iter().enumerate() {
        if let TemplateChildNode::Element(child_el) = child {
            if !is_static_element(child_el) {
                dynamic_child_indices.push(i);
            }
        }
    }
    dynamic_child_indices
}

fn transform_dynamic_children_with_ids<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    parent_id: usize,
    block: &mut BlockIRNode<'a>,
    dynamic_child_indices: &[usize],
    child_ids: &[usize],
) {
    let mut prev_template_backed_child: Option<(usize, usize)> = None;

    for (idx, &child_index) in dynamic_child_indices.iter().enumerate() {
        let child_id = child_ids[idx];
        let TemplateChildNode::Element(child_el) = &el.children[child_index] else {
            continue;
        };

        if is_template_backed_element(child_el) {
            if let Some((prev_child_id, prev_child_index)) = prev_template_backed_child {
                let offset =
                    count_rendered_child_nodes(&el.children, prev_child_index + 1, child_index);
                block.operation.push(OperationNode::NextRef(NextRefIRNode {
                    child_id,
                    prev_id: prev_child_id,
                    offset,
                }));
            } else {
                let offset =
                    count_rendered_child_nodes(&el.children, 0, child_index).saturating_sub(1);
                block
                    .operation
                    .push(OperationNode::ChildRef(ChildRefIRNode {
                        child_id,
                        parent_id,
                        offset,
                    }));
            }

            prev_template_backed_child = Some((child_id, child_index));
            transform_existing_element(ctx, child_el, child_id, block);
        } else {
            transform_component(
                ctx,
                child_el,
                block,
                Some(child_id),
                Some(parent_id),
                None,
                false,
            );
        }
    }
}

fn transform_existing_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let has_control_flow_children = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::If(_) | TemplateChildNode::For(_)));
    let has_dynamic_element_children = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::Element(child_el) if !is_static_element(child_el)));

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            transform_directive(ctx, dir, element_id, el, block);
        }
    }

    transform_template_ref(ctx, el, element_id, block);

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

    if has_dynamic_element_children {
        let dynamic_child_indices = collect_dynamic_child_indices(el);
        let child_ids: std::vec::Vec<usize> = dynamic_child_indices
            .iter()
            .map(|_| ctx.next_id())
            .collect();
        transform_dynamic_children_with_ids(
            ctx,
            el,
            element_id,
            block,
            &dynamic_child_indices,
            &child_ids,
        );
    }

    if has_control_flow_children {
        transform_existing_element_control_flow_children(ctx, el, element_id, block);
    }
}

fn transform_existing_element_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    for child in el.children.iter() {
        match child {
            TemplateChildNode::If(if_node) => {
                transform_if_node_into_parent(ctx, if_node, block, element_id);
            }
            TemplateChildNode::For(for_node) => {
                transform_for_node_into_parent(ctx, for_node, block, element_id);
            }
            _ => {}
        }
    }
}

fn count_rendered_child_nodes(
    children: &[TemplateChildNode<'_>],
    start: usize,
    end: usize,
) -> usize {
    let mut count = 0usize;
    for child in &children[start..=end] {
        count += count_rendered_nodes_for_child(child);
    }
    count
}

fn count_rendered_nodes_for_child(child: &TemplateChildNode<'_>) -> usize {
    match child {
        TemplateChildNode::Element(child_el) => {
            if child_el.tag_type == ElementType::Template {
                child_el
                    .children
                    .iter()
                    .map(count_rendered_nodes_for_child)
                    .sum()
            } else if is_template_backed_element(child_el) {
                1
            } else {
                0
            }
        }
        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => 1,
        _ => 0,
    }
}

/// Transform a component element (handles slots, built-in components, dynamic components, v-show)
fn transform_component<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    existing_id: Option<usize>,
    parent: Option<usize>,
    anchor: Option<usize>,
    add_return: bool,
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
                    } else if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(val_exp) = exp {
                            let key_node =
                                SimpleExpressionNode::new("$", true, SourceLocation::STUB);
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            let val_node = SimpleExpressionNode::new(
                                val_exp.content.clone(),
                                val_exp.is_static,
                                val_exp.loc.clone(),
                            );
                            values.push(Box::new_in(val_node, ctx.allocator));
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

    let element_id = existing_id.unwrap_or_else(|| ctx.next_id());

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
        parent,
        anchor,
    };

    block
        .operation
        .push(OperationNode::CreateComponent(create_component));
    if add_return {
        block.returns.push(element_id);
    }
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
