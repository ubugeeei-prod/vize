//! Shared raw-line ownership for directive reanchoring and both printers.
//! Mirrors existing quasi/substitution tracking, adding legal continued quotes
//! and rendered quote entities. Retains the prior raw-byte quasi ownership as
//! a lower bound; the assignment-regexp shield is deliberately bounded.
use vize_l1::markup::entity::{DecodedEntity, EntityContext, decode_one};

/// Protect decoded bare-sequence reference DATA before the existing renderer.
/// Original HTML fallback bytes and ordinary `&&`/bitwise bytes stay exact.
pub(super) fn encode_reference_data(value: vize_l0::String) -> vize_l0::String {
    let mut encoded: Option<vize_l0::String> = None;
    let mut start = 0;
    for (index, byte) in value.bytes().enumerate() {
        if byte == b'&'
            && decode_one(
                value.get(index..).unwrap_or_default().as_bytes(),
                EntityContext::Attribute,
            )
            .is_some()
        {
            let output =
                encoded.get_or_insert_with(|| vize_l0::String::with_capacity(value.len() + 4));
            output.push_str(value.get(start..index).unwrap_or_default());
            output.push_str("&amp;");
            start = index + 1;
        }
    }
    let Some(mut output) = encoded else {
        return value;
    };
    output.push_str(value.get(start..).unwrap_or_default());
    output
}

#[derive(Clone, Copy, Default)]
pub(crate) enum Representation {
    #[default]
    JavaScript,
    Html,
}

#[derive(Default)]
pub(crate) struct LiteralLineState {
    representation: Representation,
    template: bool,
    inherited_template: bool,
    inherited_escaped: bool,
    substitutions: Vec<u32>,
    quote: Option<u8>,
    escaped: bool,
    block_comment: bool,
    line_comment: bool,
}

impl LiteralLineState {
    pub(crate) fn from_line(line: &str, representation: Representation) -> Self {
        let mut state = Self {
            representation,
            ..Self::default()
        };
        state.advance_line(line);
        state
    }

    pub(crate) fn from_attribute(line: &str) -> Self {
        let value = line.split_once('=').map_or("", |(_, value)| value);
        let mut state = Self::rendered();
        state.advance_line(value.get(1..).unwrap_or_default());
        state
    }

    pub(crate) fn rendered() -> Self {
        Self {
            representation: Representation::Html,
            ..Self::default()
        }
    }

    pub(crate) fn line_is_raw(&self) -> bool {
        self.quasi_is_raw() || self.quote.is_some()
    }

    pub(crate) fn quasi_is_raw(&self) -> bool {
        self.inherited_template || self.template
    }

    fn advance_inherited(&mut self, bytes: &[u8]) {
        // Exactly the old unescaped-backtick authority on actual source bytes,
        // independent of HTML decoding, quotes, comments and regexp shielding.
        for &byte in bytes {
            if byte == b'`' && !self.inherited_escaped {
                self.inherited_template = !self.inherited_template;
            }
            self.inherited_escaped = byte == b'\\' && !self.inherited_escaped;
        }
    }

    pub(crate) fn line_holds_code(&self, line: &str) -> bool {
        !self.line_is_raw() && !line.trim().is_empty()
    }

    pub(crate) fn advance_line(&mut self, line: &str) {
        let bytes = line.trim_end_matches('\r').as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            index += self.advance_at(bytes, index);
        }
        self.finish_line();
    }

    pub(crate) fn finish_line(&mut self) {
        // Only a legal escaped line terminator carries an ordinary JS string
        // into the next line. Its leading spaces are part of the string value.
        if !self.escaped {
            self.quote = None;
        }
        self.escaped = false;
        self.inherited_escaped = false;
        self.line_comment = false;
    }

    pub(crate) fn advance_at(&mut self, bytes: &[u8], index: usize) -> usize {
        let tail = bytes.get(index..).unwrap_or_default();
        let Some(&byte) = tail.first() else { return 1 };
        if byte == b'\r' && tail.len() == 1 {
            return 1;
        }
        let (token, width) = if matches!(self.representation, Representation::Html)
            && byte == b'&'
            && let Some((decoded, width)) = decode_one(tail, EntityContext::Attribute)
        {
            let token = match decoded {
                DecodedEntity::Named(text) if text.len() == 1 => text.as_bytes()[0],
                DecodedEntity::Numeric(value) if value.is_ascii() => value as u8,
                _ => 0,
            };
            (token, width)
        } else {
            (byte, 1)
        };
        self.advance_inherited(tail.get(..width).unwrap_or_default());
        if self.line_comment {
            return width;
        }
        if self.block_comment {
            if tail.starts_with(b"*/") {
                self.block_comment = false;
                return 2;
            }
            return width;
        }
        if self.escaped {
            self.escaped = false;
            return width;
        }
        if self.quote.is_some() {
            if token == b'\\' {
                self.escaped = true;
            } else if self.quote == Some(token) {
                self.quote = None;
            }
        } else if self.template {
            if token == b'\\' {
                self.escaped = true;
            } else if token == b'`' {
                self.template = false;
            } else if tail.starts_with(b"${") {
                self.template = false;
                self.substitutions.push(0);
                return 2;
            }
        } else if tail.starts_with(b"//") {
            self.line_comment = true;
            return 2;
        } else if tail.starts_with(b"/*") {
            self.block_comment = true;
            return 2;
        } else if byte == b'/'
            && let Some(end) = assignment_regexp_end(bytes, index)
        {
            self.advance_inherited(bytes.get(index + width..end).unwrap_or_default());
            return end - index;
        } else if matches!(token, b'\'' | b'"') {
            self.quote = Some(token);
        } else if token == b'`' {
            self.template = true;
        } else if token == b'{' {
            if let Some(depth) = self.substitutions.last_mut() {
                *depth += 1;
            }
        } else if token == b'}' {
            match self.substitutions.last_mut() {
                Some(0) => {
                    self.substitutions.pop();
                    self.template = true;
                }
                Some(depth) => *depth -= 1,
                None => {}
            }
        }
        width
    }
}

/// A complete same-line pattern after a simple assignment is unambiguously an
/// operand, never division. Existing successful parsing owns syntax validity;
/// this only skips its quoted/backtick data, with slash escapes and classes.
/// Other slash contexts retain both pre-existing and logical ownership rules.
fn assignment_regexp_end(bytes: &[u8], start: usize) -> Option<usize> {
    let prefix = bytes.get(..start)?;
    let equals = prefix
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())?;
    if prefix.get(equals) != Some(&b'=')
        || equals
            .checked_sub(1)
            .and_then(|at| prefix.get(at))
            .is_some_and(|byte| matches!(byte, b'=' | b'!' | b'<' | b'>'))
    {
        return None;
    }
    let mut escaped = false;
    let mut class = false;
    for (offset, &byte) in bytes.get(start + 1..)?.iter().enumerate() {
        if matches!(byte, b'\r' | b'\n') {
            return None;
        }
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'[' => class = true,
            b']' => class = false,
            b'/' if !class => return Some(start + offset + 2),
            _ => {}
        }
    }
    None
}
