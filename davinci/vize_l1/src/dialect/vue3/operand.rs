//! Conditional operand selection belongs to the Vue dialect.

use super::VueDirectives;
use crate::markup::{
    ArgSyntax, DirectiveName, DirectiveNameError, DirectivePrefix, DirectiveSyntax,
};
use vize_l0::Span;

/// The actual complete conditional directive spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeConditionKind {
    If,
    ElseIf,
}

pub(crate) fn conditional_head(
    raw: &str,
    offset: u32,
    source: &str,
) -> Result<Option<NativeConditionKind>, DirectiveNameError> {
    let parts = VueDirectives.decompose(raw, offset)?;
    Ok(conditional_parts(parts, offset + raw.len() as u32, source))
}

// Reuse the actual dialect decomposition at the same original header event.
pub(crate) fn conditional_parts(
    parts: Option<DirectiveName>,
    name_end: u32,
    source: &str,
) -> Option<NativeConditionKind> {
    let head = parts?;
    // Only a complete argument/modifier-free conditional head is admitted.
    // The provider retains unsupported variants in their original surface.
    if head.prefix != DirectivePrefix::Full
        || head.arg.is_some()
        || head.modifiers.start != head.modifiers.end
        || head.name.end != name_end
    {
        return None;
    }
    match head.name.slice(source) {
        "if" => Some(NativeConditionKind::If),
        "else-if" => Some(NativeConditionKind::ElseIf),
        _ => None,
    }
}

/// Only the complete, argument/modifier-free Vue v-for header selects this owner.
pub(crate) fn for_head(raw: &str, offset: u32, source: &str) -> Result<bool, DirectiveNameError> {
    let Some(head) = VueDirectives.decompose(raw, offset)? else {
        return Ok(false);
    };
    Ok(head.prefix == DirectivePrefix::Full
        && head.arg.is_none()
        && head.modifiers.start == head.modifiers.end
        && head.name.end == offset + raw.len() as u32
        && head.name.slice(source) == "for")
}

/// First event head family: explicit complete static names with no modifiers.
pub(crate) fn static_event_head(
    raw: &str,
    offset: u32,
    source: &str,
) -> Result<Option<Span>, DirectiveNameError> {
    let Some(head) = VueDirectives.decompose(raw, offset)? else {
        return Ok(None);
    };
    let expected = match head.prefix {
        DirectivePrefix::On => offset.checked_add(1),
        DirectivePrefix::Full if head.name.slice(source) == "on" => head.name.end.checked_add(1),
        _ => return Ok(None),
    };
    let Some(ArgSyntax::Static(argument)) = head.arg else {
        return Ok(None);
    };
    if head.modifiers.start != head.modifiers.end
        || Some(argument.start) != expected
        || argument.end != offset + raw.len() as u32
        || argument.start == argument.end
    {
        return Ok(None);
    }
    Ok(Some(argument))
}
