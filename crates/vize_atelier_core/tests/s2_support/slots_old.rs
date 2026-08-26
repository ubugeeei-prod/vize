//! The legacy half of the slot projection ([`super::slots`]): the
//! transformed legacy tree walked into the shared unit/outlet shape,
//! through the shipped helpers themselves (`get_slot_name`,
//! `get_slot_props_string`, `is_dynamic_slot`) so the projection is the
//! legacy lane's own reading, not a re-implementation.

use vize_atelier_core::{
    ElementNode, ExpressionNode, PropNode, TemplateChildNode, get_slot_name, get_slot_props_string,
    has_v_slot, is_dynamic_slot,
};
use vize_s0::{String, is_native_tag};

use super::slots::{PGroup, PName, POutlet, PUnit};

fn trimmed(text: &str) -> String {
    String::from(text.trim())
}

fn slot_dirs<'a, 'b>(
    el: &'b ElementNode<'a>,
) -> impl Iterator<Item = &'b vize_atelier_core::DirectiveNode<'a>> {
    el.props.iter().filter_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
        _ => None,
    })
}

fn is_slot_template(el: &ElementNode<'_>) -> bool {
    el.tag == "template" && has_v_slot(el)
}

fn old_group(dir: &vize_atelier_core::DirectiveNode<'_>, source: &str) -> PGroup {
    let text = get_slot_name(dir, source);
    let name = if is_dynamic_slot(dir) {
        PName::Dynamic(trimmed(text.as_str()))
    } else {
        PName::Static(String::from(text.as_str()))
    };
    let params = get_slot_props_string(dir, source)
        .map(|text| trimmed(text.as_str()))
        .filter(|text| !text.is_empty());
    PGroup {
        name,
        invented: dir.arg.is_none(),
        params,
    }
}

/// The shared implicit-content predicate on a legacy child.
/// Conditional slot carriers (`<template v-if v-slot>`) are the recorded
/// wrapper gap: they are counted on the unit, never treated as implicit
/// default content (S2 drops them at lowering).
fn old_shared_content(child: &TemplateChildNode<'_>) -> bool {
    if old_conditional_carrier(child) {
        return false;
    }
    match child {
        TemplateChildNode::Element(el) => !is_slot_template(el),
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        _ => true,
    }
}

/// The raw legacy predicate (`collect_slots`' `has_non_slot_children`).
fn old_raw_content(child: &TemplateChildNode<'_>) -> bool {
    if old_conditional_carrier(child) {
        return false;
    }
    match child {
        TemplateChildNode::Element(el) => !is_slot_template(el),
        _ => true,
    }
}

fn old_conditional_carrier(child: &TemplateChildNode<'_>) -> bool {
    let holds = |children: &[TemplateChildNode<'_>]| {
        children
            .iter()
            .any(|inner| matches!(inner, TemplateChildNode::Element(el) if is_slot_template(el)))
    };
    match child {
        TemplateChildNode::If(node) => node.branches.iter().any(|branch| holds(&branch.children)),
        TemplateChildNode::For(node) => holds(&node.children),
        _ => false,
    }
}

fn old_unit(el: &ElementNode<'_>, source: &str) -> PUnit {
    let own: Vec<PGroup> = slot_dirs(el).map(|dir| old_group(dir, source)).collect();
    let mut groups = own.clone();
    let mut seen_static: Vec<String> = Vec::new();
    for child in el.children.iter() {
        let TemplateChildNode::Element(child_el) = child else {
            continue;
        };
        if !is_slot_template(child_el) {
            continue;
        }
        for dir in slot_dirs(child_el) {
            let group = old_group(dir, source);
            if let PName::Static(text) = &group.name {
                if seen_static.iter().any(|seen| seen == text) {
                    continue;
                }
                seen_static.push(text.clone());
            }
            groups.push(group);
        }
    }
    let shared = el.children.iter().any(old_shared_content);
    let raw = el.children.iter().any(old_raw_content);
    let no_default = !groups.iter().any(|group| match &group.name {
        PName::Static(text) | PName::Dynamic(text) => text.as_str() == "default",
    });
    let mut filler_default = false;
    if own.is_empty() && no_default {
        if shared {
            groups.push(PGroup {
                name: PName::Static(String::from("default")),
                invented: true,
                params: None,
            });
        } else if raw {
            filler_default = true;
        }
    }
    let forwarded = el.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "slots" && dir.arg.is_none() && dir.exp.is_some()
        )
    });
    PUnit {
        groups,
        conditional: el.children.iter().any(old_conditional_carrier),
        forwarded,
        filler_default,
    }
}

fn old_outlet(el: &ElementNode<'_>, source: &str) -> POutlet {
    // The shipped resolution (`codegen/slots/outlet.rs`): the first
    // `name` attribute (value-less reads as `default`) or the first
    // `:name` with an expression wins; otherwise the implicit name.
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) if attr.name == "name" => {
                let value = attr
                    .value
                    .as_ref()
                    .map(|value| value.content)
                    .unwrap_or("default");
                return POutlet {
                    name: PName::Static(String::from(value)),
                };
            }
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(
                        dir.arg.as_ref(),
                        Some(ExpressionNode::Simple(exp)) if exp.is_static && exp.content == "name"
                    ) =>
            {
                if let Some(exp) = dir.exp.as_ref() {
                    let text = match exp {
                        ExpressionNode::Simple(simple) => trimmed(simple.content),
                        ExpressionNode::Compound(compound) => {
                            trimmed(compound.loc.span.slice(source))
                        }
                    };
                    return POutlet {
                        name: PName::Dynamic(text),
                    };
                }
            }
            _ => {}
        }
    }
    POutlet {
        name: PName::Static(String::from("default")),
    }
}

/// Collect every unit and outlet under `children`, outer before nested,
/// document order — the same order the S2 projection walks.
pub fn collect_old(
    children: &[TemplateChildNode<'_>],
    source: &str,
    units: &mut Vec<PUnit>,
    outlets: &mut Vec<POutlet>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                if el.tag == "slot" {
                    outlets.push(old_outlet(el, source));
                } else if !is_native_tag(el.tag)
                    && (slot_dirs(el).next().is_some()
                        || el.children.iter().any(|child| {
                            matches!(child, TemplateChildNode::Element(inner) if is_slot_template(inner))
                        }))
                {
                    units.push(old_unit(el, source));
                }
                collect_old(&el.children, source, units, outlets);
            }
            TemplateChildNode::If(node) => {
                for branch in node.branches.iter() {
                    collect_old(&branch.children, source, units, outlets);
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                collect_old(&branch.children, source, units, outlets);
            }
            TemplateChildNode::For(node) => collect_old(&node.children, source, units, outlets),
            _ => {}
        }
    }
}
