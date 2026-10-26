//! Syntax checks for the code of branches pug skips: its lexer rejects
//! an invalid expression wherever it appears.

use vize_l0::{String, cstr};
use vize_l1::pug::{
    PugAttrItem, PugBlock, PugHtmlItem, PugInline, PugNode, PugRefusal, PugTagPart, PugText,
    PugTextPiece,
};

use super::control::interpolated;
use super::emit::Emitter;
use super::eval;
use super::heads::{Branch, Each, branch, each};

impl Emitter<'_> {
    /// pug's lexer rejects an invalid expression wherever it appears.
    pub(super) fn check_expression(&mut self, source: &str) {
        if !eval::parses(source) {
            let span = self.text_span(source);
            let message = cstr!("pug code `{}` is not valid JavaScript", source.trim());
            self.error(span, message);
        }
    }

    /// Syntax-check the expressions of nodes pug does not run.
    pub(super) fn check(&mut self, nodes: &[PugNode<'_>]) {
        for node in nodes {
            vize_l0::ensure_sufficient_stack(|| self.check_node(node));
        }
    }

    fn check_node(&mut self, node: &PugNode<'_>) {
        match node {
            PugNode::Tag(tag) => {
                for part in &tag.parts {
                    let PugTagPart::Attrs(group) = part else {
                        continue;
                    };
                    for item in &group.items {
                        if let PugAttrItem::Attr(attr) = item
                            && let Some(value) = &attr.value
                        {
                            self.check_expression(value.text);
                        }
                    }
                }
                match &tag.inline {
                    PugInline::Text(text) => self.check_text(text),
                    PugInline::Code(code) if code.buffered() => {
                        self.check_expression(code.code.text);
                    }
                    PugInline::Expansion { node, .. } => self.check_node(node),
                    _ => {}
                }
                match &tag.block {
                    PugBlock::Nodes(nodes) => self.check(nodes),
                    PugBlock::Text(text) => self.check_text(text),
                }
            }
            PugNode::Text(text) => self.check_text(text),
            PugNode::TextBlock(block) => self.check_text(&block.text),
            PugNode::Code(code) if code.buffered() => self.check_expression(code.code.text),
            PugNode::Html(html) => {
                for item in &html.items {
                    match item {
                        PugHtmlItem::Line(text) => self.check_text(text),
                        PugHtmlItem::Block(nodes) => self.check(nodes),
                        PugHtmlItem::Code(code) if code.buffered() => {
                            self.check_expression(code.code.text);
                        }
                        PugHtmlItem::Code(_) => {}
                    }
                }
            }
            PugNode::Refused(refused) => {
                match refused.refusal {
                    PugRefusal::Conditional => {
                        if let Ok(Branch::If { test, .. } | Branch::ElseIf(test)) =
                            branch(refused.head.text)
                        {
                            self.check_expression(test);
                        }
                    }
                    PugRefusal::Each => match each(refused.head.text) {
                        Ok(Each::In { subject, .. } | Each::Of { subject, .. }) => {
                            self.check_expression(subject);
                        }
                        Err(message) => {
                            let span = self.span(&refused.head);
                            self.error(span, String::from(message));
                        }
                    },
                    _ => {}
                }
                self.check(&refused.block);
            }
            _ => {}
        }
    }

    fn check_text(&mut self, text: &PugText<'_>) {
        for piece in &text.pieces {
            match piece {
                PugTextPiece::Code(token) => {
                    if let Some(source) = interpolated(token.text) {
                        self.check_expression(source);
                    }
                }
                PugTextPiece::Interpolation(interpolation) => self.check(&interpolation.nodes),
                _ => {}
            }
        }
    }
}
