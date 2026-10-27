//! pug's code generator (`pug-code-gen@3.0.4`, `doctype: "html"`,
//! `pretty: false`) replayed over the L1 tree: the derived Vue template
//! is byte-identical to what the pinned `pug` renders, with build-time
//! code evaluated where it has a fixed value ([`super::eval`]), and every
//! refused construct leaves a diagnostic instead.

use alloc::vec::Vec as StdVec;

use vize_l0::diag::Diagnostic;
use vize_l0::{Span, String, cstr};
use vize_l1::markup::Token;
use vize_l1::pug::{
    PugBlock, PugCode, PugComment, PugHtmlItem, PugInline, PugNode, PugRefusal, PugTag, PugText,
    PugTextPiece, PugTree,
};

use super::mixin::Mixins;

use super::control::interpolated;
use super::eval;
use super::literal::is_js_space;
use super::map::PugSourceMap;
use super::scope::Scope;

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
    /// The bindings pug's build-time code has made so far.
    pub(super) scope: Scope,
    pub(super) mixins: Mixins<'t>,
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
            scope: Scope::default(),
            mixins: Mixins::default(),
        }
    }

    pub(super) fn source_start(&self) -> usize {
        self.source.as_ptr() as usize
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

    pub(super) fn nodes(&mut self, nodes: &'t [PugNode<'t>]) {
        let mut index = 0;
        while let Some(node) = nodes.get(index) {
            index += 1;
            match node {
                PugNode::Refused(refused)
                    if matches!(refused.refusal, PugRefusal::Conditional | PugRefusal::Each) =>
                {
                    let rest = nodes.get(index..).unwrap_or_default();
                    index += vize_l0::ensure_sufficient_stack(|| self.control(refused, rest));
                }
                node => vize_l0::ensure_sufficient_stack(|| self.node(node)),
            }
        }
    }

    fn node(&mut self, node: &'t PugNode<'t>) {
        match node {
            PugNode::Tag(tag) => self.tag(tag),
            PugNode::Call(call) => self.call(call),
            PugNode::Refused(refused) if refused.refusal == PugRefusal::Mixin => {
                self.define(refused);
            }
            PugNode::Refused(refused) if refused.refusal == PugRefusal::MixinBlock => {
                self.mixin_block(refused);
            }
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

    fn tag(&mut self, tag: &'t PugTag<'t>) {
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
        self.content(tag);
        self.synth("</", head);
        self.name(tag, head);
        self.synth(">", head);
    }

    /// A tag's (or a mixin call's) same-line content and block.
    pub(super) fn content(&mut self, tag: &'t PugTag<'t>) {
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
    }

    fn name(&mut self, tag: &PugTag<'_>, head: Span) {
        match &tag.name {
            Some(token) => self.verbatim(token),
            None => self.synth("div", head),
        }
    }

    fn head_offset(&self, tag: &PugTag<'_>) -> u32 {
        tag.parts.first().map_or(0, |part| match part {
            vize_l1::pug::PugTagPart::Class(token)
            | vize_l1::pug::PugTagPart::Id(token)
            | vize_l1::pug::PugTagPart::AndAttributes(token) => self.offset(token),
            vize_l1::pug::PugTagPart::Attrs(group) => self.offset(&group.open),
        })
    }

    pub(super) fn text(&mut self, text: &'t PugText<'t>) {
        for piece in &text.pieces {
            match piece {
                PugTextPiece::Raw(token) => self.verbatim(token),
                PugTextPiece::Newline(at) => self.synth("\n", Span::new(*at, *at)),
                PugTextPiece::Interpolation(interpolation) => self.nodes(&interpolation.nodes),
                PugTextPiece::Code(token) => {
                    // An unclosed `#{` is already a surface error.
                    if let Some(source) = interpolated(token.text) {
                        let span = self.span(token);
                        self.buffer(source, token.text.starts_with('#'), span);
                    }
                }
                PugTextPiece::Pipe(_) | PugTextPiece::Escape(_) | PugTextPiece::Unexpected(_) => {}
            }
        }
    }

    /// `parseTextHtml`'s run: consecutive HTML text joins with `\n`,
    /// including HTML nodes of a nested block; anything else breaks it.
    fn html(&mut self, items: &'t [PugHtmlItem<'t>]) {
        let mut open = false;
        for item in items {
            match item {
                PugHtmlItem::Line(text) => {
                    self.html_join(&mut open, text);
                    self.text(text);
                }
                PugHtmlItem::Block(nodes) => {
                    let mut run = 0;
                    for (index, node) in nodes.iter().enumerate() {
                        let PugNode::Html(html) = node else {
                            continue;
                        };
                        if run < index {
                            self.nodes(nodes.get(run..index).unwrap_or_default());
                            open = false;
                        }
                        run = index + 1;
                        if open {
                            let at = self.html_anchor(&html.items);
                            self.synth("\n", Span::new(at, at));
                        }
                        self.html(&html.items);
                        open = true;
                    }
                    if run < nodes.len() {
                        self.nodes(nodes.get(run..).unwrap_or_default());
                        open = false;
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

    /// `= value` / `!= value` output the evaluated value; `- …` runs
    /// declarations.
    fn code(&mut self, code: &'t PugCode<'t>) {
        let span = self.span(&code.code);
        if !code.buffered() {
            let whole = Span::new(self.offset(&code.flag), span.end);
            if !code.block.is_empty() {
                self.error(whole, String::from(super::refusal::CODE_BLOCK));
            } else if let Err(reason) = eval::statements(&mut self.scope, code.code.text) {
                self.error(whole, super::refusal::statement(&reason));
            }
            return;
        }
        if !code.block.is_empty() {
            self.error(
                span,
                String::from("Buffered code cannot have a block attached to it"),
            );
        }
        self.buffer(code.code.text, code.escaped(), span);
    }

    /// pug's buffered output: `null`/`undefined` render nothing, other
    /// values their string, HTML-escaped unless `!=` / `!{…}`.
    fn buffer(&mut self, source: &str, escaped: bool, span: Span) {
        let value = match eval::expression(&self.scope, source) {
            Ok(evaluated) if evaluated.value.has_function() => {
                self.error(span, super::refusal::code(super::refusal::FUNCTION_VALUE));
                return;
            }
            Ok(evaluated) => evaluated.value,
            Err(reason) => {
                self.error(span, super::refusal::code(&reason));
                return;
            }
        };
        let text = if value.is_nullish() {
            String::default()
        } else {
            value.to_js_string()
        };
        let text = if escaped {
            super::attrs::escape(&text)
        } else {
            text
        };
        self.synth(&text, span);
    }

    fn comment(&mut self, comment: &'t PugComment<'t>) {
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
