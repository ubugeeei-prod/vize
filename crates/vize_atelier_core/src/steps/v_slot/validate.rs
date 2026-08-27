//! `v-slot` placement and slot-template structure validation.

use vize_s0::{String, is_builtin_directive};

use super::{
    find_v_slot, get_slot_name, has_implicit_child, has_structural_slot_directive,
    slot_name_is_static,
};
use crate::errors::ErrorCode;
use crate::lane::TransformContext;
use crate::{ElementNode, ElementType, PropNode, SourceLocation, TemplateChildNode};

/// Validate v-slot placement and slot-template structure.
pub(crate) fn validate_v_slot_usage(ctx: &mut TransformContext<'_>, el: &ElementNode<'_>) {
    let own_slot = find_v_slot(el);

    if let Some(dir) = own_slot
        && el.tag_type != ElementType::Component
        && el.tag != "template"
    {
        ctx.on_error(ErrorCode::VSlotMisplaced, Some(dir.loc.clone()));
    }

    if el.tag_type == ElementType::Slot || el.tag == "slot" {
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && !is_builtin_directive(dir.name)
            {
                ctx.on_error(
                    ErrorCode::VSlotUnexpectedDirectiveOnSlotOutlet,
                    Some(dir.loc.clone()),
                );
            }
        }
    }

    if el.tag_type != ElementType::Component || el.children.is_empty() {
        return;
    }

    let mut seen_static_slots: std::vec::Vec<String> = std::vec::Vec::new();
    let mut has_template_slots = false;
    let mut has_named_default_slot = false;
    let mut first_implicit_default_loc: Option<SourceLocation> = None;

    for child in el.children.iter() {
        let TemplateChildNode::Element(child_el) = child else {
            if first_implicit_default_loc.is_none() && has_implicit_child(child) {
                first_implicit_default_loc = Some(child.loc().clone());
            }
            continue;
        };

        let Some(slot_dir) = find_v_slot(child_el) else {
            if first_implicit_default_loc.is_none() && has_implicit_child(child) {
                first_implicit_default_loc = Some(child.loc().clone());
            }
            continue;
        };

        if child_el.tag != "template" {
            continue;
        }

        if own_slot.is_some() {
            ctx.on_error(ErrorCode::VSlotMixedSlotUsage, Some(slot_dir.loc.clone()));
            break;
        }

        has_template_slots = true;

        if !has_structural_slot_directive(child_el) && slot_name_is_static(slot_dir) {
            let slot_name = get_slot_name(slot_dir, ctx.source);
            if seen_static_slots
                .iter()
                .any(|seen| seen.as_str() == slot_name.as_str())
            {
                ctx.on_error(
                    ErrorCode::VSlotDuplicateSlotNames,
                    Some(slot_dir.loc.clone()),
                );
                continue;
            }
            if slot_name.as_str() == "default" {
                has_named_default_slot = true;
            }
            seen_static_slots.push(slot_name);
        }
    }

    if own_slot.is_none()
        && has_template_slots
        && has_named_default_slot
        && let Some(loc) = first_implicit_default_loc
    {
        ctx.on_error(ErrorCode::VSlotExtraneousDefaultSlotChildren, Some(loc));
    }
}
