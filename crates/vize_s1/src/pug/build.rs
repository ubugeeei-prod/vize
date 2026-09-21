//! Token stream → pug surface tree: a port of `pug-parser@6.0.0`'s
//! recursive descent over the lexer's tokens, with the byte-coverage
//! discipline of the Vue surface builder — every token becomes
//! `leading = [cursor, start)` + `text = [start, end)` and advances the
//! cursor, so the in-order walk partitions the authored bytes.
//!
//! **Trivia.** pug strips unbuffered `//-` comments (and their bodies)
//! from the token stream before parsing (`pug-strip-comments`), so they
//! never influence structure — two piped lines around one still join with
//! a newline. The builder does the same by skipping those token groups:
//! their bytes fall into the next token's `leading`, the SwiftSyntax
//! trivia model, and render exactly as authored.

mod tag;
mod text;

use vize_s0::{Allocator, Box, Vec};

use super::error::{PugError, PugErrorCode};
use super::lex::{Tk, Tok};
use super::logical::Logical;
use super::tree::{PugCode, PugComment, PugNode, PugRefused, PugTree, PugUnexpected};
use crate::surface::Token;

pub(crate) struct Builder<'a, 't> {
    allocator: &'a Allocator,
    source: &'a str,
    logical: &'t Logical<'a>,
    tokens: &'t [Tok],
    at: usize,
    cursor: usize,
    errors: &'t mut Vec<'a, PugError>,
}

pub(crate) fn build<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    logical: &Logical<'a>,
    tokens: &[Tok],
    errors: &mut Vec<'a, PugError>,
) -> PugTree<'a> {
    let mut builder = Builder {
        allocator,
        source,
        logical,
        tokens,
        at: 0,
        cursor: 0,
        errors,
    };
    let mut nodes = Vec::new_in(&allocator);
    loop {
        match builder.peek() {
            Tk::Eos => break,
            Tk::Newline | Tk::Outdent => builder.skip(),
            // pug's top level skips the nothing a bare `.` parses to.
            Tk::Dot => nodes.push(builder.parse_dot(true)),
            _ => builder.push_expr(&mut nodes),
        }
    }
    let eof = Token::present(&source[builder.cursor..], &source[source.len()..]);
    PugTree { source, nodes, eof }
}

impl<'a> Builder<'a, '_> {
    /// The current token kind, stepping over stripped `//-` comment groups.
    fn peek(&mut self) -> Tk {
        loop {
            let Some(token) = self.tokens.get(self.at) else {
                return Tk::Eos;
            };
            let unbuffered = token.kind == Tk::Comment && token.end - token.start == 3;
            if !unbuffered {
                return token.kind;
            }
            self.at += 1;
            if self
                .tokens
                .get(self.at)
                .is_some_and(|t| t.kind == Tk::CommentText)
            {
                self.at += 1;
            }
            if self
                .tokens
                .get(self.at)
                .is_some_and(|t| t.kind == Tk::PipelessStart)
            {
                while self
                    .tokens
                    .get(self.at)
                    .is_some_and(|t| t.kind != Tk::PipelessEnd)
                {
                    self.at += 1;
                }
                self.at += 1;
            }
        }
    }

    fn peek_is(&mut self, kind: Tk) -> bool {
        self.peek() == kind
    }

    /// Consume a zero-width structural token.
    fn skip(&mut self) {
        self.peek();
        self.at += 1;
    }

    /// Consume the current token as a surface token.
    fn take(&mut self) -> Token<'a> {
        self.peek();
        let tok = self.tokens[self.at];
        self.at += 1;
        let (start, end) = self.logical.range(tok.start as usize, tok.end as usize);
        let start = (start as usize).max(self.cursor);
        let end = (end as usize).max(start);
        debug_assert!(start >= self.cursor, "pug tokens are in source order");
        let token = Token::present(&self.source[self.cursor..start], &self.source[start..end]);
        self.cursor = end;
        token
    }

    /// A zero-width `Missing` hole at the current token's position.
    fn take_missing(&mut self) -> Token<'a> {
        self.peek();
        let tok = self.tokens[self.at];
        self.at += 1;
        let at = (self.logical.at(tok.start as usize) as usize).max(self.cursor);
        let token = Token::missing(&self.source[self.cursor..at], &self.source[at..at]);
        self.cursor = at;
        token
    }

    /// The authored offset of the current token (for zero-width facts).
    fn offset(&mut self) -> u32 {
        self.peek();
        self.tokens
            .get(self.at)
            .map_or(self.source.len() as u32, |tok| {
                self.logical.at(tok.start as usize)
            })
    }

    fn error(&mut self, code: PugErrorCode) {
        let offset = self.offset();
        self.errors.push(PugError { code, offset });
    }

    fn boxed<T>(&self, value: T) -> Box<'a, T> {
        Box::new_in(value, &self.allocator)
    }

    fn nodes(&self) -> Vec<'a, PugNode<'a>> {
        Vec::new_in(&self.allocator)
    }

    /// Parse one expression into `nodes`, forcing progress on a token no
    /// rule consumes (a stray structural token the lexer never emits in
    /// well-formed input).
    fn push_expr(&mut self, nodes: &mut Vec<'a, PugNode<'a>>) {
        let before = self.at;
        // Nesting (indentation, `a: b: c`, `#[…]`) is recursion depth.
        let node = vize_s0::ensure_sufficient_stack(|| self.parse_expr());
        nodes.push(node);
        if self.at == before {
            let token = self.take();
            nodes.push(self.unexpected(token));
        }
    }

    fn unexpected(&self, token: Token<'a>) -> PugNode<'a> {
        let block = self.nodes();
        PugNode::Unexpected(self.boxed(PugUnexpected { token, block }))
    }

    /// `indent expr* outdent`.
    fn block(&mut self) -> Vec<'a, PugNode<'a>> {
        self.skip();
        let mut nodes = self.nodes();
        loop {
            match self.peek() {
                Tk::Outdent => {
                    self.skip();
                    break;
                }
                Tk::Eos => break,
                Tk::Newline => self.skip(),
                _ => self.push_expr(&mut nodes),
            }
        }
        nodes
    }

    fn parse_expr(&mut self) -> PugNode<'a> {
        match self.peek() {
            Tk::Tag | Tk::Class | Tk::Id => self.parse_tag(),
            Tk::Keyword(refusal) => {
                let head = self.take();
                let body = self.peek_is(Tk::PipelessStart).then(|| self.text_block());
                let block = if self.peek_is(Tk::Indent) {
                    self.block()
                } else {
                    self.nodes()
                };
                PugNode::Refused(self.boxed(PugRefused {
                    refusal,
                    head,
                    body,
                    block,
                }))
            }
            Tk::Comment => {
                let marker = self.take();
                let text = self.take();
                let body = self.peek_is(Tk::PipelessStart).then(|| self.text_block());
                PugNode::Comment(self.boxed(PugComment { marker, text, body }))
            }
            Tk::Pipe | Tk::Text | Tk::Escape | Tk::CodeInterp | Tk::InterpOpen => {
                let text = self.text_run(true);
                PugNode::Text(self.boxed(text))
            }
            Tk::TextHtml => self.parse_html(),
            Tk::Dot => self.parse_dot(false),
            Tk::CodeFlag => {
                let code = self.parse_code(false);
                PugNode::Code(self.boxed(code))
            }
            Tk::Indent => {
                // pug: `INVALID_TOKEN` on an indent no construct owns.
                self.error(PugErrorCode::InvalidToken);
                let at = self.offset() as usize;
                let token = Token::present(&self.source[self.cursor..at], &self.source[at..at]);
                self.cursor = at;
                let block = self.block();
                PugNode::Unexpected(self.boxed(PugUnexpected { token, block }))
            }
            Tk::Newline | Tk::Outdent | Tk::Eos | Tk::InterpClose | Tk::InterpCloseMissing => {
                self.error(PugErrorCode::InvalidToken);
                let at = self.offset() as usize;
                let token = Token::present(&self.source[self.cursor..at], &self.source[at..at]);
                self.cursor = at;
                self.unexpected(token)
            }
            Tk::Unexpected => {
                let token = self.take();
                self.unexpected(token)
            }
            _ => {
                self.error(PugErrorCode::InvalidToken);
                let token = self.take();
                self.unexpected(token)
            }
        }
    }

    /// `(=|!=|-) code`, with an indented block unless `inline`.
    fn parse_code(&mut self, inline: bool) -> PugCode<'a> {
        let flag = self.take();
        let code = if self.peek_is(Tk::Code) {
            self.take()
        } else {
            Token::missing(
                &self.source[self.cursor..self.cursor],
                &self.source[self.cursor..self.cursor],
            )
        };
        let block = if !inline && self.peek_is(Tk::Indent) {
            self.block()
        } else {
            self.nodes()
        };
        PugCode { flag, code, block }
    }
}
