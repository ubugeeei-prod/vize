//! A total port of `pug-lexer@5.0.1` over the [`Logical`] text.
//!
//! The dispatch order, the regular expressions and the indentation stack
//! are pug's, so every structural decision (where a tag ends, which line
//! starts a text block, where an interpolation closes) is the pinned
//! package's. Two things differ, both forced by the S1 contract:
//!
//! - **Positions, not strings.** Tokens carry logical byte ranges (mapped
//!   back to authored bytes by the builder) instead of values; a value
//!   pug assembles from pieces (`\#[` escapes) is emitted as the pieces.
//! - **Total.** Where pug throws, the lexer records a [`PugErrorCode`] and
//!   recovers deterministically — usually by turning the rest of the line
//!   into an `Unexpected` token — so arbitrary bytes always lex.
//!
//! Constructs the Vue lowering refuses (mixins, includes, conditionals,
//! iteration, filters, unbuffered code, …) are recognised with pug's own
//! patterns and lexed as one rest-of-line `Keyword` token; their indented
//! bodies lex as ordinary pug.

mod attrs;
mod keyword;
mod text;

use vize_s0::{Allocator, Vec};

use super::error::PugErrorCode;
use super::tree::PugRefusal;

/// Lexer token kinds. The last group is zero-width structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tk {
    Tag,
    Class,
    Id,
    AttrsOpen,
    AttrName,
    AttrOp,
    AttrValue,
    AttrComma,
    AttrsClose,
    AndAttributes,
    Dot,
    Slash,
    Colon,
    Pipe,
    Text,
    TextHtml,
    Escape,
    InterpOpen,
    InterpClose,
    CodeInterp,
    CodeFlag,
    Code,
    Comment,
    CommentText,
    Keyword(PugRefusal),
    Unexpected,
    InterpCloseMissing,
    Newline,
    Indent,
    Outdent,
    PipelessStart,
    PipelessEnd,
    Eos,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Tok {
    pub(crate) kind: Tk,
    pub(crate) start: u32,
    pub(crate) end: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IndentRe {
    Tabs,
    Spaces,
}

pub(crate) struct Lexer<'a, 'o> {
    allocator: &'a Allocator,
    src: &'a str,
    pos: usize,
    end: usize,
    out: &'o mut Vec<'a, Tok>,
    errors: &'o mut Vec<'a, (PugErrorCode, u32)>,
    /// Current indentation last (pug's `indentStack[0]`).
    indent_stack: Vec<'a, usize>,
    indent_re: Option<IndentRe>,
    interpolation_allowed: bool,
    interpolated: bool,
    ended: bool,
}

/// Lex the whole logical text.
pub(crate) fn lex<'a>(
    allocator: &'a Allocator,
    src: &'a str,
    out: &mut Vec<'a, Tok>,
    errors: &mut Vec<'a, (PugErrorCode, u32)>,
) {
    let mut lexer = Lexer::new(allocator, src, 0, src.len(), out, errors, false);
    lexer.run();
}

impl<'a, 'o> Lexer<'a, 'o> {
    fn new(
        allocator: &'a Allocator,
        src: &'a str,
        pos: usize,
        end: usize,
        out: &'o mut Vec<'a, Tok>,
        errors: &'o mut Vec<'a, (PugErrorCode, u32)>,
        interpolated: bool,
    ) -> Self {
        let mut indent_stack = Vec::new_in(&allocator);
        indent_stack.push(0);
        Self {
            allocator,
            src,
            pos,
            end,
            out,
            errors,
            indent_stack,
            indent_re: None,
            interpolation_allowed: true,
            interpolated,
            ended: false,
        }
    }

    fn run(&mut self) {
        while !self.ended {
            let before = (self.pos, self.out.len());
            self.advance();
            if !self.ended && before == (self.pos, self.out.len()) {
                // Defensive: every dispatch arm consumes; never spin.
                self.unexpected_line(PugErrorCode::UnexpectedText);
            }
        }
    }

    fn advance(&mut self) {
        let _ = self.blank()
            || self.eos()
            || self.end_interpolation()
            || self.keyword()
            || self.tag()
            || self.filter()
            || self.block_code()
            || self.code()
            || self.id()
            || self.dot()
            || self.class_name()
            || self.attrs()
            || self.attributes_block()
            || self.indent()
            || self.text()
            || self.text_html()
            || self.comment()
            || self.slash()
            || self.colon()
            || self.fail();
    }

    fn input(&self) -> &'a str {
        &self.src[self.pos..self.end]
    }

    fn line_end(&self) -> usize {
        self.input().find('\n').map_or(self.end, |at| self.pos + at)
    }

    fn push(&mut self, kind: Tk, start: usize, end: usize) {
        self.out.push(Tok {
            kind,
            start: start as u32,
            end: end as u32,
        });
    }

    fn error(&mut self, code: PugErrorCode, at: usize) {
        self.errors.push((code, at as u32));
    }

    /// Recovery: the rest of the line becomes one `Unexpected` token.
    fn unexpected_line(&mut self, code: PugErrorCode) {
        self.error(code, self.pos);
        let end = self
            .line_end()
            .max(self.pos + self.input().chars().next().map_or(0, char::len_utf8));
        self.push(Tk::Unexpected, self.pos, end);
        self.pos = end;
    }

    fn blank(&mut self) -> bool {
        let input = self.input().as_bytes();
        if input.first() != Some(&b'\n') {
            return false;
        }
        let run = input[1..]
            .iter()
            .take_while(|&&b| b == b' ' || b == b'\t')
            .count();
        if input.get(1 + run) != Some(&b'\n') {
            return false;
        }
        self.pos += 1 + run;
        true
    }

    fn eos(&mut self) -> bool {
        if self.pos < self.end {
            return false;
        }
        if self.interpolated {
            self.error(PugErrorCode::NoEndBracket, self.pos);
            self.push(Tk::InterpCloseMissing, self.pos, self.pos);
        } else {
            while self.indent_stack.last().is_some_and(|&top| top != 0) {
                self.indent_stack.pop();
                self.push(Tk::Outdent, self.pos, self.pos);
            }
            self.push(Tk::Eos, self.pos, self.pos);
        }
        self.ended = true;
        true
    }

    fn end_interpolation(&mut self) -> bool {
        if !self.interpolated || !self.input().starts_with(']') {
            return false;
        }
        self.push(Tk::InterpClose, self.pos, self.pos + 1);
        self.pos += 1;
        self.ended = true;
        true
    }

    fn fail(&mut self) -> bool {
        self.unexpected_line(PugErrorCode::UnexpectedText);
        true
    }

    /// `scanIndentation`: the captured indentation width at a `\n`, and the
    /// full match length (the tabs pattern also swallows trailing spaces).
    fn scan_indentation(&mut self) -> Option<(usize, usize)> {
        let input = self.input().as_bytes();
        if input.first() != Some(&b'\n') {
            return None;
        }
        let count = |byte: u8| input[1..].iter().take_while(|&&b| b == byte).count();
        let tabs = |count_tabs: usize| {
            let spaces = input[1 + count_tabs..]
                .iter()
                .take_while(|&&b| b == b' ')
                .count();
            (count_tabs, 1 + count_tabs + spaces)
        };
        match self.indent_re {
            Some(IndentRe::Tabs) => Some(tabs(count(b'\t'))),
            Some(IndentRe::Spaces) => {
                let spaces = count(b' ');
                Some((spaces, 1 + spaces))
            }
            None => {
                let tab_count = count(b'\t');
                if tab_count > 0 {
                    self.indent_re = Some(IndentRe::Tabs);
                    return Some(tabs(tab_count));
                }
                let spaces = count(b' ');
                if spaces > 0 {
                    self.indent_re = Some(IndentRe::Spaces);
                }
                Some((spaces, 1 + spaces))
            }
        }
    }

    fn indent(&mut self) -> bool {
        let Some((indents, _)) = self.scan_indentation() else {
            return false;
        };
        self.pos += indents + 1;
        let stray = self
            .input()
            .bytes()
            .take_while(|&b| b == b' ' || b == b'\t')
            .count();
        if stray > 0 {
            self.error(PugErrorCode::InvalidIndentation, self.pos);
            self.pos += stray;
        }
        if self.input().starts_with('\n') {
            self.interpolation_allowed = true;
            return true;
        }
        let top = self.top();
        if indents < top {
            while self.top() > indents {
                let below = self.indent_stack.len().checked_sub(2);
                if below.map_or(0, |at| self.indent_stack[at]) < indents {
                    // pug throws; recover by keeping the line a sibling at
                    // the innermost level it does not outdent past.
                    self.error(PugErrorCode::InconsistentIndentation, self.pos);
                    self.push(Tk::Newline, self.pos, self.pos);
                    break;
                }
                self.indent_stack.pop();
                self.push(Tk::Outdent, self.pos, self.pos);
            }
        } else if indents != 0 && indents != top {
            self.indent_stack.push(indents);
            self.push(Tk::Indent, self.pos, self.pos);
        } else {
            self.push(Tk::Newline, self.pos, self.pos);
        }
        self.interpolation_allowed = true;
        true
    }

    fn top(&self) -> usize {
        self.indent_stack.last().copied().unwrap_or(0)
    }
}
