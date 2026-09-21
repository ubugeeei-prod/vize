//! Text shapes: `parseText` runs (with pug's newline-join rule),
//! `parseTextBlock` bodies, `#[…]` interpolations, `parseTextHtml` runs
//! and a line-start `.` block.

use vize_s0::Vec;

use super::Builder;
use crate::pug::error::PugErrorCode;
use crate::pug::lex::Tk;
use crate::pug::tree::{
    PugDotBlock, PugHtml, PugHtmlItem, PugInterpolation, PugNode, PugText, PugTextPiece,
};
use crate::surface::Token;

impl<'a> Builder<'a, '_> {
    /// `parseText`: in a block, a newline followed by more text joins the
    /// lines with a `\n` piece; anything else ends the run.
    pub(super) fn text_run(&mut self, block: bool) -> PugText<'a> {
        let mut pieces = Vec::new_in(&self.allocator);
        loop {
            let piece = match self.peek() {
                Tk::Pipe => PugTextPiece::Pipe(self.take()),
                Tk::Text => PugTextPiece::Raw(self.take()),
                Tk::Escape => PugTextPiece::Escape(self.take()),
                Tk::CodeInterp => PugTextPiece::Code(self.take()),
                Tk::InterpOpen => PugTextPiece::Interpolation({
                    let value = self.interpolation();
                    self.boxed(value)
                }),
                Tk::Newline if block => {
                    let at = self.offset();
                    self.skip();
                    if matches!(self.peek(), Tk::Pipe | Tk::Text | Tk::CodeInterp) {
                        pieces.push(PugTextPiece::Newline(at));
                    }
                    continue;
                }
                _ => break,
            };
            pieces.push(piece);
        }
        PugText { pieces }
    }

    /// `parseTextBlock`: every line of a pipeless block, each line break a
    /// `\n` piece.
    pub(super) fn text_block(&mut self) -> PugText<'a> {
        self.skip();
        let mut pieces = Vec::new_in(&self.allocator);
        loop {
            let piece = match self.peek() {
                Tk::PipelessEnd => {
                    self.skip();
                    break;
                }
                Tk::Eos => break,
                Tk::Text => PugTextPiece::Raw(self.take()),
                Tk::Escape => PugTextPiece::Escape(self.take()),
                Tk::CodeInterp => PugTextPiece::Code(self.take()),
                Tk::InterpOpen => PugTextPiece::Interpolation({
                    let value = self.interpolation();
                    self.boxed(value)
                }),
                Tk::Newline => {
                    let at = self.offset();
                    self.skip();
                    PugTextPiece::Newline(at)
                }
                _ => {
                    self.error(PugErrorCode::InvalidToken);
                    PugTextPiece::Unexpected(self.take())
                }
            };
            pieces.push(piece);
        }
        PugText { pieces }
    }

    /// `#[ expr ]`, the close a `Missing` hole when the line ran out.
    fn interpolation(&mut self) -> PugInterpolation<'a> {
        let open = self.take();
        let mut nodes = self.nodes();
        while !matches!(
            self.peek(),
            Tk::InterpClose | Tk::InterpCloseMissing | Tk::Eos
        ) {
            self.push_expr(&mut nodes);
        }
        if nodes.len() != 1 {
            self.error(PugErrorCode::InvalidToken);
        }
        let close = match self.peek() {
            Tk::InterpClose => self.take(),
            Tk::InterpCloseMissing => self.take_missing(),
            _ => Token::missing(
                &self.source[self.cursor..self.cursor],
                &self.source[self.cursor..self.cursor],
            ),
        };
        PugInterpolation { open, nodes, close }
    }

    /// `parseTextHtml`: `<…` lines, their indented blocks and inline code.
    pub(super) fn parse_html(&mut self) -> PugNode<'a> {
        let mut items = Vec::new_in(&self.allocator);
        loop {
            let item = match self.peek() {
                Tk::TextHtml => {
                    let mut pieces = Vec::new_in(&self.allocator);
                    loop {
                        let piece = match self.peek() {
                            Tk::TextHtml => PugTextPiece::Raw(self.take()),
                            Tk::Escape => PugTextPiece::Escape(self.take()),
                            _ => break,
                        };
                        pieces.push(piece);
                    }
                    PugHtmlItem::Line(PugText { pieces })
                }
                Tk::Indent => PugHtmlItem::Block(self.block()),
                Tk::CodeFlag => PugHtmlItem::Code({
                    let value = self.parse_code(true);
                    self.boxed(value)
                }),
                Tk::Newline => {
                    self.skip();
                    continue;
                }
                _ => break,
            };
            items.push(item);
        }
        PugNode::Html(self.boxed(PugHtml { items }))
    }

    /// `parseDot`: a line-start `.` and its text block. Without a block
    /// pug's `parseDot` yields no node: the top level skips it, anywhere
    /// else pug crashes on it, so that is diagnosed.
    pub(super) fn parse_dot(&mut self, top_level: bool) -> PugNode<'a> {
        let dot = self.take();
        let text = if self.peek_is(Tk::PipelessStart) {
            self.text_block()
        } else {
            if !top_level {
                self.error(PugErrorCode::InvalidToken);
            }
            PugText {
                pieces: Vec::new_in(&self.allocator),
            }
        };
        PugNode::TextBlock(self.boxed(PugDotBlock { dot, text }))
    }
}
