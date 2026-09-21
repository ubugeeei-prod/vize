//! A port of `character-parser@2.2.0` — the JavaScript bracket/string
//! tracker pug's lexer uses to find the end of `( … )` attribute blocks,
//! `#{ … }` interpolations and attribute values. Same states, same
//! regexp-vs-divide heuristic, same failure points, so the surface tree
//! splits exactly where the pinned `pug` does.

use alloc::vec::Vec;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Frame {
    LineComment,
    BlockComment,
    Single,
    Double,
    Template,
    Regexp,
    Close(u8),
}

/// Why a scan failed (`CHARACTER_PARSER:*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScanError {
    /// `END_OF_STRING_REACHED`.
    EndOfString,
    /// `MISMATCHED_BRACKET` at this byte offset.
    Mismatched(usize),
}

pub(crate) struct State {
    stack: Vec<Frame>,
    regexp_start: bool,
    escaped: bool,
    has_dollar: bool,
    /// Significant characters, most recent last (comments excluded).
    history: Vec<char>,
    last_char: Option<char>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            stack: Vec::new(),
            regexp_start: false,
            escaped: false,
            has_dollar: false,
            history: Vec::new(),
            last_char: None,
        }
    }

    pub(crate) fn is_nesting(&self) -> bool {
        !self.stack.is_empty()
    }

    pub(crate) fn is_string(&self) -> bool {
        matches!(
            self.stack.last(),
            Some(Frame::Single | Frame::Double | Frame::Template)
        )
    }

    fn is_comment(&self) -> bool {
        matches!(
            self.stack.last(),
            Some(Frame::LineComment | Frame::BlockComment)
        )
    }

    /// `parseChar`; `Err(())` is a mismatched closing bracket.
    pub(crate) fn push(&mut self, ch: char) -> Result<(), ()> {
        let was_comment = self.is_comment();
        let last = self.history.last().copied();
        if self.regexp_start {
            if ch == '/' || ch == '*' {
                self.stack.pop();
            }
            self.regexp_start = false;
        }
        match self.stack.last().copied() {
            Some(Frame::LineComment) => {
                if ch == '\n' {
                    self.stack.pop();
                }
            }
            Some(Frame::BlockComment) => {
                if self.last_char == Some('*') && ch == '/' {
                    self.stack.pop();
                }
            }
            Some(Frame::Single) => self.quoted(ch, '\''),
            Some(Frame::Double) => self.quoted(ch, '"'),
            Some(Frame::Regexp) => self.quoted(ch, '/'),
            Some(Frame::Template) => {
                if ch == '`' && !self.escaped {
                    self.stack.pop();
                    self.has_dollar = false;
                } else if ch == '\\' && !self.escaped {
                    self.escaped = true;
                    self.has_dollar = false;
                } else if ch == '$' && !self.escaped {
                    self.has_dollar = true;
                } else if ch == '{' && self.has_dollar {
                    self.stack.push(Frame::Close(b'}'));
                } else {
                    self.escaped = false;
                    self.has_dollar = false;
                }
            }
            _ => self.code(ch, last)?,
        }
        if !self.is_comment() && !was_comment {
            self.history.push(ch);
        }
        self.last_char = Some(ch);
        Ok(())
    }

    fn quoted(&mut self, ch: char, quote: char) {
        if ch == quote && !self.escaped {
            self.stack.pop();
        } else {
            self.escaped = ch == '\\' && !self.escaped;
        }
    }

    fn code(&mut self, ch: char, last: Option<char>) -> Result<(), ()> {
        match ch {
            '(' => self.stack.push(Frame::Close(b')')),
            '{' => self.stack.push(Frame::Close(b'}')),
            '[' => self.stack.push(Frame::Close(b']')),
            ')' | '}' | ']' => {
                if self.stack.last() != Some(&Frame::Close(ch as u8)) {
                    return Err(());
                }
                self.stack.pop();
            }
            '/' if last == Some('/') => {
                self.history.pop();
                self.stack.push(Frame::LineComment);
            }
            '*' if last == Some('/') => {
                self.history.pop();
                self.stack.push(Frame::BlockComment);
            }
            '/' if is_regexp(&self.history) => {
                self.stack.push(Frame::Regexp);
                self.regexp_start = true;
            }
            '\'' => self.stack.push(Frame::Single),
            '"' => self.stack.push(Frame::Double),
            '`' => self.stack.push(Frame::Template),
            _ => {}
        }
        Ok(())
    }
}

/// `parseUntil(src, delimiter, { start })`: the offset (relative to `src`)
/// of the first `delimiter` byte met while not nesting.
pub(crate) fn parse_until(src: &str, delimiter: u8, start: usize) -> Result<usize, ScanError> {
    let mut state = State::new();
    for (index, ch) in src[start..].char_indices() {
        let at = start + index;
        if !state.is_nesting() && src.as_bytes()[at] == delimiter {
            return Ok(at);
        }
        state.push(ch).map_err(|()| ScanError::Mismatched(at))?;
    }
    Err(ScanError::EndOfString)
}

/// `isPunctuator` — `None` (the start of a string) counts as one.
pub(crate) fn is_punctuator(ch: Option<char>) -> bool {
    match ch {
        None => true,
        Some(ch) => matches!(
            ch,
            '.' | '('
                | ')'
                | ';'
                | ','
                | '{'
                | '}'
                | '['
                | ']'
                | ':'
                | '?'
                | '~'
                | '%'
                | '&'
                | '*'
                | '+'
                | '-'
                | '/'
                | '<'
                | '>'
                | '^'
                | '|'
                | '!'
                | '='
        ),
    }
}

const KEYWORDS: &[&str] = &[
    "if",
    "in",
    "do",
    "var",
    "for",
    "new",
    "try",
    "let",
    "this",
    "else",
    "case",
    "void",
    "with",
    "enum",
    "while",
    "break",
    "catch",
    "throw",
    "const",
    "yield",
    "class",
    "super",
    "return",
    "typeof",
    "delete",
    "switch",
    "export",
    "import",
    "default",
    "finally",
    "extends",
    "function",
    "continue",
    "debugger",
    "package",
    "private",
    "interface",
    "instanceof",
    "implements",
    "protected",
    "public",
    "static",
];

/// `isRegexp(history)`: could a `/` here start a regexp literal?
fn is_regexp(history: &[char]) -> bool {
    let recent = history
        .iter()
        .rev()
        .copied()
        .skip_while(|ch| super::logical::is_js_whitespace(*ch));
    let Some(first) = recent.clone().next() else {
        return true;
    };
    if first == ')' {
        return false;
    }
    if first == '}' || is_punctuator(Some(first)) {
        return true;
    }
    // `/^\w+\b/` over the reversed history: the latest word, reversed back.
    let mut word: Vec<u8> = recent
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(|ch| ch as u8)
        .collect();
    word.reverse();
    !word.is_empty()
        && KEYWORDS
            .iter()
            .any(|keyword| keyword.as_bytes() == word.as_slice())
}
