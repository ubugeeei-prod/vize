//! Conditional operand selection belongs to the Vue dialect.

use super::VueDirectives;
use crate::markup::{DirectiveNameError, DirectivePrefix, DirectiveSyntax};

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
    let Some(head) = VueDirectives.decompose(raw, offset)? else {
        return Ok(None);
    };
    // Only a complete argument/modifier-free conditional head is admitted.
    // The provider retains unsupported variants in their original surface.
    if head.prefix != DirectivePrefix::Full
        || head.arg.is_some()
        || head.modifiers.start != head.modifiers.end
        || head.name.end != offset + raw.len() as u32
    {
        return Ok(None);
    }
    Ok(match head.name.slice(source) {
        "if" => Some(NativeConditionKind::If),
        "else-if" => Some(NativeConditionKind::ElseIf),
        _ => None,
    })
}
