//! The node writer: the L1 surface tree as the documented template AST.
//!
//! The L1 component surface is the authored, lossless tree: every comment
//! stays in place and every token is a slice of the source, so each node's
//! location is exact. Nodes are written in source order, which keeps every
//! position query on the forward path of the [`Cursor`].

use vize_atelier_core::options::CustomElementMatcher;
use vize_l1::markup::entity::EntityContext;
use vize_l1::markup::{ComponentParse, Element, ElementClose, Interpolation, SurfaceChild, Token};

use super::json::Json;
use super::position::{Cursor, Position};

pub(super) struct Writer<'s> {
    pub(super) source: &'s str,
    pub(super) json: Json,
    pub(super) cursor: Cursor<'s>,
    pub(super) custom_elements: CustomElementMatcher,
}

impl<'s> Writer<'s> {
    pub(super) fn new(source: &'s str, custom_elements: CustomElementMatcher) -> Self {
        Self {
            source,
            // The JSON tree is a small multiple of its source.
            json: Json::with_capacity(source.len().saturating_mul(4).max(256)),
            cursor: Cursor::new(source),
            custom_elements,
        }
    }

    /// The byte offset of a token slice in the source.
    pub(super) fn offset(&self, slice: &str) -> u32 {
        let offset = (slice.as_ptr() as usize).wrapping_sub(self.source.as_ptr() as usize);
        debug_assert!(offset <= self.source.len(), "L1 tokens are source slices");
        offset.min(self.source.len()) as u32
    }

    pub(super) fn start(&self, token: &Token<'_>) -> u32 {
        self.offset(token.text)
    }

    pub(super) fn end(&self, token: &Token<'_>) -> u32 {
        self.start(token) + token.text.len() as u32
    }

    /// `{"start":…,"end":…}` from an already-taken start position.
    pub(super) fn loc_from(&mut self, start: Position, end: u32) {
        let end = self.cursor.at(end);
        self.loc_pair(start, end);
    }

    pub(super) fn loc_pair(&mut self, start: Position, end: Position) {
        self.json.raw("{\"start\":");
        self.position(start);
        self.json.raw(",\"end\":");
        self.position(end);
        self.json.raw("}");
    }

    pub(super) fn loc(&mut self, start: u32, end: u32) {
        let start = self.cursor.at(start);
        self.loc_from(start, end);
    }

    fn position(&mut self, position: Position) {
        self.json.raw("{\"offset\":");
        self.json.number(position.offset);
        self.json.raw(",\"line\":");
        self.json.number(position.line);
        self.json.raw(",\"column\":");
        self.json.number(position.column);
        self.json.raw("}");
    }

    pub(super) fn root(&mut self, parse: &ComponentParse<'_>) {
        self.json.raw("{\"ast\":{\"type\":\"ROOT\",\"children\":[");
        let start = self.cursor.at(0);
        self.children(&parse.tree.children, false, false);
        self.json.raw("],\"loc\":");
        self.loc_from(start, self.source.len() as u32);
        self.json.raw("}");
        super::diagnostics::write(self, parse);
        self.json.raw("}");
    }

    /// Write a children level; returns the end of the last child's extent.
    fn children(
        &mut self,
        children: &[SurfaceChild<'_>],
        raw_text: bool,
        verbatim: bool,
    ) -> Option<u32> {
        let mut first = true;
        let mut last_end = None;
        for child in children {
            let (start, end) = match child {
                SurfaceChild::Element(element) => {
                    self.separator(&mut first);
                    last_end = Some(self.element(element, verbatim));
                    continue;
                }
                SurfaceChild::Interpolation(interpolation) => {
                    self.separator(&mut first);
                    last_end = Some(self.interpolation(interpolation));
                    continue;
                }
                SurfaceChild::Text(token) => {
                    self.separator(&mut first);
                    self.text(token.text, raw_text);
                    (self.start(token), self.end(token))
                }
                SurfaceChild::Comment(token) => {
                    self.separator(&mut first);
                    self.comment(token.text);
                    (self.start(token), self.end(token))
                }
                SurfaceChild::Cdata(token) => {
                    self.separator(&mut first);
                    self.cdata(token.text);
                    (self.start(token), self.end(token))
                }
                // A stray end tag or a processing instruction has no node in
                // Vue's tree; its diagnostic is in `errors`.
                SurfaceChild::ProcessingInstruction(token) | SurfaceChild::Unexpected(token) => {
                    (self.start(token), self.end(token))
                }
            };
            debug_assert!(start <= end);
            last_end = Some(end);
        }
        last_end
    }

    fn separator(&mut self, first: &mut bool) {
        if !*first {
            self.json.raw(",");
        }
        *first = false;
    }

    fn element(&mut self, element: &Element<'_>, parent_verbatim: bool) -> u32 {
        let start = self.cursor.at(self.start(&element.open.lt_name));
        let tag = element.tag();
        let verbatim = element.open.is_verbatim();
        self.json.raw("{\"type\":\"ELEMENT\",\"tag\":");
        self.json.string(tag);
        self.json.raw(",\"tagType\":\"");
        self.json
            .raw(super::props::tag_type(self, element, verbatim));
        self.json.raw("\",\"isSelfClosing\":");
        self.json.boolean(element.open.slash.is_some());
        self.json.raw(",\"props\":[");
        super::props::write(self, &element.open.attrs, verbatim, parent_verbatim);
        self.json.raw("],\"children\":[");
        // Raw-text elements keep their character references, as in Vue.
        let raw_text = matches!(tag, "script" | "style");
        let last_child = self.children(&element.children, raw_text, verbatim);
        let open_end = self.end(&element.open.gt);
        let end = match &element.close {
            ElementClose::Present(close) => self.end(&close.gt),
            ElementClose::NotExpected => open_end,
            ElementClose::Implicit | ElementClose::Missing => last_child.unwrap_or(open_end),
        };
        self.json.raw("],\"loc\":");
        self.loc_from(start, end);
        self.json.raw("}");
        end
    }

    fn text(&mut self, text: &str, raw_text: bool) {
        let start = self.offset(text);
        self.json.raw("{\"type\":\"TEXT\",\"content\":");
        if raw_text {
            self.json.string(text);
        } else {
            self.json.decoded(text, EntityContext::Text);
        }
        self.json.raw(",\"loc\":");
        self.loc(start, start + text.len() as u32);
        self.json.raw("}");
    }

    fn comment(&mut self, text: &str) {
        let start = self.offset(text);
        let inner = text.strip_prefix("<!--").unwrap_or(text);
        // `<!-->` and `<!--->` are abrupt empty comments; an unterminated
        // comment runs to the end of the input.
        let content = inner
            .strip_suffix("-->")
            .or_else(|| inner.strip_suffix("--!>"))
            .unwrap_or(if matches!(inner, ">" | "->") {
                ""
            } else {
                inner
            });
        self.json.raw("{\"type\":\"COMMENT\",\"content\":");
        self.json.string(content);
        self.json.raw(",\"loc\":");
        self.loc(start, start + text.len() as u32);
        self.json.raw("}");
    }

    fn cdata(&mut self, text: &str) {
        let start = self.offset(text);
        let inner = text.strip_prefix("<![CDATA[").unwrap_or(text);
        let content = inner.strip_suffix("]]>").unwrap_or(inner);
        self.json.raw("{\"type\":\"TEXT\",\"content\":");
        self.json.string(content);
        self.json.raw(",\"loc\":");
        self.loc(start, start + text.len() as u32);
        self.json.raw("}");
    }

    fn interpolation(&mut self, interpolation: &Interpolation<'_>) -> u32 {
        let start = self.cursor.at(self.start(&interpolation.open));
        let end = self.end(&interpolation.close);
        let content = interpolation.content.text;
        let expression = content.trim_matches(is_html_whitespace);
        let expression_start = self.offset(expression);
        self.json.raw("{\"type\":\"INTERPOLATION\",\"content\":");
        self.expression(
            expression,
            false,
            expression_start,
            expression_start + expression.len() as u32,
            Some(EntityContext::Text),
        );
        self.json.raw(",\"loc\":");
        self.loc_from(start, end);
        self.json.raw("}");
        end
    }

    /// `{"type":"SIMPLE_EXPRESSION",…}` for `[start, end)`; `decode` is the
    /// character-reference context Vue decodes the content in, if any.
    pub(super) fn expression(
        &mut self,
        content: &str,
        is_static: bool,
        start: u32,
        end: u32,
        decode: Option<EntityContext>,
    ) {
        self.json
            .raw("{\"type\":\"SIMPLE_EXPRESSION\",\"content\":");
        match decode {
            Some(context) => self.json.decoded(content, context),
            None => self.json.string(content),
        }
        self.json.raw(",\"isStatic\":");
        self.json.boolean(is_static);
        self.json.raw(",\"loc\":");
        self.loc(start, end);
        self.json.raw("}");
    }
}

/// The whitespace Vue's tokenizer trims around an interpolation.
const fn is_html_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r' | '\x0c')
}
