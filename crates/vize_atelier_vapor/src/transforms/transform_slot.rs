//! Slot transform for Vapor mode.
//!
//! Transforms slot-related nodes (slot outlets and slot content).

use vize_carton::{Box, Bump, Vec};

use crate::ir::{BlockIRNode, IRSlot, OperationNode, SlotOutletIRNode};
use vize_atelier_core::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

/// Transform slot outlet (<slot>) to IR
pub fn transform_slot_outlet<'a>(
    allocator: &'a Bump,
    el: &ElementNode<'a>,
    element_id: usize,
    fallback: Option<BlockIRNode<'a>>,
) -> OperationNode<'a> {
    let name = get_slot_outlet_name(allocator, el);
    let props = get_slot_outlet_props(allocator, el);

    let slot_outlet = SlotOutletIRNode {
        id: element_id,
        name,
        props,
        fallback,
    };

    OperationNode::SlotOutlet(slot_outlet)
}

/// Get slot outlet name from element
fn get_slot_outlet_name<'a>(
    allocator: &'a Bump,
    el: &ElementNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    // Look for name attribute or v-bind:name
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name" {
                    if let Some(ref value) = attr.value {
                        let node = SimpleExpressionNode::new(
                            value.content.clone(),
                            true,
                            SourceLocation::STUB,
                        );
                        return Box::new_in(node, allocator);
                    }
                }
            }
            PropNode::Directive(dir) => {
                if dir.name == "bind" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(arg_exp) = arg {
                            if arg_exp.content == "name" {
                                if let Some(ref exp) = dir.exp {
                                    return extract_expression(allocator, exp);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Default slot name
    let node = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
    Box::new_in(node, allocator)
}

/// Get slot outlet props
fn get_slot_outlet_props<'a>(
    allocator: &'a Bump,
    el: &ElementNode<'a>,
) -> Vec<'a, crate::ir::IRProp<'a>> {
    let mut props = Vec::new_in(allocator);

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "bind" && dir.arg.is_some() {
                if let Some(ref arg) = dir.arg {
                    if let ExpressionNode::Simple(arg_exp) = arg {
                        // Skip name binding
                        if arg_exp.content == "name" {
                            continue;
                        }

                        let key = Box::new_in(
                            SimpleExpressionNode::new(
                                arg_exp.content.clone(),
                                arg_exp.is_static,
                                arg_exp.loc.clone(),
                            ),
                            allocator,
                        );

                        let mut values = Vec::new_in(allocator);
                        if let Some(ref exp) = dir.exp {
                            values.push(extract_expression(allocator, exp));
                        }

                        props.push(crate::ir::IRProp {
                            key,
                            values,
                            is_component: false,
                        });
                    }
                }
            }
        }
    }

    props
}

/// Collect slots from component children
pub fn collect_component_slots<'a>(
    allocator: &'a Bump,
    el: &ElementNode<'a>,
    transform_children: impl Fn(&'a Bump, &[TemplateChildNode<'a>]) -> BlockIRNode<'a>,
) -> Vec<'a, IRSlot<'a>> {
    let mut slots = Vec::new_in(allocator);

    for child in el.children.iter() {
        if let TemplateChildNode::Element(child_el) = child {
            if child_el.tag == "template" {
                // Look for v-slot directive
                for prop in child_el.props.iter() {
                    if let PropNode::Directive(dir) = prop {
                        if dir.name == "slot" {
                            let name = get_slot_name(allocator, dir);
                            let fn_exp = get_slot_params(allocator, dir);
                            let block = transform_children(allocator, &child_el.children);

                            slots.push(IRSlot {
                                name,
                                fn_exp,
                                block,
                            });
                        }
                    }
                }
            }
        }
    }

    // Check for implicit default slot
    let has_non_slot_children = el.children.iter().any(|child| {
        if let TemplateChildNode::Element(child_el) = child {
            !(child_el.tag == "template" && has_v_slot(child_el))
        } else {
            true
        }
    });

    if has_non_slot_children && !slots.iter().any(|s| s.name.content == "default") {
        let default_name = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
        let default_block = transform_children(allocator, &el.children);

        slots.push(IRSlot {
            name: Box::new_in(default_name, allocator),
            fn_exp: None,
            block: default_block,
        });
    }

    slots
}

/// Check if element has v-slot directive
fn has_v_slot(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
}

/// Get slot name from v-slot directive
fn get_slot_name<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    if let Some(ref arg) = dir.arg {
        extract_expression(allocator, arg)
    } else {
        let node = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
        Box::new_in(node, allocator)
    }
}

/// Get slot params from v-slot directive
fn get_slot_params<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    dir.exp
        .as_ref()
        .map(|exp| extract_expression(allocator, exp))
}

/// Extract expression helper
fn extract_expression<'a>(
    allocator: &'a Bump,
    exp: &ExpressionNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match exp {
        ExpressionNode::Simple(simple) => {
            let node = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            Box::new_in(node, allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            Box::new_in(node, allocator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::has_v_slot;
    use vize_atelier_core::{parser::parse, TemplateChildNode};
    use vize_carton::Bump;

    #[test]
    fn test_has_v_slot() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, r#"<template v-slot:header>content</template>"#);

        if let TemplateChildNode::Element(el) = &root.children[0] {
            assert!(has_v_slot(el));
        }
    }
}
