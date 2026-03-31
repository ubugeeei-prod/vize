//! Tokenizer callback implementation for the parser.
//!
//! Contains the `ParserCallbacks` struct that bridges the tokenizer
//! with the parser by implementing the `Callbacks` trait.

use crate::tokenizer::{Callbacks, QuoteType};
use vize_relief::errors::ErrorCode;

use super::Parser;

/// Parse directive name from raw attribute name
pub(super) fn parse_directive_name(raw: &str) -> &str {
    // Handle shorthand
    match raw.chars().next() {
        Some(':') => return "bind",
        Some('@') => return "on",
        Some('#') => return "slot",
        Some('.') => return "bind", // .prop shorthand
        _ => {}
    }

    // Handle v-directive
    if let Some(rest) = raw.strip_prefix("v-") {
        // Find end of directive name (before : or .)
        let end = rest.find([':', '.']).unwrap_or(rest.len());
        return &rest[..end];
    }

    raw
}

/// Wrapper struct for implementing Callbacks
pub(super) struct ParserCallbacks<'a, 'p> {
    pub(super) parser: &'p mut Parser<'a>,
}

impl<'a, 'p> Callbacks for ParserCallbacks<'a, 'p> {
    fn on_text(&mut self, start: usize, end: usize) {
        self.parser.on_text_impl(start, end);
    }

    fn on_text_entity(&mut self, char: char, start: usize, end: usize) {
        let _ = char;
        self.parser.on_text_impl(start, end);
    }

    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.parser.on_interpolation_impl(start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.parser.on_open_tag_name_impl(start, end);
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.parser.on_open_tag_end_impl(end);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        self.parser.on_self_closing_tag_impl(end);
        self.parser.on_open_tag_end_impl(end);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.parser.on_close_tag_impl(start, end);
    }

    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.parser.on_attrib_data_impl(start, end);
    }

    fn on_attrib_entity(&mut self, _char: char, start: usize, end: usize) {
        self.parser.on_attrib_entity_impl(start, end);
    }

    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        self.parser.on_attrib_end_impl(quote, end);
    }

    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.parser.on_attrib_name_impl(start, end);
    }

    fn on_attrib_name_end(&mut self, _end: usize) {
        // No-op for now
    }

    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.parser.on_dir_name_impl(start, end);
    }

    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.parser.on_dir_arg_impl(start, end);
    }

    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.parser.on_dir_modifier_impl(start, end);
    }

    fn on_comment(&mut self, start: usize, end: usize) {
        self.parser.on_comment_impl(start, end);
    }

    fn on_cdata(&mut self, _start: usize, _end: usize) {
        // CDATA handling
    }

    fn on_processing_instruction(&mut self, _start: usize, _end: usize) {
        // Processing instruction handling
    }

    fn on_end(&mut self) {
        // End of input
    }

    fn on_error(&mut self, code: ErrorCode, index: usize) {
        self.parser.on_error_impl(code, index);
    }

    fn is_in_v_pre(&self) -> bool {
        self.parser.in_v_pre
    }
}
