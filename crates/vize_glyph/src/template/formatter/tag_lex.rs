//! Opening and closing tags read from L1's markup lexer.
//!
//! The formatter walks text, comments and interpolations itself; at each `<`
//! it pulls exactly one tag from `vize_l1`'s HTML-ish lexer and turns the
//! tag's events into formatter attributes. No byte of a tag is scanned here.

use super::TemplateFormatter;
use crate::template::{attributes::ParsedAttribute, directives::normalize_attribute_with_vue_version};
use vize_l0::String;
use vize_l1::markup::{
    Component, LexErrorCode, LexOptions, Lexer, QuoteType, Sink, entity::DecodedEntity,
    htmlish::lex::PulledTag,
};

/// An opening tag: its name, attributes, self-closing flag and end offset.
pub(super) type OpeningTag<'s> = (&'s str, Vec<ParsedAttribute>, bool, usize);

struct TagSink<'f, 's> {
    formatter: &'f TemplateFormatter<'f>,
    source: &'s str,
    depth: usize,
    attrs: Vec<ParsedAttribute>,
    name: (usize, usize),
    attr_start: usize,
    attr_name_end: usize,
    value_start: usize,
    self_closing: bool,
    refused: bool,
}

impl Sink for TagSink<'_, '_> {
    #[inline]
    fn on_text(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_text_entity(&mut self, _: char, _: usize, _: usize) {}
    #[inline]
    fn on_interpolation(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.name = (start, end);
    }
    #[inline]
    fn on_open_tag_end(&mut self, _: usize) {}
    #[inline]
    fn on_self_closing_tag(&mut self, _: usize) {
        self.self_closing = true;
    }
    #[inline]
    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.name = (start, end);
    }
    #[inline]
    fn on_attrib_data(&mut self, start: usize, end: usize) {
        let _ = end;
        self.value_start = self.value_start.min(start);
    }
    #[inline]
    fn on_attrib_entity(&mut self, _: char, _: usize, _: usize) {}
    #[inline]
    fn on_attrib_entity_value(&mut self, _: DecodedEntity, start: usize, end: usize) {
        self.on_attrib_data(start, end);
    }
    #[inline]
    fn on_missing_attribute_value_at_eof(&mut self, end: usize) {
        self.on_attrib_end(QuoteType::Unquoted, end);
    }
    #[inline]
    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        let value = match quote {
            QuoteType::NoValue => None,
            _ => Some(if self.value_start == usize::MAX {
                ""
            } else {
                self.source.get(self.value_start..end).unwrap_or_default()
            }),
        };
        let raw_name = self
            .source
            .get(self.attr_start..self.attr_name_end)
            .unwrap_or_default();
        let (name, value, priority, indent_multiline_value, owns_value_lines, order_sensitive) =
            normalize_attribute_with_vue_version(
                raw_name,
                value.map(String::from),
                self.formatter.options,
                self.formatter.vue_version,
                self.depth + self.formatter.base_depth,
            );
        let original_index = self.attrs.len();
        self.attrs.push(ParsedAttribute {
            name,
            value,
            priority,
            original_index,
            indent_multiline_value,
            owns_value_lines,
            order_sensitive,
        });
        self.attr_start = usize::MAX;
        self.value_start = usize::MAX;
    }
    #[inline]
    fn on_attrib_name(&mut self, start: usize, _: usize) {
        self.attr_start = self.attr_start.min(start);
    }
    #[inline]
    fn on_attrib_name_end(&mut self, end: usize) {
        self.attr_name_end = end;
    }
    #[inline]
    fn on_dir_name(&mut self, start: usize, _: usize) {
        self.attr_start = self.attr_start.min(start);
    }
    #[inline]
    fn on_dir_arg(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_dir_modifier(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_comment(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_cdata(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_processing_instruction(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_end(&mut self) {}
    #[inline]
    fn on_error(&mut self, code: LexErrorCode, _: usize) {
        if matches!(
            code,
            LexErrorCode::UnexpectedSolidusInTag
                | LexErrorCode::UnexpectedCharacterInAttributeName
        ) {
            self.refused = true;
        }
    }
}

impl TemplateFormatter<'_> {
    /// Read the opening tag whose `<` is at `start` from the L1 lexer.
    #[inline]
    pub(super) fn lex_opening_tag<'s>(
        &self,
        input: &'s str,
        start: usize,
        depth: usize,
    ) -> Option<OpeningTag<'s>> {
        let mut lexer = Lexer::<Component, _>::new(
            input,
            TagSink {
                formatter: self,
                source: input,
                depth,
                attrs: Vec::new(),
                name: (0, 0),
                attr_start: usize::MAX,
                attr_name_end: 0,
                value_start: usize::MAX,
                self_closing: false,
                refused: false,
            },
            LexOptions::default(),
        );
        let PulledTag::Open { end, .. } = lexer.pull_tag(start) else {
            return None;
        };
        let sink = lexer.into_sink();
        let name = input.get(sink.name.0..sink.name.1)?;
        Some((name, sink.attrs, sink.self_closing, end))
    }
}

/// Read the closing tag whose `<` is at `start` from the L1 lexer.
#[inline]
pub(crate) fn lex_closing_tag(input: &str, start: usize) -> Option<(&str, usize)> {
    let mut lexer = Lexer::<Component, _>::new(input, CloseSink(usize::MAX, 0), LexOptions::default());
    let PulledTag::Close { end, .. } = lexer.pull_tag(start) else {
        return None;
    };
    let CloseSink(name_start, name_end) = lexer.into_sink();
    if name_start != start + 2 {
        return None;
    }
    Some((input.get(name_start..name_end)?, end))
}

struct CloseSink(usize, usize);

impl Sink for CloseSink {
    #[inline]
    fn on_text(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_text_entity(&mut self, _: char, _: usize, _: usize) {}
    #[inline]
    fn on_interpolation(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_open_tag_name(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_open_tag_end(&mut self, _: usize) {}
    #[inline]
    fn on_self_closing_tag(&mut self, _: usize) {}
    #[inline]
    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.0 = start;
        self.1 = end;
    }
    #[inline]
    fn on_attrib_data(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_attrib_entity(&mut self, _: char, _: usize, _: usize) {}
    #[inline]
    fn on_attrib_end(&mut self, _: QuoteType, _: usize) {}
    #[inline]
    fn on_attrib_name(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_attrib_name_end(&mut self, _: usize) {}
    #[inline]
    fn on_dir_name(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_dir_arg(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_dir_modifier(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_comment(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_cdata(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_processing_instruction(&mut self, _: usize, _: usize) {}
    #[inline]
    fn on_end(&mut self) {}
    #[inline]
    fn on_error(&mut self, _: LexErrorCode, _: usize) {}
}

/// Experiment switch: route tags through L1.
pub(crate) static L1_TAGS: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

fn l1() -> bool { L1_TAGS.load(core::sync::atomic::Ordering::Relaxed) }

pub(crate) fn closing(input: &str, start: usize) -> Option<(String, usize)> {
    if l1() {
        lex_closing_tag(input, start).map(|(n, e)| (String::from(n), e))
    } else {
        crate::template::helpers::parse_closing_tag(input, start)
    }
}

impl TemplateFormatter<'_> {
    pub(super) fn opening(&self, input: &str, start: usize, depth: usize) -> Option<(String, Vec<ParsedAttribute>, bool, usize)> {
        if l1() {
            self.lex_opening_tag(input, start, depth).map(|(n, a, s, e)| (String::from(n), a, s, e))
        } else {
            self.parse_opening_tag(input, start, depth)
        }
    }
    pub(super) fn immediate_empty(&self, text: &str, start: usize, tag_name: &str) -> Option<usize> {
        if !l1() {
            return self.parse_immediate_empty_closing_tag(text, start, tag_name);
        }
        let source = text.as_bytes();
        let mut pos = start;
        while pos < source.len() && crate::template::helpers::is_whitespace(source[pos]) {
            pos += 1;
        }
        if source.get(pos..pos + 2) != Some(b"</".as_slice()) {
            return None;
        }
        let (name, end) = lex_closing_tag(text, pos)?;
        (name == tag_name).then_some(end)
    }
}
