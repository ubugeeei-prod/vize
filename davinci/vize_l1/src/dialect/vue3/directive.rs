//! Vue directive-head syntax, resolved outside the generic markup lexer.

use crate::markup::directive::{
    ArgSyntax, DirectiveName, DirectiveNameError, DirectivePrefix, DirectiveSyntax,
};
use vize_l0::Span;

mod scan;

/// Maximum simultaneous distinct delimiter runs in the allocation-free hook.
/// Repeated identical brackets share a counter rather than consuming entries.
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
            while bytes
                .get(index)
                .is_some_and(|byte| !matches!(byte, b':' | b'.' | b'['))
            {
                index += 1;
            }
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
        let mut section_start = index;
        while let Some(&byte) = bytes.get(index) {
            if byte == b'.' {
                if section_start < index {
                    result.arg = Some(ArgSyntax::Static(absolute(offset, section_start, index)));
                }
                result.modifiers = absolute(offset, index, name.len());
                return Ok(Some(result));
            }
            if byte == b'[' {
                if section_start < index {
                    result.arg = Some(ArgSyntax::Static(absolute(offset, section_start, index)));
                }
                let (end, closed) = scan::argument(bytes, index + 1)?;
                if closed || index + 1 < end {
                    result.arg = Some(ArgSyntax::Dynamic(absolute(offset, index + 1, end)));
                }
                if !closed {
                    return Ok(Some(result));
                }
                index = end + 1;
                section_start = index;
            } else {
                index += 1;
            }
        }
        if section_start < index {
            result.arg = Some(ArgSyntax::Static(absolute(offset, section_start, index)));
        }
        Ok(Some(result))
    }
}

/// The complete input range was admitted before any component is constructed.
fn absolute(offset: u32, start: usize, end: usize) -> Span {
    Span::new(offset + start as u32, offset + end as u32)
}
