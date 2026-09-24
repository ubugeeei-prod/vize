//! Cursor over Rust source for the rule-fixture scanner.
//!
//! Strings, raw strings, comments, and nested brackets are real tokens, so a
//! comma inside `make_rule(a, b)` does not end an argument.

use vize_s0::String;

use super::fixture_escape::{
    decode_ascii_escape, decode_unicode_escape, hash_prefix, is_ident_continue, is_ident_start,
    is_ws, normalize_newlines, skip_continuation_ws, string_prefix_len,
};

pub(super) struct Cursor<'a> {
    pub(super) source: &'a str,
    pub(super) i: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self { source, i: 0 }
    }

    pub(super) fn eof(&self) -> bool {
        self.i >= self.source.len()
    }

    pub(super) fn peek(&self) -> char {
        self.rest().chars().next().unwrap_or('\0')
    }

    pub(super) fn bump(&mut self) {
        if let Some(ch) = self.rest().chars().next() {
            self.i += ch.len_utf8();
        }
    }

    pub(super) fn line_at(&self, index: usize) -> usize {
        self.source
            .get(..index)
            .unwrap_or_default()
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    }

    pub(super) fn rest(&self) -> &'a str {
        self.source.get(self.i..).unwrap_or_default()
    }

    pub(super) fn skip_ws_and_comments(&mut self) {
        loop {
            let rest = self.rest();
            if rest.starts_with("//") {
                self.i += 2;
                while !self.eof() && self.peek() != '\n' {
                    self.bump();
                }
                continue;
            }
            if rest.starts_with("/*") {
                self.skip_block_comment();
                continue;
            }
            if !self.eof() && is_ws(self.peek()) {
                self.bump();
                continue;
            }
            break;
        }
    }

    fn skip_block_comment(&mut self) {
        self.i += 2;
        let mut depth = 1i32;
        while !self.eof() && depth > 0 {
            let rest = self.rest();
            if rest.starts_with("/*") {
                depth += 1;
                self.i += 2;
            } else if rest.starts_with("*/") {
                depth -= 1;
                self.i += 2;
            } else {
                self.bump();
            }
        }
    }

    /// Cooked or raw string, including `b` / `br` / `c` / `cr`. The body is
    /// not scanned for calls.
    pub(super) fn skip_string_token(&mut self) -> bool {
        let Some(prefix) = string_prefix_len(self.rest()) else {
            return false;
        };
        let hashes = hash_prefix(self.rest().get(prefix..).unwrap_or_default());
        self.i += prefix + hashes + 1;
        if hashes == 0 {
            self.skip_cooked_body();
        } else {
            self.skip_raw_body(hashes);
        }
        self.skip_suffix();
        true
    }

    fn skip_cooked_body(&mut self) {
        while !self.eof() {
            match self.peek() {
                '\\' => {
                    self.bump();
                    if !self.eof() {
                        self.bump();
                    }
                }
                '"' => {
                    self.bump();
                    return;
                }
                _ => self.bump(),
            }
        }
    }

    fn skip_raw_body(&mut self, hashes: usize) {
        while !self.eof() {
            if self.peek() == '"'
                && hash_prefix(self.source.get(self.i + 1..).unwrap_or_default()) >= hashes
            {
                self.i += 1 + hashes;
                return;
            }
            self.bump();
        }
    }

    fn skip_suffix(&mut self) {
        if !self.eof() && is_ident_continue(self.peek()) {
            self.bump_ident();
        }
    }

    pub(super) fn skip_char_or_lifetime(&mut self) -> bool {
        if self.peek() != '\'' {
            return false;
        }
        let next = self
            .source
            .get(self.i + 1..)
            .unwrap_or_default()
            .chars()
            .next();
        if next.is_some_and(is_ident_start) || self.rest().starts_with("'r#") {
            self.bump();
            if self.rest().starts_with("r#") {
                self.i += 2;
            }
            if !self.eof() && is_ident_start(self.peek()) {
                self.bump_ident();
            }
            return true;
        }
        self.bump();
        if self.peek() == '\\' {
            self.bump();
            match self.peek() {
                'x' => {
                    self.bump();
                    self.bump();
                    self.bump();
                }
                'u' => self.skip_unicode_escape_body(),
                _ => {
                    if !self.eof() {
                        self.bump();
                    }
                }
            }
        } else if !self.eof() {
            self.bump();
        }
        if self.peek() == '\'' {
            self.bump();
        }
        true
    }

    fn skip_unicode_escape_body(&mut self) {
        self.bump();
        if self.peek() != '{' {
            return;
        }
        self.bump();
        while !self.eof() && self.peek() != '}' {
            self.bump();
        }
        if self.peek() == '}' {
            self.bump();
        }
    }

    pub(super) fn ident_start(&self) -> Option<usize> {
        (!self.eof() && is_ident_start(self.peek())).then_some(self.i)
    }

    pub(super) fn bump_ident(&mut self) -> &'a str {
        let start = self.i;
        self.bump();
        while !self.eof() && is_ident_continue(self.peek()) {
            self.bump();
        }
        self.source.get(start..self.i).unwrap_or_default()
    }

    pub(super) fn call_paren(&self) -> Option<usize> {
        let mut peek = Cursor {
            source: self.source,
            i: self.i,
        };
        peek.skip_ws_and_comments();
        (peek.peek() == '(').then_some(peek.i)
    }

    /// One `&str` literal at the cursor (`"..."` or `r#"..."#`), not a byte
    /// or C string. A suffix makes it not a string expression.
    pub(super) fn decode_one_str(&mut self) -> Option<String> {
        let prefix = string_prefix_len(self.rest())?;
        if prefix > 0 && !self.rest().starts_with('r') {
            return None;
        }
        let hashes = hash_prefix(self.rest().get(prefix..).unwrap_or_default());
        self.i += prefix + hashes + 1;
        let value = if hashes == 0 {
            self.decode_cooked()?
        } else {
            self.decode_raw(hashes)?
        };
        if !self.eof() && is_ident_continue(self.peek()) {
            return None;
        }
        Some(value)
    }

    fn decode_raw(&mut self, hashes: usize) -> Option<String> {
        let start = self.i;
        while !self.eof() {
            if self.peek() == '"'
                && hash_prefix(self.source.get(self.i + 1..).unwrap_or_default()) >= hashes
            {
                let body = normalize_newlines(self.source.get(start..self.i).unwrap_or_default());
                self.i += 1 + hashes;
                return Some(body);
            }
            self.bump();
        }
        None
    }

    fn decode_cooked(&mut self) -> Option<String> {
        let mut out = String::default();
        while !self.eof() {
            match self.peek() {
                '"' => {
                    self.bump();
                    return Some(out);
                }
                '\\' => {
                    self.bump();
                    self.push_escape(&mut out)?;
                }
                '\r' => {
                    self.bump();
                    if self.peek() != '\n' {
                        return None;
                    }
                    out.push('\n');
                    self.bump();
                }
                ch => {
                    out.push(ch);
                    self.bump();
                }
            }
        }
        None
    }

    fn push_escape(&mut self, out: &mut String) -> Option<()> {
        let ch = self.peek();
        self.bump();
        match ch {
            '0' => out.push('\0'),
            't' => out.push('\t'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            '\\' => out.push('\\'),
            '\'' => out.push('\''),
            '"' => out.push('"'),
            'x' => out.push(decode_ascii_escape(self)?),
            'u' => out.push(decode_unicode_escape(self)?),
            '\n' => skip_continuation_ws(self),
            '\r' if self.peek() == '\n' => {
                self.bump();
                skip_continuation_ws(self);
            }
            _ => return None,
        }
        Some(())
    }
}
