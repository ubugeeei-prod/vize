//! Observe original Vue 1 text callbacks without changing lexer or scope policy.

use super::super::surface::Vue1Policy;
use super::{TextBinding, TextBoundary, TextBoundaryKind, observe};
use crate::dialect::vue::surface::sink::VueSink;
use crate::markup::entity::DecodedEntity;
use crate::markup::token::{LexErrorCode, LexMode, QuoteType, Sink};
use alloc::vec::Vec;
use vize_l0::{Allocator, SourceBlock, Span};

pub(in crate::dialect::vue1) struct TextSink<'a, 'v> {
    pub allocator: &'a Allocator,
    pub block: SourceBlock<'a>,
    pub inner: VueSink<'a, 'v, Vue1Policy>,
    pub bindings: &'v mut Vec<TextBinding<'a>>,
    pub boundaries: &'v mut Vec<TextBoundary>,
}

macro_rules! forward {
    ($($name:ident($($arg:ident: $ty:ty),*);)*) => {
        $(fn $name(&mut self, $($arg: $ty),*) { self.inner.$name($($arg),*); })*
    };
}
impl Sink for TextSink<'_, '_> {
    forward! {
        on_text(start: usize, end: usize);
        on_attrib_data(start: usize, end: usize);
        on_attrib_entity(ch: char, start: usize, end: usize);
        on_attrib_entity_value(value: DecodedEntity, start: usize, end: usize);
        on_attrib_end(quote: QuoteType, end: usize);
        on_attrib_name(start: usize, end: usize);
        on_attrib_name_end(end: usize);
        on_dir_name(start: usize, end: usize);
        on_dir_arg(start: usize, end: usize);
        on_dir_modifier(start: usize, end: usize);
        on_comment(start: usize, end: usize);
        on_in_tag_comment(start: usize, end: usize);
        on_cdata(start: usize, end: usize);
        on_processing_instruction(start: usize, end: usize);
        on_declaration(start: usize, end: usize, terminated: bool);
        on_declaration_recovery(start: usize, end: usize, terminated: bool);
        on_open_tag_name(start: usize, end: usize);
        on_open_tag_end(end: usize);
        on_self_closing_tag(end: usize);
        on_close_tag(start: usize, end: usize);
        on_end();
        on_error(code: LexErrorCode, index: usize);
    }
    fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
        if matches!(ch, '{' | '}') {
            self.encoded_delimiter(start, end);
        }
        self.inner.on_text_entity(ch, start, end);
    }
    fn on_text_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        let mut brace = false;
        value.for_each(|ch| brace |= matches!(ch, '{' | '}'));
        if brace {
            self.encoded_delimiter(start, end);
        }
        self.inner.on_text_entity_value(value, start, end);
    }
    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.observe(start, end, false);
        self.inner.on_interpolation(start, end);
    }
    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        self.observe(start, end, true);
        self.inner.on_raw_interpolation(start, end);
    }
    fn mode(&self) -> LexMode {
        self.inner.mode()
    }
}
impl TextSink<'_, '_> {
    fn observe(&mut self, start: usize, end: usize, raw: bool) {
        if let Some(binding) = observe(self.allocator, self.block, start, end, raw) {
            self.bindings.push(binding);
        } else {
            // Private callbacks should always be authentic UTF-8 windows. Fail
            // closed on an invalid one without inventing replacement bytes.
            self.boundaries.push(TextBoundary {
                span: self.block.span(),
                kind: TextBoundaryKind::SourcePreparation,
            });
        }
    }
    fn encoded_delimiter(&mut self, start: usize, end: usize) {
        // Literal v-pre retains entity spelling without interpretation.
        if self.inner.mode() == LexMode::Verbatim {
            return;
        }
        self.boundaries.push(TextBoundary {
            span: Span::new(
                self.block.start() + start as u32,
                self.block.start() + end as u32,
            ),
            kind: TextBoundaryKind::EncodedDelimiter,
        });
    }
}
