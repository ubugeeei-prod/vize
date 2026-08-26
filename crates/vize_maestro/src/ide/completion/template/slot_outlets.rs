//! Template `<slot>` outlet extraction for component metadata.

use std::collections::BTreeSet;

use vize_relief::{ElementNode, PropNode, TemplateChildNode};

pub(super) fn extract_template_slot_names(template: &str) -> Vec<String> {
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut names = Vec::new();
    let mut seen = BTreeSet::new();

    for child in root.children.iter() {
        collect_template_slot_outlets(child, &mut names, &mut seen);
    }

    names
}

fn collect_template_slot_outlets(
    child: &TemplateChildNode<'_>,
    names: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
) {
    match child {
        TemplateChildNode::Element(el) => collect_element_slot_outlets(el, names, seen),
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter() {
                for child in branch.children.iter() {
                    collect_template_slot_outlets(child, names, seen);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in branch.children.iter() {
                collect_template_slot_outlets(child, names, seen);
            }
        }
        TemplateChildNode::For(for_node) => {
            for child in for_node.children.iter() {
                collect_template_slot_outlets(child, names, seen);
            }
        }
        TemplateChildNode::Text(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Interpolation(_)
        | TemplateChildNode::TextCall(_)
        | TemplateChildNode::CompoundExpression(_)
        | TemplateChildNode::Hoisted(_) => {}
    }
}

fn collect_element_slot_outlets(
    el: &ElementNode<'_>,
    names: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
) {
    if el.tag == "slot" {
        let name = static_slot_outlet_name(el);
        if seen.insert(name.clone()) {
            names.push(name);
        }
    }

    for child in el.children.iter() {
        collect_template_slot_outlets(child, names, seen);
    }
}

fn static_slot_outlet_name(el: &ElementNode<'_>) -> String {
    el.props
        .iter()
        .find_map(|prop| match prop {
            PropNode::Attribute(attr) if attr.name == "name" => {
                attr.value.as_ref().map(|value| value.content.to_string())
            }
            _ => None,
        })
        .unwrap_or_else(|| "default".to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_template_slot_names;

    #[test]
    fn extracts_slot_outlets_from_template_ast() {
        assert_eq!(
            extract_template_slot_names(r#"<slot /><slot name="header" /><slot name="header" />"#),
            vec!["default".to_string(), "header".to_string()]
        );
    }

    #[test]
    fn ignores_slot_like_text_in_comments_and_attribute_values() {
        assert_eq!(
            extract_template_slot_names(
                r#"<div data-example="<slot name='ghost'></slot>"><!-- <slot name="ghost" /> --><slot name="real" /></div>"#
            ),
            vec!["real".to_string()]
        );
    }

    #[test]
    fn extracts_slots_nested_in_control_flow() {
        assert_eq!(
            extract_template_slot_names(
                r#"<template v-if="ok"><slot name="header" /></template><li v-for="item in items"><slot /></li>"#
            ),
            vec!["header".to_string(), "default".to_string()]
        );
    }
}
