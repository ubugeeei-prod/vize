//! Test-only original first-scalar Armature consumer ownership.
//!
//! The production parser uses CompatSink<ParserCallbacks> directly. This
//! reference intercepts only attribute entity values, before the private full
//! callback override, so BEFORE never silently inherits the new AFTER policy.
//! All other compatibility operations remain delegated to the original sink.

use crate::tokenizer::Callbacks;
use vize_l1::markup::{
    LexErrorCode, LexMode, QuoteType, Sink, entity::DecodedEntity, lex::compat::adapter::CompatSink,
};

pub(super) struct OriginalFirstScalarSink<C> {
    compat: CompatSink<C>,
}

impl<C> OriginalFirstScalarSink<C> {
    pub(super) fn new(callbacks: C) -> Self {
        Self {
            compat: CompatSink::new(callbacks),
        }
    }

    #[cfg(test)]
    fn into_callbacks(self) -> C {
        self.compat.into_callbacks()
    }
}

impl<C: Callbacks> Sink for OriginalFirstScalarSink<C> {
    fn on_text(&mut self, start: usize, end: usize) {
        self.compat.on_text(start, end);
    }

    fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
        self.compat.on_text_entity(ch, start, end);
    }

    fn on_text_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        self.compat.on_text_entity_value(value, start, end);
    }

    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.compat.on_interpolation(start, end);
    }

    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        self.compat.on_raw_interpolation(start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.compat.on_open_tag_name(start, end);
    }

    fn svg_title_children(&self) -> bool {
        self.compat.svg_title_children()
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.compat.on_open_tag_end(end);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        self.compat.on_self_closing_tag(end);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.compat.on_close_tag(start, end);
    }

    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.compat.on_attrib_data(start, end);
    }

    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
        self.compat.on_attrib_entity(ch, start, end);
    }

    fn on_attrib_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        // This is a scalar projection of an already-decoded value, not a
        // decoder or production fallback. Bypass the opt-in complete callback.
        let first = match value {
            DecodedEntity::Named(text) => text.chars().next(),
            DecodedEntity::Numeric(ch) => Some(ch),
        };
        if let Some(first) = first {
            self.compat.on_attrib_entity(first, start, end);
        }
    }

    fn on_missing_attribute_value_at_eof(&mut self, end: usize) {
        self.compat.on_missing_attribute_value_at_eof(end);
    }

    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        self.compat.on_attrib_end(quote, end);
    }

    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.compat.on_attrib_name(start, end);
    }

    fn on_attrib_name_end(&mut self, end: usize) {
        self.compat.on_attrib_name_end(end);
    }

    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.compat.on_dir_name(start, end);
    }

    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.compat.on_dir_arg(start, end);
    }

    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.compat.on_dir_modifier(start, end);
    }

    fn on_comment(&mut self, start: usize, end: usize) {
        self.compat.on_comment(start, end);
    }

    fn on_in_tag_comment(&mut self, start: usize, end: usize) {
        self.compat.on_in_tag_comment(start, end);
    }

    fn on_cdata(&mut self, start: usize, end: usize) {
        self.compat.on_cdata(start, end);
    }

    fn on_processing_instruction(&mut self, start: usize, end: usize) {
        self.compat.on_processing_instruction(start, end);
    }

    fn on_declaration(&mut self, start: usize, end: usize, terminated: bool) {
        self.compat.on_declaration(start, end, terminated);
    }

    fn on_declaration_recovery(&mut self, start: usize, end: usize, terminated: bool) {
        self.compat.on_declaration_recovery(start, end, terminated);
    }

    fn on_end(&mut self) {
        self.compat.on_end();
    }

    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        self.compat.on_error(code, index);
    }

    fn mode(&self) -> LexMode {
        self.compat.mode()
    }

    fn directive_heads_in_verbatim(&self) -> bool {
        self.compat.directive_heads_in_verbatim()
    }
}

#[cfg(test)]
mod tests;
