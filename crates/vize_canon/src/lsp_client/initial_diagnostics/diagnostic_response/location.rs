//! TypeScript beta's redundant ECMA location metadata, checked against OWNED text.
//! Primary: TypeScript@3e9236361fa97a9e6c685f4833ac457ae2d8c50a tsc/internal/api/proto.go
//! newDiagnosticResponse/diagnosticSourceLines. LSP's CR/LF index stays separate.

use serde::{Deserialize, Deserializer};
use vize_l0::{String, cstr};

// Omission is a supported older DTO; explicit null is not a native location.
pub(super) fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePosition {
    line: u32,
    character: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeSourceLine {
    line: u32,
    text: String,
}

pub(super) struct EcmaLines {
    // UTF16 offset and UTF8 byte index at the same original line start.
    starts: Vec<(u32, usize)>,
    len: u32,
}

impl EcmaLines {
    pub(super) fn new(text: &str) -> Result<Self, String> {
        let mut starts = vec![(0, 0)];
        let mut len = 0_u32;
        let mut characters = text.char_indices().peekable();
        while let Some((offset, character)) = characters.next() {
            len = len
                .checked_add(character.len_utf16() as u32)
                .ok_or_else(|| cstr!("Native ECMA source exceeds u32"))?;
            let mut end = offset + character.len_utf8();
            if character == '\r' && characters.peek().is_some_and(|(_, next)| *next == '\n') {
                let (offset, _) = characters
                    .next()
                    .ok_or_else(|| cstr!("Invalid native CRLF index"))?;
                end = offset + 1;
                len = len
                    .checked_add(1)
                    .ok_or_else(|| cstr!("Native ECMA source exceeds u32"))?;
            }
            if matches!(character, '\r' | '\n' | '\u{2028}' | '\u{2029}') {
                starts.push((len, end));
            }
        }
        Ok(Self { starts, len })
    }

    fn position(&self, offset: i64) -> Result<NativePosition, String> {
        let offset = u32::try_from(offset).map_err(|_| cstr!("Invalid native ECMA offset"))?;
        if offset > self.len {
            return Err(cstr!("Native ECMA span exceeds its snapshot source"));
        }
        let line = self
            .starts
            .partition_point(|(start, _)| *start <= offset)
            .checked_sub(1)
            .ok_or_else(|| cstr!("Invalid native ECMA line"))?;
        let (start, _) = self
            .starts
            .get(line)
            .ok_or_else(|| cstr!("Invalid native ECMA line"))?;
        Ok(NativePosition {
            line: u32::try_from(line).map_err(|_| cstr!("Invalid native ECMA line"))?,
            character: offset - *start,
        })
    }

    pub(super) fn validate(
        &self,
        text: &str,
        pos: i64,
        end: i64,
        start: Option<&NativePosition>,
        finish: Option<&NativePosition>,
        lines: Option<&[NativeSourceLine]>,
    ) -> Result<(), String> {
        let (Some(start), Some(finish), Some(lines)) = (start, finish, lines) else {
            return Err(cstr!("Incomplete native diagnostic location metadata"));
        };
        if end < pos || *start != self.position(pos)? || *finish != self.position(end)? {
            return Err(cstr!(
                "Native diagnostic positions disagree with its snapshot source"
            ));
        }
        let expected = if finish.line - start.line >= 4 {
            vec![start.line, start.line + 1, finish.line - 1, finish.line]
        } else {
            (start.line..=finish.line).collect::<Vec<_>>()
        };
        if lines.len() != expected.len() {
            return Err(cstr!(
                "Native diagnostic source lines disagree with its span"
            ));
        }
        for (line, expected) in lines.iter().zip(expected) {
            let index =
                usize::try_from(expected).map_err(|_| cstr!("Invalid native source line"))?;
            let (_, start) = self
                .starts
                .get(index)
                .ok_or_else(|| cstr!("Invalid native source line"))?;
            let end = self
                .starts
                .get(index + 1)
                .map_or(text.len(), |(_, byte)| *byte);
            if line.line != expected || Some(line.text.as_str()) != text.get(*start..end) {
                return Err(cstr!(
                    "Native diagnostic source line differs from its snapshot source"
                ));
            }
        }
        Ok(())
    }
}
