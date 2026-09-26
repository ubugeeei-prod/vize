//! `v-model` on a native input/textarea/select or an ordinary component. S2 carries
//! the same reference for reads and writes, an optional static argument, and
//! the owner kind. The shared generator realizes either the DOM directive or
//! the component's prop, update listener, and modifiers prop.

use oxc_ast::ast::Expression;
use vize_carton::Vec;
use vize_s3::{
    op::OpId,
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Binding, BindingKind, Content, Node};
use super::{
    Result,
    component::component_prop,
    operands::{js, one},
};
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained};

pub(super) fn model<'a>(
    values: &[Operand<'a>],
    retained: &Retained<'_, 'a>,
) -> Result<(OpId, Binding<'a>)> {
    let alloc = retained.allocator();
    let kind = one(values, Role::BindingKind)?;
    let target = kind.target.ok_or(LegacyReason::Structure)?;
    let read_operand = one(values, Role::ModelRead)?;
    let read = read_operand.value;
    let write = one(values, Role::ModelWrite)?.value;
    let name = one(values, Role::Name)?.value;
    let mut element = None;
    let mut modifiers = Vec::new_in(&alloc);
    for value in values {
        if value.target != Some(target) || value.region.is_some() {
            return Err(LegacyReason::Binding.into());
        }
        match (value.role, value.name, value.value.kind) {
            (Role::BindingKind | Role::ModelRead | Role::ModelWrite, None, _) => {}
            (Role::Name, None, ValueKind::Absent | ValueKind::Literal) => {}
            (Role::ModelAttribute, Some("element-kind"), ValueKind::Literal) => {
                if element.replace(value.value.text).is_some() {
                    return Err(LegacyReason::Binding.into());
                }
            }
            (
                Role::ModelAttribute,
                Some(name @ ("lazy" | "number" | "trim")),
                ValueKind::Absent,
            ) if element != Some("component") && !modifiers.contains(&name) => {
                modifiers.push(name);
            }
            (Role::ModelAttribute, Some(name), ValueKind::Absent)
                if element == Some("component")
                    && component_prop(name)
                    && !modifiers.contains(&name) =>
            {
                modifiers.push(name);
            }
            _ => return Err(LegacyReason::Binding.into()),
        }
    }
    let name = match (element, name.kind) {
        (Some("component"), ValueKind::Absent) => "modelValue",
        (Some("component"), ValueKind::Literal) if component_prop(name.text) => name.text,
        (Some("input" | "textarea" | "select"), ValueKind::Absent) => "",
        _ => return Err(LegacyReason::Binding.into()),
    };
    // Reads and writes share the same authored assignment target. Retained
    // member ASTs preserve computed keys without reparsing generated handlers.
    let text = read.text.trim();
    if read.kind != ValueKind::Js
        || write.kind != ValueKind::Js
        || write.text.trim() != text
        || text.starts_with('_')
        || text.starts_with('$')
        || text.contains("$event")
        || !matches!(element, Some("input" | "textarea" | "select" | "component"))
    {
        return Err(LegacyReason::Binding.into());
    }
    let value = js(retained, read_operand)?;
    if let Some(js) = value.js {
        match js.ast {
            Expression::Identifier(_) => {}
            Expression::StaticMemberExpression(member) if !member.optional => {}
            Expression::ComputedMemberExpression(member) if !member.optional => {}
            _ => return Err(LegacyReason::Binding.into()),
        }
    }
    Ok((
        target,
        Binding {
            kind: BindingKind::Model,
            name,
            dynamic_name: None,
            value,
            modifiers,
            merge: None,
            model_element: element,
            position: 0,
            spans: [
                (read.span.start, read.span.end),
                (read.span.start, read.span.end),
            ],
        },
    ))
}

/// A native model binds to an `<input>` with static `type`, or an empty
/// `<textarea>`, or a `<select>` checked by the separate parsing guard.
/// Component models become props before this check.
/// Textarea contents are RCDATA, so child text needs its own parser contract.
pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let bound = |kind: BindingKind, name: &str| {
            node.bindings
                .iter()
                .any(|binding| binding.kind == kind && binding.name == name)
        };
        let model = node
            .bindings
            .iter()
            .find(|binding| binding.kind == BindingKind::Model);
        if let Some(model) = model
            && !matches!(node.content, Content::Element { tag, .. } if model.model_element == Some(tag))
        {
            return Err(AdmissionFailure::Invalid(
                "model element kind disagrees with target",
            ));
        }
        match &node.content {
            Content::Element { tag: "input", .. }
                if model.is_some() && bound(BindingKind::Prop, "type") =>
            {
                return Err(LegacyReason::Binding.into());
            }
            Content::Element {
                tag: "textarea",
                attributes,
                ..
            } if model.is_none()
                || !node.children.is_empty()
                || attributes.iter().any(|(name, ..)| *name == "value")
                || bound(BindingKind::Prop, "value")
                || bound(BindingKind::Html, "")
                || bound(BindingKind::Text, "") =>
            {
                return Err(LegacyReason::Binding.into());
            }
            Content::Element {
                tag: "input" | "textarea" | "select",
                ..
            } => {}
            _ if model.is_some() => return Err(LegacyReason::Binding.into()),
            _ => {}
        }
    }
    Ok(())
}
