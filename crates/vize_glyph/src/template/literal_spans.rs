//! Raw-line ownership of a directive value from its parsed string and
//! template-literal spans.
//!
//! A line that starts inside a quasi or a continued string literal belongs to
//! the runtime value and is printed verbatim. When the value parses, the
//! parser's spans decide that exactly, so a backtick inside a string or comment
//! is never mistaken for a literal (#9583). Values that are not JavaScript on
//! their own (`v-for` aliases, partial or invalid code) keep the lexical
//! ownership of [`LiteralLineState`]. The spans are L1's
//! ([`literal_line_starts`]); Glyph only decodes attribute references first.

use vize_l1::embed::syntax::{LiteralLineOptions, literal_line_starts};
use vize_l1::markup::entity::{EntityContext, decode_one};

use super::literal_lines::{LiteralLineState, Representation};

/// Line ownership for one value, by parsed spans when available.
pub(crate) enum ValueLines {
    /// `raw[n]` is whether value line `n` starts inside a literal; `next` is
    /// the line the next `line_is_raw` call answers for.
    Parsed {
        raw: Vec<bool>,
        next: usize,
    },
    Lexed(LiteralLineState),
}

impl ValueLines {
    /// Ownership for `value`, positioned before its first line.
    pub(crate) fn new(value: &str, representation: Representation) -> Self {
        parsed_raw_lines(value, representation).map_or_else(
            || Self::Lexed(LiteralLineState::from_line("", representation)),
            |raw| Self::Parsed { raw, next: 0 },
        )
    }

    pub(crate) fn with_cr(value: &str, representation: Representation) -> Self {
        parsed_raw_lines_with_separator::<b'\r'>(value, representation).map_or_else(
            || Self::Lexed(LiteralLineState::from_line("", representation)),
            |raw| Self::Parsed { raw, next: 0 },
        )
    }

    /// Ownership for the lines after the first of a rendered `name="value"`.
    pub(crate) fn for_attribute(attr: &str) -> Self {
        let value = attr
            .split_once('=')
            .and_then(|(_, value)| {
                let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;
                let value = value.get(1..)?;
                Some(value.strip_suffix(quote).unwrap_or(value))
            })
            .unwrap_or_default();
        match parsed_raw_lines(value, Representation::Html) {
            Some(raw) => Self::Parsed { raw, next: 1 },
            None => Self::Lexed(LiteralLineState::from_attribute(
                attr.split('\n').next().unwrap_or_default(),
            )),
        }
    }

    pub(crate) fn line_is_raw(&self) -> bool {
        match self {
            Self::Parsed { raw, next } => raw.get(*next).copied().unwrap_or(false),
            Self::Lexed(state) => state.line_is_raw(),
        }
    }

    pub(crate) fn line_holds_code(&self, line: &str) -> bool {
        !self.line_is_raw() && !line.trim().is_empty()
    }

    pub(crate) fn advance_line(&mut self, line: &str) {
        match self {
            Self::Parsed { next, .. } => *next += 1,
            Self::Lexed(state) => state.advance_line(line),
        }
    }
}

/// Per-line "starts inside a string or quasi" flags for a value that parses
/// as one expression (or, outside interpolations, as handler statements).
pub(crate) fn parsed_raw_lines(value: &str, representation: Representation) -> Option<Vec<bool>> {
    parsed_raw_lines_with_separator::<b'\n'>(value, representation)
}

fn parsed_raw_lines_with_separator<const NEWLINE: u8>(
    value: &str,
    representation: Representation,
) -> Option<Vec<bool>> {
    if !value.contains(char::from(NEWLINE)) {
        return None;
    }
    multi_line_raw_lines::<NEWLINE>(value, representation)
}

/// The parse behind [`parsed_raw_lines`], out of line from its one-line check.
#[inline(never)]
fn multi_line_raw_lines<const NEWLINE: u8>(
    value: &str,
    representation: Representation,
) -> Option<Vec<bool>> {
    let decoded = match representation {
        Representation::Html => decode_attribute(value)?,
        Representation::JavaScript | Representation::JavaScriptInterpolation => value.into(),
    };
    literal_line_starts(
        &decoded,
        LiteralLineOptions {
            newline: NEWLINE,
            // Directive printers keep a whole literal, substitutions included,
            // as the expression formatter laid it out; interpolations own only
            // the quasis.
            whole_templates: !matches!(representation, Representation::JavaScriptInterpolation),
            statements: !matches!(representation, Representation::JavaScriptInterpolation),
        },
    )
}

/// Decode attribute character references; a reference that decodes to a line
/// terminator would change the line structure, so such values stay lexical.
fn decode_attribute(value: &str) -> Option<vize_l0::String> {
    let mut decoded = vize_l0::String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        decoded.push_str(rest.get(..amp)?);
        let tail = rest.get(amp..)?;
        match decode_one(tail.as_bytes(), EntityContext::Attribute) {
            Some((entity, width)) => {
                let mut newline = false;
                entity.for_each(|c| {
                    newline |= matches!(c, '\n' | '\r');
                    decoded.push(c);
                });
                if newline {
                    return None;
                }
                rest = tail.get(width..)?;
            }
            None => {
                decoded.push('&');
                rest = tail.get(1..)?;
            }
        }
    }
    decoded.push_str(rest);
    Some(decoded)
}
