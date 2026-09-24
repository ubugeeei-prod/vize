//! `( … )` attribute blocks: pug's `attrs` / `attribute` /
//! `attributeValue`, positions instead of strings.
//!
//! The value scanner stops where pug stops: at whitespace followed by a
//! non-punctuator (or a quote, `:` or `...`) once the value so far is a
//! complete expression, or at a top-level `,`. pug decides "complete
//! expression" with a full JavaScript parse (`is-expression`); here it is
//! the structural approximation [`looks_complete`], which agrees with the
//! parse on every value the Vue lowering accepts (literals) — values it
//! disagrees on are executable pug, refused downstream either way.

use super::{Lexer, Tk};
use crate::pug::chars::{State, is_punctuator, parse_until};
use crate::pug::error::PugErrorCode;

fn is_ws(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\t')
}

impl Lexer<'_, '_> {
    pub(super) fn attrs(&mut self) -> bool {
        if !self.input().starts_with('(') {
            return false;
        }
        let close = match parse_until(self.allocator, self.input(), b')', 1) {
            Ok(close) => close,
            Err(error) => {
                let code = match error {
                    crate::pug::chars::ScanError::EndOfString => PugErrorCode::NoEndBracket,
                    crate::pug::chars::ScanError::Mismatched(_) => PugErrorCode::BracketMismatch,
                };
                self.unexpected_line(code);
                return true;
            }
        };
        let start = self.pos;
        self.push(Tk::AttrsOpen, start, start + 1);
        let end = start + close;
        let mut at = start + 1;
        while at < end {
            let next = self.attribute(at, end);
            debug_assert!(next > at, "attribute scanning always consumes");
            at = next.max(at + 1);
        }
        self.push(Tk::AttrsClose, end, end + 1);
        self.pos = end + 1;
        true
    }

    /// One attribute from `[at, end)`; returns where the next one starts.
    fn attribute(&mut self, mut at: usize, end: usize) -> usize {
        let bytes = self.src.as_bytes();
        while at < end && is_ws(byte_at(bytes, at)) {
            at += 1;
        }
        if at == end {
            return end;
        }
        let key_start = at;
        let quote = matches!(byte_at(bytes, at), b'\'' | b'"').then_some(byte_at(bytes, at));
        if quote.is_some() {
            at += 1;
        }
        while at < end {
            let byte = byte_at(bytes, at);
            if let Some(quote) = quote {
                if byte == quote {
                    at += 1;
                    break;
                }
            } else if is_ws(byte) || matches!(byte, b'!' | b'=' | b',') {
                break;
            }
            at += crate::slice::from(self.src, at)
                .chars()
                .next()
                .map_or(1, char::len_utf8);
        }
        self.push(Tk::AttrName, key_start, at);
        let Some(mut rest) = self.attribute_value(at, end) else {
            return end;
        };
        while rest < end && is_ws(byte_at(bytes, rest)) {
            rest += 1;
        }
        if rest < end && byte_at(bytes, rest) == b',' {
            self.push(Tk::AttrComma, rest, rest + 1);
            rest += 1;
        }
        rest
    }

    /// `attributeValue`: `None` after a recovered error (the rest of the
    /// block became `Unexpected`), otherwise where the attribute ends.
    fn attribute_value(&mut self, from: usize, end: usize) -> Option<usize> {
        let bytes = self.src.as_bytes();
        let mut at = from;
        while at < end && is_ws(byte_at(bytes, at)) {
            at += 1;
        }
        if at == end {
            return Some(from);
        }
        let op_start = at;
        if byte_at(bytes, at) == b'!' {
            at += 1;
            if at >= end || byte_at(bytes, at) != b'=' {
                return self.attribute_error(op_start, end);
            }
        }
        if byte_at(bytes, at) != b'=' {
            if at == from && !is_ws(byte_at(bytes, from)) && byte_at(bytes, from) != b',' {
                return self.attribute_error(from, end);
            }
            return Some(from);
        }
        at += 1;
        self.push(Tk::AttrOp, op_start, at);
        while at < end && is_ws(byte_at(bytes, at)) {
            at += 1;
        }
        let value_start = at;
        let mut state = State::new(self.allocator);
        while at < end {
            let byte = byte_at(bytes, at);
            if !(state.is_nesting() || state.is_string()) {
                let value = crate::slice::range(self.src, value_start, at);
                if is_ws(byte) {
                    let next = (at..end).find(|&x| !is_ws(byte_at(bytes, x)));
                    let Some(next) = next else {
                        break;
                    };
                    let ch = crate::slice::from(self.src, next).chars().next();
                    let ends = !is_punctuator(ch)
                        || matches!(ch, Some('\'' | '"' | ':'))
                        || crate::slice::range(self.src, next, end).starts_with("...");
                    if ends && looks_complete(value) {
                        break;
                    }
                }
                if byte == b',' && looks_complete(value) {
                    break;
                }
            }
            let ch = crate::slice::from(self.src, at)
                .chars()
                .next()
                .unwrap_or('\0');
            if state.push(ch).is_err() {
                return self.attribute_error(at, end);
            }
            at += ch.len_utf8();
        }
        self.push(Tk::AttrValue, value_start, at);
        Some(at)
    }

    fn attribute_error(&mut self, at: usize, end: usize) -> Option<usize> {
        self.error(PugErrorCode::InvalidAttribute, at);
        self.push(Tk::Unexpected, at, end);
        None
    }
}

/// The structural stand-in for `is-expression`: non-blank, balanced, not
/// inside a string or comment, and not ending in an operator.
///
/// Called only where the running scanner state is neither nesting nor
/// inside a string (so the value is balanced and outside any comment);
/// what remains is the operator test on its last significant character.
pub(crate) fn looks_complete(value: &str) -> bool {
    let trimmed = value.trim_end_matches(crate::pug::logical::is_js_whitespace);
    trimmed
        .chars()
        .last()
        .is_some_and(|last| matches!(last, ')' | ']' | '}') || !is_punctuator(Some(last)))
}

/// The byte at `at`, or NUL past the end (no attribute rule matches NUL, so
/// scans stop there).
fn byte_at(bytes: &[u8], at: usize) -> u8 {
    bytes.get(at).copied().unwrap_or(0)
}
