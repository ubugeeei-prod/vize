//! pug's code generator (`pug-code-gen@3.0.4`, `doctype: "html"`,
//! `pretty: false`) replayed over the S1 tree: the derived Vue template
//! is byte-identical to what the pinned `pug` renders for the static
//! subset, and every refused construct leaves a diagnostic instead.

use alloc::vec::Vec as StdVec;

use vize_davinci::diagnostic::Diagnostic;
use vize_s0::{Span, String, cstr};
use vize_s1::Token;
use vize_s1::pug::{
    PugBlock, PugCode, PugComment, PugHtmlItem, PugInline, PugNode, PugTag, PugText, PugTextPiece,
    PugTree,
};

use super::literal::{Lit, evaluate, is_js_space};
use super::map::PugSourceMap;

/// `void-elements@3.1.0`, pug's self-closing table.
const VOID: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub(super) struct Emitter<'t> {
    source: &'t str,
    pub(super) html: String,
    pub(super) map: PugSourceMap,
    pub(super) diagnostics: StdVec<Diagnostic>,
    compiled_tag: bool,
    pub(super) rendering: super::PugRendering,
}

impl<'t> Emitter<'t> {
    pub(super) fn new(tree: &PugTree<'t>, rendering: super::PugRendering) -> Self {
        Self {
            rendering,
            source: tree.source,
            html: String::with_capacity(tree.source.len()),
            map: PugSourceMap::default(),
            diagnostics: StdVec::new(),
            compiled_tag: false,
        }
    }

    pub(super) fn offset(&self, token: &Token<'_>) -> u32 {
        (token.text.as_ptr() as usize).saturating_sub(self.source.as_ptr() as usize) as u32
    }

    pub(super) fn span(&self, token: &Token<'_>) -> Span {
        let start = self.offset(token);
        Span::new(start, start + token.text.len() as u32)
    }

    /// Copy a token's bytes (a verbatim map segment).
    pub(super) fn verbatim(&mut self, token: &Token<'_>) {
        let span = self.span(token);
        self.html.push_str(token.text);
        self.map.push_verbatim(token.text.len(), span.start);
    }

    /// Emit bytes pug synthesizes for the construct at `span`.
    pub(super) fn synth(&mut self, text: &str, span: Span) {
        self.html.push_str(text);
        self.map
            .push_synth(text.len(), span.start, span.end.max(span.start));
    }

    pub(super) fn error(&mut self, span: Span, message: String) {
        self.diagnostics
            .push(crate::exemptions::lowering(span, &message));
    }

    pub(super) fn nodes(&mut self, nodes: &[PugNode<'_>]) {
        for node in nodes {
            vize_s0::ensure_sufficient_stack(|| self.node(node));
        }
    }

    fn node(&mut self, node: &PugNode<'_>) {
        match node {
            PugNode::Tag(tag) => self.tag(tag),
            PugNode::Text(text) => self.text(text),
            PugNode::Html(html) => self.html(&html.items),
            PugNode::TextBlock(block) => self.text(&block.text),
            PugNode::Code(code) => self.code(code),
            PugNode::Comment(comment) => self.comment(comment),
            PugNode::Refused(refused) => {
                let span = self.span(&refused.head);
                self.error(span, super::refusal::message(refused.refusal));
            }
            // The surface parser already reported why these bytes have no
            // structure; they contribute nothing.
            PugNode::Unexpected(_) => {}
        }
    }

    fn tag(&mut self, tag: &PugTag<'_>) {
        let name = tag.tag_name();
        let head = tag.name.as_ref().map_or_else(
            || Span::new(self.head_offset(tag), self.head_offset(tag)),
            |token| self.span(token),
        );
        if !self.compiled_tag {
            if name == "html" {
                self.synth("<!DOCTYPE html>", head);
            }
            self.compiled_tag = true;
        }
        self.synth("<", head);
        self.name(tag, head);
        self.attributes(tag);
        let self_closing = matches!(tag.inline, PugInline::SelfClosing(_));
        if self_closing || VOID.contains(&name) {
            self.synth(if self_closing { "/>" } else { ">" }, head);
            if has_content(tag) {
                let message = cstr!(
                    "{name} is a self closing element: <{name}/> but contains nested content."
                );
                self.error(head, message);
            }
            return;
        }
        self.synth(">", head);
        match &tag.inline {
            PugInline::Text(text) => self.text(text),
            PugInline::Code(code) => self.code(code),
            PugInline::Expansion { node, .. } => self.nodes(core::slice::from_ref(&**node)),
            PugInline::None | PugInline::SelfClosing(_) => {}
        }
        match &tag.block {
            PugBlock::Nodes(nodes) => self.nodes(nodes),
            PugBlock::Text(text) => self.text(text),
        }
        self.synth("</", head);
        self.name(tag, head);
        self.synth(">", head);
    }

    fn name(&mut self, tag: &PugTag<'_>, head: Span) {
        match &tag.name {
            Some(token) => self.verbatim(token),
            None => self.synth("div", head),
        }
    }

    fn head_offset(&self, tag: &PugTag<'_>) -> u32 {
        tag.parts.first().map_or(0, |part| match part {
            vize_s1::pug::PugTagPart::Class(token)
            | vize_s1::pug::PugTagPart::Id(token)
            | vize_s1::pug::PugTagPart::AndAttributes(token) => self.offset(token),
            vize_s1::pug::PugTagPart::Attrs(group) => self.offset(&group.open),
        })
    }

    pub(super) fn text(&mut self, text: &PugText<'_>) {
        for piece in &text.pieces {
            match piece {
                PugTextPiece::Raw(token) => self.verbatim(token),
                PugTextPiece::Newline(at) => self.synth("\n", Span::new(*at, *at)),
                PugTextPiece::Interpolation(interpolation) => self.nodes(&interpolation.nodes),
                PugTextPiece::Code(token) => {
                    let span = self.span(token);
                    self.error(span, String::from(super::refusal::CODE_INTERPOLATION));
                }
                PugTextPiece::Pipe(_) | PugTextPiece::Escape(_) | PugTextPiece::Unexpected(_) => {}
            }
        }
    }

    /// `parseTextHtml`'s run: consecutive HTML text joins with `\n`,
    /// including HTML nodes of a nested block; anything else breaks it.
    fn html(&mut self, items: &[PugHtmlItem<'_>]) {
        let mut open = false;
        for item in items {
            match item {
                PugHtmlItem::Line(text) => {
                    self.html_join(&mut open, text);
                    self.text(text);
                }
                PugHtmlItem::Block(nodes) => {
                    for node in nodes {
                        if let PugNode::Html(html) = node {
                            if open {
                                let at = self.html_anchor(&html.items);
                                self.synth("\n", Span::new(at, at));
                            }
                            self.html(&html.items);
                            open = true;
                        } else {
                            self.nodes(core::slice::from_ref(node));
                            open = false;
                        }
                    }
                }
                PugHtmlItem::Code(code) => {
                    self.code(code);
                    open = false;
                }
            }
        }
    }

    fn html_join(&mut self, open: &mut bool, text: &PugText<'_>) {
        if *open {
            let at = text.pieces.first().map_or(0, |piece| match piece {
                PugTextPiece::Raw(token) => self.offset(token),
                _ => 0,
            });
            self.synth("\n", Span::new(at, at));
        }
        *open = true;
    }

    fn html_anchor(&self, items: &[PugHtmlItem<'_>]) -> u32 {
        match items.first() {
            Some(PugHtmlItem::Line(text)) => match text.pieces.first() {
                Some(PugTextPiece::Raw(token)) => self.offset(token),
                _ => 0,
            },
            _ => 0,
        }
    }

    /// `= value` / `!= value`: pug escapes (`=`) or copies (`!=`) the
    /// folded value; `null`/`undefined` render nothing.
    fn code(&mut self, code: &PugCode<'_>) {
        let span = self.span(&code.code);
        if !code.buffered() {
            let whole = Span::new(self.offset(&code.flag), span.end);
            self.error(whole, String::from(super::refusal::UNBUFFERED_CODE));
            return;
        }
        if !code.block.is_empty() {
            self.error(
                span,
                String::from("Buffered code cannot have a block attached to it"),
            );
        }
        let Some(value) = evaluate(code.code.text) else {
            self.error(span, String::from(super::refusal::EXECUTABLE_CODE));
            return;
        };
        let value = match value {
            Lit::Null | Lit::Undefined => String::default(),
            other => other.to_js_string(),
        };
        let value = if code.escaped() {
            super::attrs::escape(&value)
        } else {
            value
        };
        self.synth(&value, span);
    }

    fn comment(&mut self, comment: &PugComment<'_>) {
        if !comment.buffered() {
            return;
        }
        let span = self.span(&comment.marker);
        self.synth("<!--", span);
        self.verbatim(&comment.text);
        if let Some(body) = &comment.body {
            self.text(body);
        }
        self.synth("-->", span);
    }
}

/// pug's `SELF_CLOSING_CONTENT` test: code, or a block node that is not
/// whitespace-only text.
fn has_content(tag: &PugTag<'_>) -> bool {
    let text_has_content = |text: &PugText<'_>| {
        text.pieces.iter().any(|piece| match piece {
            PugTextPiece::Raw(token) => !token.text.chars().all(is_js_space),
            PugTextPiece::Newline(_) | PugTextPiece::Pipe(_) => false,
            _ => true,
        })
    };
    let inline = match &tag.inline {
        PugInline::Text(text) => text_has_content(text),
        PugInline::Code(_) | PugInline::Expansion { .. } => true,
        PugInline::None | PugInline::SelfClosing(_) => false,
    };
    inline
        || match &tag.block {
            PugBlock::Nodes(nodes) => nodes.iter().any(|node| match node {
                PugNode::Text(text) => text_has_content(text),
                _ => true,
            }),
            PugBlock::Text(text) => text_has_content(text),
        }
}
