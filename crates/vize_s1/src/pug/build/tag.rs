//! `pug-parser`'s `tag()`: head parts, the same-line content switch, and
//! the indented (or text-only) block.

use vize_s0::Vec;

use super::Builder;
use crate::pug::error::PugErrorCode;
use crate::pug::lex::Tk;
use crate::pug::tree::{
    PugAttr, PugAttrGroup, PugAttrItem, PugBlock, PugInline, PugNode, PugTag, PugTagPart, PugText,
    PugTextPiece,
};

impl<'a> Builder<'a, '_> {
    pub(super) fn parse_tag(&mut self) -> PugNode<'a> {
        let name = self.peek_is(Tk::Tag).then(|| self.take());
        let mut parts = Vec::new_in(&self.allocator);
        loop {
            let part = match self.peek() {
                Tk::Class => PugTagPart::Class(self.take()),
                Tk::Id => PugTagPart::Id(self.take()),
                Tk::AndAttributes => PugTagPart::AndAttributes(self.take()),
                Tk::AttrsOpen => PugTagPart::Attrs({
                    let value = self.attr_group();
                    self.boxed(value)
                }),
                _ => break,
            };
            parts.push(part);
        }
        let dot = self.peek_is(Tk::Dot).then(|| self.take());
        let inline = self.inline();
        while self.peek_is(Tk::Newline) {
            self.skip();
        }
        let block = if dot.is_some() {
            let text = if self.peek_is(Tk::PipelessStart) {
                self.text_block()
            } else {
                PugText {
                    pieces: Vec::new_in(&self.allocator),
                }
            };
            PugBlock::Text(text)
        } else if self.peek_is(Tk::Indent) {
            PugBlock::Nodes(self.block())
        } else {
            PugBlock::Nodes(self.nodes())
        };
        PugNode::Tag(self.boxed(PugTag {
            name,
            parts,
            dot,
            inline,
            block,
        }))
    }

    /// `(text | code | ':' expr | '/')?` — pug's switch after the head.
    fn inline(&mut self) -> PugInline<'a> {
        match self.peek() {
            Tk::Pipe | Tk::Text | Tk::CodeInterp => PugInline::Text(self.text_run(false)),
            Tk::CodeFlag => PugInline::Code({
                let value = self.parse_code(true);
                self.boxed(value)
            }),
            Tk::Colon => {
                let colon = self.take();
                let node = vize_s0::ensure_sufficient_stack(|| self.parse_expr());
                PugInline::Expansion {
                    colon,
                    node: self.boxed(node),
                }
            }
            Tk::Slash => PugInline::SelfClosing(self.take()),
            Tk::Newline
            | Tk::Indent
            | Tk::Outdent
            | Tk::Eos
            | Tk::PipelessStart
            | Tk::InterpClose
            | Tk::InterpCloseMissing => PugInline::None,
            _ => {
                // pug: `INVALID_TOKEN`. Keep the rest of the line verbatim.
                let mut pieces = Vec::new_in(&self.allocator);
                while !matches!(
                    self.peek(),
                    Tk::Newline
                        | Tk::Indent
                        | Tk::Outdent
                        | Tk::Eos
                        | Tk::PipelessStart
                        | Tk::InterpClose
                        | Tk::InterpCloseMissing
                ) {
                    if !self.peek_is(Tk::Unexpected) {
                        self.error(PugErrorCode::InvalidToken);
                    }
                    pieces.push(PugTextPiece::Unexpected(self.take()));
                }
                PugInline::Text(PugText { pieces })
            }
        }
    }

    /// `( attribute* )` — the lexer already split keys, ops and values.
    fn attr_group(&mut self) -> PugAttrGroup<'a> {
        let open = self.take();
        let mut items = Vec::new_in(&self.allocator);
        loop {
            match self.peek() {
                Tk::AttrName => {
                    let name = self.take();
                    let op = self.peek_is(Tk::AttrOp).then(|| self.take());
                    let value = self.peek_is(Tk::AttrValue).then(|| self.take());
                    let comma = self.peek_is(Tk::AttrComma).then(|| self.take());
                    items.push(PugAttrItem::Attr(PugAttr {
                        name,
                        op,
                        value,
                        comma,
                    }));
                }
                Tk::Unexpected => items.push(PugAttrItem::Unexpected(self.take())),
                _ => break,
            }
        }
        let close = self.take();
        PugAttrGroup { open, items, close }
    }
}
