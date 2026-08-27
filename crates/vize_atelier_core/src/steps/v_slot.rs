//! v-slot directive transform.
//!
//! Transforms v-slot (# shorthand) directives for slot content.

use vize_s0::{String, ensure_sufficient_stack};

use crate::lane::TransformContext;
use crate::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
};

mod params;
#[cfg(test)]
mod tests;
mod validate;

pub use params::extract_slot_prop_names;
pub(crate) use validate::validate_v_slot_usage;

/// Check if element has v-slot directive
pub fn has_v_slot(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
}

fn find_v_slot<'a, 'b>(el: &'b ElementNode<'a>) -> Option<&'b DirectiveNode<'a>> {
    el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
        _ => None,
    })
}

/// Get the v-slot name; dynamic slots return raw source without `_ctx.` prefix.
pub fn get_slot_name(dir: &DirectiveNode<'_>, source: &str) -> String {
    match dir.arg.as_ref() {
        Some(ExpressionNode::Simple(exp)) if exp.is_static => {
            static_slot_name_with_modifiers(exp.content.into(), dir)
        }
        Some(ExpressionNode::Simple(exp)) => String::new(exp.loc.span.slice(source)),
        Some(ExpressionNode::Compound(exp)) => String::new(exp.loc.span.slice(source)),
        None => static_slot_name_with_modifiers(String::new("default"), dir),
    }
}

fn static_slot_name_with_modifiers(mut name: String, dir: &DirectiveNode<'_>) -> String {
    for modifier in dir.modifiers.iter() {
        name.push('.');
        name.push_str(modifier.content);
    }
    name
}

pub fn get_slot_props_string(dir: &DirectiveNode<'_>, source: &str) -> Option<String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => String::new(s.content),
        ExpressionNode::Compound(c) => String::new(c.loc.span.slice(source)),
    })
}

pub fn get_slot_prop_names(dir: &DirectiveNode<'_>, source: &str) -> Vec<String> {
    get_slot_props_string(dir, source)
        .map(|pattern| extract_slot_prop_names(pattern.as_str()))
        .unwrap_or_default()
}

/// Check if slot is dynamic (has dynamic name)
pub fn is_dynamic_slot(dir: &DirectiveNode<'_>) -> bool {
    if let Some(arg) = &dir.arg {
        match arg {
            ExpressionNode::Simple(exp) => !exp.is_static,
            ExpressionNode::Compound(_) => true,
        }
    } else {
        false
    }
}

fn is_slot_template(el: &ElementNode<'_>) -> bool {
    el.tag == "template" && has_v_slot(el)
}

fn has_structural_slot_directive(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if matches!(dir.name, "if" | "else-if" | "else" | "for")
        )
    })
}

fn has_implicit_child(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Comment(_) => false,
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        TemplateChildNode::Element(el) if is_slot_template(el) => false,
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .any(|branch| any_implicit_child(&branch.children)),
        TemplateChildNode::For(for_node) => any_implicit_child(&for_node.children),
        _ => true,
    }
}

/// Guarded: `v-if`/`v-for` nodes nest, so this descends once per nesting level
/// and its depth is bounded by the input (`vize_s0::recursion`).
fn any_implicit_child(children: &[TemplateChildNode<'_>]) -> bool {
    ensure_sufficient_stack(|| children.iter().any(has_implicit_child))
}

fn slot_name_is_static(dir: &DirectiveNode<'_>) -> bool {
    dir.arg.as_ref().is_none_or(|arg| match arg {
        ExpressionNode::Simple(exp) => exp.is_static,
        ExpressionNode::Compound(_) => false,
    })
}

/// Slot outlet info for codegen
#[derive(Debug)]
pub struct SlotOutletInfo {
    pub name: String,
    pub props_expr: Option<String>,
    pub has_fallback: bool,
}

/// Transform v-slot directive for slot outlet (<slot>)
pub fn transform_slot_outlet<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &DirectiveNode<'a>,
    el: &ElementNode<'a>,
) -> Option<SlotOutletInfo> {
    ctx.helper(RuntimeHelper::RenderSlot);

    // Only for <slot> elements
    if el.tag != "slot" {
        return None;
    }

    let slot_name = get_slot_name(dir, ctx.source);
    let props_expr = get_slot_props_string(dir, ctx.source);
    let has_fallback = !el.children.is_empty();

    Some(SlotOutletInfo {
        name: slot_name,
        props_expr,
        has_fallback,
    })
}

/// Slot info for component slots
#[derive(Debug)]
pub struct SlotInfo {
    pub name: String,
    pub params_expr: Option<String>,
    pub is_dynamic: bool,
}

/// Collect slot information from component children
pub fn collect_slots<'a>(el: &ElementNode<'a>, source: &str) -> Vec<SlotInfo> {
    let mut slots = Vec::new();
    let mut seen_static_slots: std::vec::Vec<String> = std::vec::Vec::new();

    for child in el.children.iter() {
        if let TemplateChildNode::Element(child_el) = child
            && child_el.tag == "template"
        {
            // Check for v-slot on template
            for prop in child_el.props.iter() {
                if let PropNode::Directive(dir) = prop
                    && dir.name == "slot"
                {
                    let name = get_slot_name(dir, source);
                    let params_expr = get_slot_props_string(dir, source);
                    let is_dynamic = is_dynamic_slot(dir);

                    if !is_dynamic
                        && seen_static_slots
                            .iter()
                            .any(|seen| seen.as_str() == name.as_str())
                    {
                        continue;
                    }
                    if !is_dynamic {
                        seen_static_slots.push(name.clone());
                    }

                    slots.push(SlotInfo {
                        name,
                        params_expr,
                        is_dynamic,
                    });
                }
            }
        }
    }

    // Check for implicit default slot
    let has_non_slot_children = el.children.iter().any(|child| {
        if let TemplateChildNode::Element(el) = child {
            !(el.tag == "template" && has_v_slot(el))
        } else {
            true
        }
    });

    if has_non_slot_children && !slots.iter().any(|s| s.name == "default") {
        slots.push(SlotInfo {
            name: String::new("default"),
            params_expr: None,
            is_dynamic: false,
        });
    }

    slots
}

/// Check if component has dynamic slots
pub fn has_dynamic_slots<'a>(el: &ElementNode<'a>) -> bool {
    for child in el.children.iter() {
        if let TemplateChildNode::Element(child_el) = child
            && child_el.tag == "template"
        {
            for prop in child_el.props.iter() {
                if let PropNode::Directive(dir) = prop
                    && dir.name == "slot"
                    && is_dynamic_slot(dir)
                {
                    return true;
                }
            }
        }
    }
    false
}
