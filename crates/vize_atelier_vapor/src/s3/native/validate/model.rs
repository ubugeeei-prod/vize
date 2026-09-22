//! `v-model` on a native input: one reference read and written in place, the
//! element kind S2 recorded, and `lazy`/`number`/`trim` modifiers. The shared
//! generator picks the text, checkbox or radio helper from the static `type`.

use vize_s3::{
    op::OpId,
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Binding, BindingKind, Content, Expr, Node};
use super::{Result, operands::one, operands::reference};
use crate::s3::LegacyReason;

pub(super) fn model<'a>(values: &[Operand<'a>]) -> Result<(OpId, Binding<'a>)> {
    let kind = one(values, Role::BindingKind)?;
    let target = kind.target.ok_or(LegacyReason::Structure)?;
    let read = one(values, Role::ModelRead)?.value;
    let write = one(values, Role::ModelWrite)?.value;
    let mut element = None;
    let mut modifiers = std::vec::Vec::new();
    for value in values {
        if value.target != Some(target) || value.region.is_some() {
            return Err(LegacyReason::Binding.into());
        }
        match (value.role, value.name, value.value.kind) {
            (Role::BindingKind | Role::ModelRead | Role::ModelWrite, None, _) => {}
            (Role::Name, None, ValueKind::Absent) => {}
            (Role::ModelAttribute, Some("element-kind"), ValueKind::Literal) => {
                element = Some(value.value.text);
            }
            (
                Role::ModelAttribute,
                Some(name @ ("lazy" | "number" | "trim")),
                ValueKind::Absent,
            ) if !modifiers.contains(&name) => {
                modifiers.push(name);
            }
            _ => return Err(LegacyReason::Binding.into()),
        }
    }
    // A plain reference is read and assigned as authored; `$event`, reserved
    // context roots and computed targets stay legacy.
    let text = read.text.trim();
    if read.kind != ValueKind::Js
        || write.kind != ValueKind::Js
        || write.text.trim() != text
        || !reference(text)
        || text.starts_with('_')
        || text.starts_with('$')
        || element != Some("input")
    {
        return Err(LegacyReason::Binding.into());
    }
    Ok((
        target,
        Binding {
            kind: BindingKind::Model,
            name: "",
            value: Expr::plain(text),
            modifiers,
            merge: None,
            position: 0,
        },
    ))
}

/// A model binds only to an `<input>` whose `type`, when present, is static.
pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let bound = |kind: BindingKind, name: &str| {
            node.bindings
                .iter()
                .any(|binding| binding.kind == kind && binding.name == name)
        };
        if bound(BindingKind::Model, "")
            && (!matches!(node.content, Content::Element { tag: "input", .. })
                || bound(BindingKind::Prop, "type"))
        {
            return Err(LegacyReason::Binding.into());
        }
    }
    Ok(())
}
