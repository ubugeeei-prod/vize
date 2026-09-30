//! Compatibility callback bridge for the profile lexer.
//!
//! The compiler retains `Tokenizer` until #6880's product gate. This bridge
//! lets the existing callback contract be checked against `Lexer<P>` without
//! putting legacy output rules into L1's native sink.

use vize_relief::ErrorCode;

use super::{Callbacks, QuoteType as CompatQuote};
use crate::markup::entity::DecodedEntity;
use crate::markup::token::{LexErrorCode, LexMode, QuoteType, Sink};

/// Adapts the preserved compiler callback vocabulary to the profile lexer.
pub struct CompatSink<C> {
    callbacks: C,
}

impl<C> CompatSink<C> {
    pub fn new(callbacks: C) -> Self {
        Self { callbacks }
    }

    pub fn into_callbacks(self) -> C {
        self.callbacks
    }
}

fn first_scalar(value: DecodedEntity) -> Option<char> {
    match value {
        DecodedEntity::Named(text) => text.chars().next(),
        DecodedEntity::Numeric(ch) => Some(ch),
    }
}

impl<C: Callbacks> Sink for CompatSink<C> {
    fn on_text(&mut self, start: usize, end: usize) {
        self.callbacks.on_text(start, end);
    }

    fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
        self.callbacks.on_text_entity(ch, start, end);
    }

    fn on_text_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        // The old tokenizer only delivered the first scalar of a named
        // reference. Preserve that output at the compatibility boundary.
        if let Some(first) = first_scalar(value) {
            self.callbacks.on_text_entity(first, start, end);
        }
    }

    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.callbacks.on_interpolation(start, end);
    }

    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        self.callbacks.on_raw_interpolation(start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.callbacks.on_open_tag_name(start, end);
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.callbacks.on_open_tag_end(end);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        self.callbacks.on_self_closing_tag(end);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.callbacks.on_close_tag(start, end);
    }

    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.callbacks.on_attrib_data(start, end);
    }

    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
        self.callbacks.on_attrib_entity(ch, start, end);
    }

    fn on_attrib_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        if let Some(first) = first_scalar(value) {
            self.callbacks.on_attrib_entity(first, start, end);
        }
    }

    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        let quote = match quote {
            QuoteType::NoValue => CompatQuote::NoValue,
            QuoteType::Unquoted => CompatQuote::Unquoted,
            QuoteType::Single => CompatQuote::Single,
            QuoteType::Double => CompatQuote::Double,
        };
        self.callbacks.on_attrib_end(quote, end);
    }

    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.callbacks.on_attrib_name(start, end);
    }

    fn on_attrib_name_end(&mut self, end: usize) {
        self.callbacks.on_attrib_name_end(end);
    }

    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.callbacks.on_dir_name(start, end);
    }

    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.callbacks.on_dir_arg(start, end);
    }

    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.callbacks.on_dir_modifier(start, end);
    }

    fn on_comment(&mut self, start: usize, end: usize) {
        self.callbacks.on_comment(start, end);
    }

    fn on_in_tag_comment(&mut self, start: usize, end: usize) {
        self.callbacks.on_in_tag_comment(start, end);
    }

    fn on_cdata(&mut self, start: usize, end: usize) {
        self.callbacks.on_cdata(start, end);
    }

    fn on_processing_instruction(&mut self, start: usize, end: usize) {
        self.callbacks.on_processing_instruction(start, end);
    }

    fn on_end(&mut self) {
        self.callbacks.on_end();
    }

    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        let code = match code {
            LexErrorCode::AbruptClosingOfEmptyComment => ErrorCode::AbruptClosingOfEmptyComment,
            LexErrorCode::EndTagWithAttributes => ErrorCode::EndTagWithAttributes,
            LexErrorCode::EndTagWithTrailingSolidus => ErrorCode::EndTagWithTrailingSolidus,
            LexErrorCode::EofBeforeTagName => ErrorCode::EofBeforeTagName,
            LexErrorCode::EofInCdata => ErrorCode::EofInCdata,
            LexErrorCode::EofInComment => ErrorCode::EofInComment,
            LexErrorCode::EofInTag => ErrorCode::EofInTag,
            LexErrorCode::IncorrectlyClosedComment => ErrorCode::IncorrectlyClosedComment,
            LexErrorCode::IncorrectlyOpenedComment => ErrorCode::IncorrectlyOpenedComment,
            LexErrorCode::InvalidFirstCharacterOfTagName => {
                ErrorCode::InvalidFirstCharacterOfTagName
            }
            LexErrorCode::MissingAttributeValue => ErrorCode::MissingAttributeValue,
            LexErrorCode::MissingDynamicDirectiveArgumentEnd => {
                ErrorCode::MissingDynamicDirectiveArgumentEnd
            }
            LexErrorCode::MissingEndTagName => ErrorCode::MissingEndTagName,
            LexErrorCode::MissingInterpolationEnd => ErrorCode::MissingInterpolationEnd,
            LexErrorCode::MissingWhitespaceBetweenAttributes => {
                ErrorCode::MissingWhitespaceBetweenAttributes
            }
            LexErrorCode::NestedComment => ErrorCode::NestedComment,
            LexErrorCode::UnexpectedCharacterInAttributeName => {
                ErrorCode::UnexpectedCharacterInAttributeName
            }
            LexErrorCode::UnexpectedCharacterInUnquotedAttributeValue => {
                ErrorCode::UnexpectedCharacterInUnquotedAttributeValue
            }
            LexErrorCode::UnexpectedEqualsSignBeforeAttributeName => {
                ErrorCode::UnexpectedEqualsSignBeforeAttributeName
            }
            LexErrorCode::UnexpectedQuestionMarkInsteadOfTagName => {
                ErrorCode::UnexpectedQuestionMarkInsteadOfTagName
            }
            LexErrorCode::UnexpectedSolidusInTag => ErrorCode::UnexpectedSolidusInTag,
        };
        self.callbacks.on_error(code, index);
    }

    fn mode(&self) -> LexMode {
        if self.callbacks.is_in_v_pre() {
            LexMode::Verbatim
        } else {
            LexMode::Normal
        }
    }
}
