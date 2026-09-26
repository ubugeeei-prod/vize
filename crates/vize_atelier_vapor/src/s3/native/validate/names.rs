//! Exact static prop/event names and retained computed names.

use super::super::{BindingKind, Expr};
use super::{
    Result,
    operands::{attribute_name, js, one},
};
use crate::s3::{LegacyReason, retained::Retained};
use vize_carton::Vec;
use vize_s3::operand::{Operand, OperandRole as Role, ValueKind};

pub(super) fn named<'a>(
    values: &[Operand<'a>],
    family: BindingKind,
    retained: &Retained<'_, 'a>,
) -> Result<(&'a str, Option<Expr<'a>>, Vec<'a, &'a str>)> {
    let event = family == BindingKind::Event;
    let name = one(values, Role::Name)?;
    let mut modifiers = Vec::new_in(&retained.allocator());
    for value in values.iter().filter(|value| value.role == Role::Modifier) {
        if !event || value.value.kind != ValueKind::Literal || !event_name(value.value.text) {
            return Err(LegacyReason::Binding.into());
        }
        modifiers.push(value.value.text);
    }
    if values.len() != 3 + modifiers.len() {
        return Err(LegacyReason::Binding.into());
    }
    let dynamic = match name.value.kind {
        ValueKind::Js => Some(js(retained, name)?),
        ValueKind::Literal
            if if event {
                event_name(name.value.text)
            } else {
                // `key` is checked against its loop and `is` against its component.
                matches!(name.value.text, "key" | "is") || attribute_name(name.value.text)
            } =>
        {
            None
        }
        _ => return Err(LegacyReason::Binding.into()),
    };
    Ok((name.value.text, dynamic, modifiers))
}

fn event_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'))
}
