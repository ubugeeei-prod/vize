//! Rendering a pug tree back to source, the TS-19 byte-fidelity check,
//! and the typed-hole census.
//!
//! Canonical order: each token's `leading` then `text`; a tag's name,
//! head parts, `.`, same-line content, then its block; `Missing` tokens
//! and `Newline` pieces render zero bytes.

use super::tree::{
    PugAttrItem, PugBlock, PugHtmlItem, PugInline, PugNode, PugTag, PugTagPart, PugText,
    PugTextPiece, PugTree,
};
use crate::surface::Token;

/// Feed every rendered piece to `sink`, in canonical order.
pub fn render_pug<'a>(tree: &PugTree<'a>, sink: &mut dyn FnMut(&'a str)) {
    let mut visitor = Visitor { sink, holes: None };
    visitor.nodes(&tree.nodes);
    visitor.token(&tree.eof);
}

/// Verify `render(tree) == tree.source` without allocating; `Err` is the
/// first diverging byte offset.
pub fn check_pug_fidelity(tree: &PugTree<'_>) -> Result<(), usize> {
    let src = tree.source.as_bytes();
    let mut pos = 0usize;
    let mut mismatch = None;
    render_pug(tree, &mut |piece| {
        if mismatch.is_some() {
            return;
        }
        let bytes = piece.as_bytes();
        if src.get(pos..pos + bytes.len()) == Some(bytes) {
            pos += bytes.len();
        } else {
            mismatch = Some(pos);
        }
    });
    match mismatch {
        Some(at) => Err(at),
        None if pos == src.len() => Ok(()),
        None => Err(pos),
    }
}

/// The typed holes a pug tree carries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PugHoleCounts {
    /// Clause 1: `Missing` tokens (an unterminated `#[`).
    pub missing_tokens: usize,
    /// Clause 2: `Unexpected` nodes, pieces and attribute items.
    pub unexpected: usize,
}

pub fn pug_hole_counts(tree: &PugTree<'_>) -> PugHoleCounts {
    let mut counts = PugHoleCounts::default();
    let mut sink = |_: &str| {};
    let mut visitor = Visitor {
        sink: &mut sink,
        holes: Some(&mut counts),
    };
    visitor.nodes(&tree.nodes);
    visitor.token(&tree.eof);
    counts
}

struct Visitor<'s, 'a> {
    sink: &'s mut dyn FnMut(&'a str),
    holes: Option<&'s mut PugHoleCounts>,
}

impl<'a> Visitor<'_, 'a> {
    fn token(&mut self, token: &Token<'a>) {
        if let Some(holes) = self.holes.as_deref_mut() {
            holes.missing_tokens += usize::from(token.is_missing());
        }
        (self.sink)(token.leading);
        (self.sink)(token.text);
    }

    fn unexpected(&mut self, token: &Token<'a>) {
        if let Some(holes) = self.holes.as_deref_mut() {
            holes.unexpected += 1;
        }
        self.token(token);
    }

    fn nodes(&mut self, nodes: &[PugNode<'a>]) {
        for node in nodes {
            vize_s0::ensure_sufficient_stack(|| self.node(node));
        }
    }

    fn node(&mut self, node: &PugNode<'a>) {
        match node {
            PugNode::Tag(tag) => self.tag(tag),
            PugNode::Text(text) => self.text(text),
            PugNode::Html(html) => {
                for item in &html.items {
                    match item {
                        PugHtmlItem::Line(text) => self.text(text),
                        PugHtmlItem::Block(nodes) => self.nodes(nodes),
                        PugHtmlItem::Code(code) => {
                            self.token(&code.flag);
                            self.token(&code.code);
                            self.nodes(&code.block);
                        }
                    }
                }
            }
            PugNode::TextBlock(block) => {
                self.token(&block.dot);
                self.text(&block.text);
            }
            PugNode::Code(code) => {
                self.token(&code.flag);
                self.token(&code.code);
                self.nodes(&code.block);
            }
            PugNode::Comment(comment) => {
                self.token(&comment.marker);
                self.token(&comment.text);
                if let Some(body) = &comment.body {
                    self.text(body);
                }
            }
            PugNode::Refused(refused) => {
                self.token(&refused.head);
                if let Some(body) = &refused.body {
                    self.text(body);
                }
                self.nodes(&refused.block);
            }
            PugNode::Unexpected(unexpected) => {
                self.unexpected(&unexpected.token);
                self.nodes(&unexpected.block);
            }
        }
    }

    fn tag(&mut self, tag: &PugTag<'a>) {
        if let Some(name) = &tag.name {
            self.token(name);
        }
        for part in &tag.parts {
            match part {
                PugTagPart::Class(token) | PugTagPart::Id(token) => self.token(token),
                PugTagPart::AndAttributes(token) => self.token(token),
                PugTagPart::Attrs(group) => {
                    self.token(&group.open);
                    for item in &group.items {
                        match item {
                            PugAttrItem::Attr(attr) => {
                                self.token(&attr.name);
                                for token in
                                    [&attr.op, &attr.value, &attr.comma].into_iter().flatten()
                                {
                                    self.token(token);
                                }
                            }
                            PugAttrItem::Unexpected(token) => self.unexpected(token),
                        }
                    }
                    self.token(&group.close);
                }
            }
        }
        if let Some(dot) = &tag.dot {
            self.token(dot);
        }
        match &tag.inline {
            PugInline::None => {}
            PugInline::Text(text) => self.text(text),
            PugInline::Code(code) => {
                self.token(&code.flag);
                self.token(&code.code);
                self.nodes(&code.block);
            }
            PugInline::Expansion { colon, node } => {
                self.token(colon);
                self.nodes(core::slice::from_ref(&**node));
            }
            PugInline::SelfClosing(slash) => self.token(slash),
        }
        match &tag.block {
            PugBlock::Nodes(nodes) => self.nodes(nodes),
            PugBlock::Text(text) => self.text(text),
        }
    }

    fn text(&mut self, text: &PugText<'a>) {
        for piece in &text.pieces {
            match piece {
                PugTextPiece::Pipe(token)
                | PugTextPiece::Raw(token)
                | PugTextPiece::Escape(token)
                | PugTextPiece::Code(token) => self.token(token),
                PugTextPiece::Newline(_) => {}
                PugTextPiece::Unexpected(token) => self.unexpected(token),
                PugTextPiece::Interpolation(interpolation) => {
                    self.token(&interpolation.open);
                    self.nodes(&interpolation.nodes);
                    self.token(&interpolation.close);
                }
            }
        }
    }
}
