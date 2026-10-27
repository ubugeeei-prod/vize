//! Line-head lexing: pug's keyword constructs (lexed whole), mixin calls,
//! tags, the `.class` / `#id` shorthands, code, and the one-byte markers.

use super::{Lexer, Tk};
use crate::pug::chars::parse_until;
use crate::pug::error::PugErrorCode;
use crate::pug::tree::PugRefusal;

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Does `line` start with `word` followed by a regexp `\b`?
fn starts_word(line: &[u8], word: &str) -> bool {
    line.starts_with(word.as_bytes()) && !line.get(word.len()).copied().is_some_and(is_word)
}

/// `scanEndOfLine` after a match of `len` bytes: `:` or `[ \t]*` to EOL.
fn ends_line(line: &[u8], len: usize) -> bool {
    let rest = line.get(len.min(line.len())..).unwrap_or_default();
    rest.first() == Some(&b':') || rest.iter().all(|&b| b == b' ' || b == b'\t')
}

/// `^word +([^\n]+)`: one space at least, then at least one byte (the
/// space run backtracks to leave it).
fn word_spaces_rest(line: &[u8], word: &str) -> bool {
    line.starts_with(word.as_bytes())
        && line.get(word.len()) == Some(&b' ')
        && line.len() >= word.len() + 2
}

fn spaces_after(line: &[u8], from: usize) -> usize {
    line.get(from.min(line.len())..)
        .unwrap_or_default()
        .iter()
        .take_while(|&&b| b == b' ')
        .count()
}

/// pug's `append` / `prepend` / `block NAME` lexers: a non-blank name
/// before any `//` comment.
fn named_block(line: &[u8]) -> bool {
    let has_name = |rest: &[u8]| {
        let name = rest
            .windows(2)
            .position(|pair| pair == b"//")
            .map_or(rest, |at| rest.get(..at).unwrap_or_default());
        name.iter().any(|&b| b != b' ' && b != b'\t')
    };
    let mut starts = [None, Some(0)];
    if line.starts_with(b"block") && spaces_after(line, 5) > 0 {
        starts[0] = Some(5 + spaces_after(line, 5));
    }
    let mode = starts.iter().flatten().any(|&at| {
        ["append", "prepend"].iter().any(|word| {
            word_spaces_rest(line.get(at..).unwrap_or_default(), word)
                && has_name(line.get(at + word.len()..).unwrap_or_default())
        })
    });
    mode || (word_spaces_rest(line, "block") && has_name(line.get(5..).unwrap_or_default()))
}

impl Lexer<'_, '_> {
    /// Every pug keyword construct, in pug's dispatch order, as one
    /// rest-of-line `Keyword` token (plus pug's error where it throws).
    pub(super) fn keyword(&mut self) -> bool {
        let input = self.input().as_bytes();
        let line = input.get(..self.line_end() - self.pos).unwrap_or_default();
        let malformed = |ok: bool| (!ok).then_some(PugErrorCode::MalformedKeyword);
        let (refusal, error) = if line.starts_with(b"yield") && ends_line(line, 5) {
            (PugRefusal::Yield, None)
        } else if line.starts_with(b"doctype") {
            (PugRefusal::Doctype, None)
        } else if line.starts_with(b"#{") {
            let unbalanced = parse_until(self.allocator, self.input(), b'}', 2).is_err();
            (
                PugRefusal::Interpolation,
                unbalanced.then_some(PugErrorCode::NoEndBracket),
            )
        } else if starts_word(line, "case") {
            (PugRefusal::Case, malformed(word_spaces_rest(line, "case")))
        } else if starts_word(line, "when") {
            let spaces = spaces_after(line, 4);
            let next = line.get(4 + spaces);
            let ok = spaces >= 2 || (spaces == 1 && next.is_some_and(|&b| b != b':'));
            (PugRefusal::When, malformed(ok))
        } else if starts_word(line, "default") {
            (PugRefusal::Default, malformed(ends_line(line, 7)))
        } else if starts_word(line, "extend") || starts_word(line, "extends") {
            (PugRefusal::Extends, None)
        } else if named_block(line) {
            (PugRefusal::Block, None)
        } else if line.starts_with(b"block") && ends_line(line, 5) {
            (PugRefusal::MixinBlock, None)
        } else if starts_word(line, "include") {
            (PugRefusal::Include, None)
        } else if line.starts_with(b"mixin")
            && spaces_after(line, 5) > 0
            && line
                .get(5 + spaces_after(line, 5))
                .is_some_and(|&b| is_word(b) || b == b'-')
        {
            (PugRefusal::Mixin, None)
        } else if line.first() == Some(&b'+') && is_call(input) {
            self.call();
            return true;
        } else if ["if", "unless", "else"]
            .iter()
            .any(|kw| starts_word(line, kw))
        {
            (PugRefusal::Conditional, None)
        } else if starts_word(line, "each") || starts_word(line, "for") {
            (PugRefusal::Each, None)
        } else if starts_word(line, "while") {
            (
                PugRefusal::While,
                malformed(word_spaces_rest(line, "while")),
            )
        } else {
            return false;
        };
        if let Some(error) = error {
            self.error(error, self.pos);
        }
        let end = self.line_end();
        self.push(Tk::Keyword(refusal), self.pos, end);
        self.pos = end;
        true
    }

    /// pug's `call`: `^\+(\s*)(([-\w]+)|(#\{))`, then `^ *\(` arguments
    /// unless they read as attributes (`^\s*[-\w]+ *=`). The rest of the
    /// line lexes as a tag's tail; an unclosed `(` is left to `attrs`,
    /// which reports it.
    fn call(&mut self) {
        let input = self.input().as_bytes();
        let blanks = input
            .get(1..)
            .unwrap_or_default()
            .iter()
            .take_while(|&&b| is_js_blank(b))
            .count();
        let name_at = self.pos + 1 + blanks;
        let rest = input.get(1 + blanks..).unwrap_or_default();
        let end = if rest.starts_with(b"#{") {
            let src = crate::slice::range(self.src, name_at, self.end);
            match parse_until(self.allocator, src, b'}', 2) {
                Ok(close) => name_at + close + 1,
                Err(_) => {
                    self.error(PugErrorCode::NoEndBracket, name_at);
                    self.line_end().max(name_at)
                }
            }
        } else {
            name_at
                + rest
                    .iter()
                    .take_while(|&&b| is_word(b) || b == b'-')
                    .count()
        };
        self.push(Tk::Call, self.pos, end);
        self.pos = end;
        let spaces = self.input().bytes().take_while(|&b| b == b' ').count();
        if !self
            .input()
            .as_bytes()
            .get(spaces)
            .is_some_and(|&b| b == b'(')
        {
            return;
        }
        let open = self.pos + spaces;
        let src = crate::slice::range(self.src, open, self.end);
        if let Ok(close) = parse_until(self.allocator, src, b')', 1)
            && !names_an_attribute(src.get(1..close).unwrap_or_default().as_bytes())
        {
            self.push(Tk::CallArgs, open, open + close + 1);
            self.pos = open + close + 1;
        }
    }

    /// `^(\w(?:[-:\w]*\w)?)`.
    pub(super) fn tag(&mut self) -> bool {
        let input = self.input().as_bytes();
        if !input.first().copied().is_some_and(is_word) {
            return false;
        }
        let run = input
            .iter()
            .take_while(|&&b| is_word(b) || b == b'-' || b == b':')
            .count();
        let len = input
            .get(..run)
            .unwrap_or_default()
            .iter()
            .rposition(|&b| is_word(b))
            .map_or(1, |at| at + 1);
        self.push(Tk::Tag, self.pos, self.pos + len);
        self.pos += len;
        true
    }

    /// `^:([\w\-]+)` — a filter, refused with its pipeless body.
    pub(super) fn filter(&mut self) -> bool {
        let input = self.input().as_bytes();
        let named = input.get(1).is_some_and(|&b| is_word(b) || b == b'-');
        if input.first() != Some(&b':') || !named {
            return false;
        }
        self.refuse_with_body(PugRefusal::Filter);
        true
    }

    /// `scanEndOfLine(/^-/)` — unbuffered block code, refused with its body.
    pub(super) fn block_code(&mut self) -> bool {
        let line = self
            .input()
            .as_bytes()
            .get(..self.line_end() - self.pos)
            .unwrap_or_default();
        if line.first() != Some(&b'-') || !ends_line(line, 1) {
            return false;
        }
        self.refuse_with_body(PugRefusal::BlockCode);
        true
    }

    fn refuse_with_body(&mut self, refusal: PugRefusal) {
        let end = self.line_end();
        self.push(Tk::Keyword(refusal), self.pos, end);
        self.pos = end;
        self.interpolation_allowed = false;
        self.pipeless_text(None);
    }

    /// `^(!?=|-)[ \t]*([^\n]+)`, cut at `]` inside a tag interpolation.
    pub(super) fn code(&mut self) -> bool {
        let input = self.input().as_bytes();
        let flag = if input.starts_with(b"!=") {
            2
        } else if matches!(input.first(), Some(b'=' | b'-')) {
            1
        } else {
            return false;
        };
        let line_len = self.line_end() - self.pos;
        if line_len <= flag {
            return false;
        }
        let blanks = input
            .get(flag..line_len)
            .unwrap_or_default()
            .iter()
            .take_while(|&&b| b == b' ' || b == b'\t')
            .count();
        // `[ \t]*` backtracks one byte when the rest is only blanks.
        let code_start = self.pos + flag + blanks.min(line_len - flag - 1);
        let mut code_end = self.pos + line_len;
        if self.interpolated {
            match parse_until(
                self.allocator,
                crate::slice::range(self.src, code_start, code_end),
                b']',
                0,
            ) {
                Ok(close) => code_end = code_start + close,
                Err(_) => self.error(PugErrorCode::NoEndBracket, code_start),
            }
        }
        self.push(Tk::CodeFlag, self.pos, self.pos + flag);
        self.push(Tk::Code, code_start, code_end);
        self.pos = code_end;
        true
    }

    /// `^#([\w-]+)`; a bare `#` is `INVALID_ID`.
    pub(super) fn id(&mut self) -> bool {
        let input = self.input().as_bytes();
        if input.first() != Some(&b'#') {
            return false;
        }
        let len = input
            .get(1..)
            .unwrap_or_default()
            .iter()
            .take_while(|&&b| is_word(b) || b == b'-')
            .count();
        if len == 0 {
            self.unexpected_line(PugErrorCode::InvalidId);
        } else {
            self.push(Tk::Id, self.pos, self.pos + 1 + len);
            self.pos += 1 + len;
        }
        true
    }

    /// `scanEndOfLine(/^\./)`, then a pipeless block.
    pub(super) fn dot(&mut self) -> bool {
        let line = self
            .input()
            .as_bytes()
            .get(..self.line_end() - self.pos)
            .unwrap_or_default();
        if line.first() != Some(&b'.') || !ends_line(line, 1) {
            return false;
        }
        self.push(Tk::Dot, self.pos, self.pos + 1);
        self.pos += 1;
        if !self.input().starts_with(':') {
            self.pos += self
                .input()
                .bytes()
                .take_while(|&b| b == b' ' || b == b'\t')
                .count();
            self.pipeless_text(None);
        }
        true
    }

    /// `^\.([_a-z0-9\-]*[_a-z][_a-z0-9\-]*)`/i; other dots are errors.
    pub(super) fn class_name(&mut self) -> bool {
        let input = self.input().as_bytes();
        if input.first() != Some(&b'.') {
            return false;
        }
        let run = input
            .get(1..)
            .unwrap_or_default()
            .iter()
            .take_while(|&&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            .count();
        let named = input
            .get(1..1 + run)
            .unwrap_or_default()
            .iter()
            .any(|&b| b.is_ascii_alphabetic() || b == b'_');
        if named {
            self.push(Tk::Class, self.pos, self.pos + 1 + run);
            self.pos += 1 + run;
        } else {
            self.unexpected_line(PugErrorCode::InvalidClassName);
        }
        true
    }

    /// `^&attributes\b` + a bracket expression.
    pub(super) fn attributes_block(&mut self) -> bool {
        if !starts_word(self.input().as_bytes(), "&attributes") {
            return false;
        }
        let after = self.pos + 11;
        let rest = crate::slice::range(self.src, after, self.end);
        let end = match rest
            .starts_with('(')
            .then(|| parse_until(self.allocator, rest, b')', 1))
        {
            Some(Ok(close)) => after + close + 1,
            _ => {
                self.error(PugErrorCode::NoEndBracket, after);
                after
            }
        };
        self.push(Tk::AndAttributes, self.pos, end);
        self.pos = end;
        true
    }

    pub(super) fn slash(&mut self) -> bool {
        if !self.input().starts_with('/') {
            return false;
        }
        self.push(Tk::Slash, self.pos, self.pos + 1);
        self.pos += 1;
        true
    }

    /// `^: +`.
    pub(super) fn colon(&mut self) -> bool {
        let input = self.input().as_bytes();
        if input.first() != Some(&b':') || spaces_after(input, 1) == 0 {
            return false;
        }
        self.push(Tk::Colon, self.pos, self.pos + 1);
        self.pos += 1 + spaces_after(input, 1);
        true
    }
}

/// The ASCII bytes of a regexp `\s`.
fn is_js_blank(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// `^\s*[-\w]+ *=`: call parentheses that hold attributes.
fn names_an_attribute(src: &[u8]) -> bool {
    let blanks = src.iter().take_while(|&&b| is_js_blank(b)).count();
    let rest = src.get(blanks..).unwrap_or_default();
    let name = rest
        .iter()
        .take_while(|&&b| is_word(b) || b == b'-')
        .count();
    let spaces = spaces_after(rest, name);
    name > 0 && rest.get(name + spaces) == Some(&b'=')
}

/// `^\+(\s*)(([-\w]+)|(#\{))`.
fn is_call(input: &[u8]) -> bool {
    let blanks = input
        .get(1..)
        .unwrap_or_default()
        .iter()
        .take_while(|&&b| is_js_blank(b))
        .count();
    let rest = input.get(1 + blanks..).unwrap_or_default();
    rest.first().is_some_and(|&b| is_word(b) || b == b'-') || rest.starts_with(b"#{")
}
