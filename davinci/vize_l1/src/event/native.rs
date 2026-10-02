//! Native events enter the shared surface recorder without a legacy adapter.

use vize_l0::ErrorCode;

use super::{EventKind, Recorder};
use crate::markup::entity::DecodedEntity;
use crate::markup::token::{LexErrorCode, QuoteType, Sink};
use crate::parse::SurfaceError;

impl Sink for Recorder<'_, '_> {
    fn on_text(&mut self, start: usize, end: usize) {
        self.push(EventKind::Text, start, end);
    }

    fn on_text_entity(&mut self, _ch: char, start: usize, end: usize) {
        self.push(EventKind::Text, start, end);
    }

    fn on_text_entity_value(&mut self, _value: DecodedEntity, start: usize, end: usize) {
        // A complete entity value may contain several scalars, but its authored
        // bytes have one span. Recording each scalar would duplicate coverage.
        self.push(EventKind::Text, start, end);
    }

    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.push(EventKind::Interpolation, start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.push(EventKind::OpenTagName, start, end);
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.push(EventKind::OpenTagEnd, end, end);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        self.push(EventKind::SelfClosingTag, end, end);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.push(EventKind::CloseTag, start, end);
    }

    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrData, start, end);
    }

    fn on_attrib_entity(&mut self, _ch: char, start: usize, end: usize) {
        self.push(EventKind::AttrData, start, end);
    }

    fn on_attrib_entity_value(&mut self, _value: DecodedEntity, start: usize, end: usize) {
        self.push(EventKind::AttrData, start, end);
    }

    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        self.attr_end(quote, end);
    }

    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_attrib_name_end(&mut self, end: usize) {
        self.push(EventKind::AttrNameEnd, end, end);
    }

    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_comment(&mut self, start: usize, end: usize) {
        self.push(EventKind::Comment, start, end);
    }

    // In-tag comments keep their bytes in the next token's leading slice.

    fn on_cdata(&mut self, start: usize, end: usize) {
        self.push(EventKind::Cdata, start, end);
    }

    fn on_processing_instruction(&mut self, start: usize, end: usize) {
        self.push(EventKind::ProcessingInstruction, start, end);
    }

    fn on_end(&mut self) {}

    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        self.errors.push(SurfaceError {
            code: surface_error_code(code),
            offset: index as u32,
        });
    }
}

/// Keep the surface diagnostic contract stable without the feature-gated
/// callback adapter. Exhaustiveness makes additions to the native enum fail
/// compilation until their surface diagnostic has been specified.
fn surface_error_code(code: LexErrorCode) -> ErrorCode {
    match code {
        LexErrorCode::AbruptClosingOfEmptyComment => ErrorCode::AbruptClosingOfEmptyComment,
        LexErrorCode::EndTagWithAttributes => ErrorCode::EndTagWithAttributes,
        LexErrorCode::EndTagWithTrailingSolidus => ErrorCode::EndTagWithTrailingSolidus,
        LexErrorCode::EofBeforeTagName => ErrorCode::EofBeforeTagName,
        LexErrorCode::EofInCdata => ErrorCode::EofInCdata,
        LexErrorCode::EofInComment => ErrorCode::EofInComment,
        LexErrorCode::EofInTag => ErrorCode::EofInTag,
        LexErrorCode::IncorrectlyClosedComment => ErrorCode::IncorrectlyClosedComment,
        LexErrorCode::IncorrectlyOpenedComment => ErrorCode::IncorrectlyOpenedComment,
        LexErrorCode::InvalidFirstCharacterOfTagName => ErrorCode::InvalidFirstCharacterOfTagName,
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
    }
}
