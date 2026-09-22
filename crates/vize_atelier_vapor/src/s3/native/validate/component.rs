//! Operand schemas for components and slot outlets: the tag or static name,
//! and static attributes as literal props in authored order.

use oxc_allocator::HashSet;
use vize_carton::{Allocator, Vec};
use vize_s3::operand::{Operand, OperandRole as Role, ValueKind};

use super::super::{Content, Expr, Prop};
use super::{Result, ident::Folded, operands::one};
use crate::s3::LegacyReason;

/// A resolved component: its tag and static attributes (as literal props).
pub(super) fn component<'a>(values: &[Operand<'a>], alloc: &'a Allocator) -> Result<Content<'a>> {
    let tag = one(values, Role::Tag)?;
    if tag.value.kind != ValueKind::Literal || !component_tag(tag.value.text) {
        return Err(LegacyReason::Component.into());
    }
    let props = static_props(values, Role::Tag, alloc)?;
    Ok(Content::Component {
        tag: tag.value.text,
        tag_span: (tag.value.span.start, tag.value.span.end),
        props,
        is: None,
    })
}

/// A `<slot>` outlet with a static name and static attribute props.
pub(super) fn outlet<'a>(values: &[Operand<'a>], alloc: &'a Allocator) -> Result<Content<'a>> {
    let name = one(values, Role::Name)?;
    if name.value.kind != ValueKind::Literal || name.target.is_some() {
        return Err(LegacyReason::Component.into());
    }
    let props = static_props(values, Role::Name, alloc)?;
    Ok(Content::Outlet {
        name: name.value.text,
        props,
    })
}

fn static_props<'a>(
    values: &[Operand<'a>],
    head: Role,
    alloc: &'a Allocator,
) -> Result<Vec<'a, Prop<'a>>> {
    let mut props = Vec::new_in(&alloc);
    let mut seen = HashSet::with_capacity_in(values.len(), alloc.as_oxc());
    for value in values {
        if value.target.is_some() || value.region.is_some() {
            return Err(LegacyReason::Structure.into());
        }
        match value.role {
            role if role == head && value.name.is_none() => {}
            Role::Attribute => {
                let name = value.name.ok_or(LegacyReason::Binding)?;
                // The legacy parser reports repeats case-insensitively.
                if !component_prop(name) || !seen.insert(Folded(name)) {
                    return Err(LegacyReason::Component.into());
                }
                let literal = match value.value.kind {
                    ValueKind::Absent => None,
                    ValueKind::Literal if !value.value.text.contains('&') => {
                        Some(Expr::plain(value.value.text))
                    }
                    _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
                };
                props.push(Prop {
                    key: name,
                    value: literal,
                    dynamic: false,
                    handler: false,
                    position: value.value.span.start,
                });
            }
            _ => return Err(LegacyReason::Structure.into()),
        }
    }
    Ok(props)
}

/// Ordinary user components and `<component :is>` (its `:is` checked once
/// bindings attach). Built-ins and self references have their own runtime
/// contracts and stay on the legacy lane.
fn component_tag(tag: &str) -> bool {
    !matches!(
        tag,
        "Component"
            | "Teleport"
            | "teleport"
            | "KeepAlive"
            | "keep-alive"
            | "Suspense"
            | "suspense"
            | "Transition"
            | "transition"
            | "TransitionGroup"
            | "transition-group"
            | "Self"
    ) && tag.starts_with(|c: char| c.is_ascii_alphabetic())
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Component and outlet prop names. Keys that the runtime treats specially
/// (`key`, `ref`, `is`, legacy `slot`) are not plain props.
pub(super) fn component_prop(name: &str) -> bool {
    !matches!(name, "key" | "ref" | "ref_for" | "ref_key" | "is" | "slot")
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'))
}
