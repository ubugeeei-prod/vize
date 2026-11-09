//! Vue directive-head syntax, resolved outside the generic markup lexer.

use crate::markup::directive::{
    ArgSyntax, DirectiveName, DirectiveNameError, DirectivePrefix, DirectiveSyntax,
};
use vize_l0::{Allocator, Span, StringBuilder};

mod scan;
#[cfg(test)]
mod tests;

/// Historical delimiter-run bound, retained for published Rust consumers.
/// Vue directive names are no longer limited by expression nesting.
pub const MAX_DIRECTIVE_DELIMITER_RUNS: usize = 64;

/// Vue 3 directive-name syntax.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VueDirectives;

impl DirectiveSyntax for VueDirectives {
    fn decompose(
        &self,
        name: &str,
        offset: u32,
    ) -> Result<Option<DirectiveName>, DirectiveNameError> {
        let length = u32::try_from(name.len()).map_err(|_| DirectiveNameError::OffsetOverflow)?;
        offset
            .checked_add(length)
            .ok_or(DirectiveNameError::OffsetOverflow)?;
        let bytes = name.as_bytes();
        let (prefix, name_start, mut index) = match bytes.first() {
            Some(b'v') if bytes.get(1) == Some(&b'-') => (DirectivePrefix::Full, 2, 2),
            Some(b':') => (DirectivePrefix::Bind, 0, 1),
            Some(b'.') => (DirectivePrefix::Prop, 0, 1),
            Some(b'@') => (DirectivePrefix::On, 0, 1),
            Some(b'#') => (DirectivePrefix::Slot, 0, 1),
            _ => return Ok(None),
        };
        let name_end = if prefix == DirectivePrefix::Full {
            index = full_head_end(bytes);
            index
        } else {
            0
        };
        let mut result = DirectiveName {
            prefix,
            name: absolute(offset, name_start, name_end),
            arg: None,
            modifiers: absolute(offset, name.len(), name.len()),
        };
        if prefix == DirectivePrefix::Full && bytes.get(index) == Some(&b':') {
            index += 1;
        }
        let (end, closed) = scan::argument(bytes, index);
        if index < end {
            result.arg = Some(if bytes.get(index) == Some(&b'[') {
                // Vue classifies the whole argument by its first byte and
                // drops its last byte only after leaving the dynamic state.
                ArgSyntax::Dynamic(absolute(offset, index + 1, end - usize::from(closed)))
            } else {
                ArgSyntax::Static(absolute(offset, index, end))
            });
        }
        result.modifiers = absolute(offset, end, name.len());
        Ok(Some(result))
    }
}

/// The directive a Vue attribute name spells. L1→L2 lowering, the L1
/// surface policies and the formatter select directives through this table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveKind {
    If,
    ElseIf,
    Else,
    For,
    Model,
    Slot,
    Bind,
    On,
    Html,
    Text,
    Show,
    Cloak,
    Once,
    Pre,
    Memo,
    /// A user-defined directive (`v-focus`).
    Custom,
}

impl DirectiveKind {
    /// The directive a full `v-` head spells, from its name without `v-`.
    pub fn from_full_name(name: &str) -> Self {
        match name {
            "if" => Self::If,
            "else-if" => Self::ElseIf,
            "else" => Self::Else,
            "for" => Self::For,
            "model" => Self::Model,
            "slot" => Self::Slot,
            "bind" => Self::Bind,
            "on" => Self::On,
            "html" => Self::Html,
            "text" => Self::Text,
            "show" => Self::Show,
            "cloak" => Self::Cloak,
            "once" => Self::Once,
            "pre" => Self::Pre,
            "memo" => Self::Memo,
            _ => Self::Custom,
        }
    }

    /// The directive `head` spells; `source` is the text its spans index.
    pub fn of(head: &DirectiveName, source: &str) -> Self {
        match head.prefix {
            DirectivePrefix::Bind | DirectivePrefix::Prop => Self::Bind,
            DirectivePrefix::On => Self::On,
            DirectivePrefix::Slot => Self::Slot,
            DirectivePrefix::Full => Self::from_full_name(head.name.slice(source)),
        }
    }
}

/// Freeze the complete name after Vue's `v-pre` boundary. Only the full
/// directive head's argument separator disappears; the argument and modifiers
/// are opaque authored bytes, including their quotes and brackets.
pub fn frozen_attribute_name<'a>(allocator: &'a Allocator, authored: &'a str) -> &'a str {
    if !authored.starts_with("v-") {
        return authored;
    }
    let end = full_head_end(authored.as_bytes());
    if authored.as_bytes().get(end) != Some(&b':') {
        return authored;
    }
    let Some((head, tail)) = authored.split_at_checked(end) else {
        return authored;
    };
    let Some(tail) = tail.strip_prefix(':') else {
        return authored;
    };
    let mut out = StringBuilder::with_capacity_in(authored.len() - 1, allocator);
    out.push_str(head);
    out.push_str(tail);
    out.into_str()
}

/// The existing full directive head boundary, shared without scanning its
/// argument. Every admitted name has the ASCII `v-` prefix.
fn full_head_end(bytes: &[u8]) -> usize {
    let mut index = 2;
    while bytes
        .get(index)
        .is_some_and(|byte| !matches!(byte, b':' | b'.'))
    {
        index += 1;
    }
    index
}

/// The complete input range was admitted before any component is constructed.
fn absolute(offset: u32, start: usize, end: usize) -> Span {
    Span::new(offset + start as u32, offset + end as u32)
}
