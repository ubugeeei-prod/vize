use htmlize::Context;

use super::{
    Lexer,
    char_codes::{AMP, DASH, EXCLAMATION_MARK, GT, LEFT_SQUARE, LT},
    sequences::Sequence,
    types::{State, is_end_of_tag_section, is_whitespace},
};
use crate::markup::entity::{DecodedEntity, decode::try_decode_entity};
use crate::markup::profile::Profile;
use crate::markup::token::{LexErrorCode, LexMode, Sink};

impl<P: Profile, S: Sink> Lexer<'_, P, S> {
    pub(super) fn state_before_declaration(&mut self, c: u8) {
        if c == DASH {
            self.state = State::BeforeComment;
            self.section_start = self.index + 1;
        } else if c == LEFT_SQUARE {
            self.sequence_index = 0;
            self.state = State::CDATASequence;
            self.section_start = self.index + 1;
        } else {
            // In document mode a leading `<!DOCTYPE html>` (or any markup
            // declaration) is expected; skip it silently rather than reporting
            // a spurious recoverable error. SFC `<template>` mode keeps the
            // original behavior so existing output stays byte-identical.
            if !P::TOLERATE_DECLARATIONS {
                self.sink.on_error(
                    LexErrorCode::IncorrectlyOpenedComment,
                    self.section_start.saturating_sub(2),
                );
            }
            self.state = State::InDeclaration;
        }
    }

    pub(super) fn state_in_declaration(&mut self, c: u8) {
        if c == GT {
            self.state = State::Text;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_in_processing_instruction(&mut self, c: u8) {
        if c == GT {
            self.sink
                .on_processing_instruction(self.section_start, self.index);
            self.state = State::Text;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_before_comment(&mut self, c: u8) {
        if c == DASH {
            self.sequence_index = 2;
            self.state = State::InCommentLike;
            self.current_sequence = Some(Sequence::CommentEnd);
            self.section_start = self.index + 1;
        } else {
            self.state = State::InDeclaration;
        }
    }

    pub(super) fn state_cdata_sequence(&mut self, c: u8) {
        let prefix = Sequence::Cdata.bytes();
        if prefix.get(self.sequence_index) == Some(&c) {
            self.sequence_index += 1;
            if self.sequence_index == prefix.len() {
                self.state = State::InCommentLike;
                self.current_sequence = Some(Sequence::CdataEnd);
                self.sequence_index = 0;
                self.section_start = self.index + 1;
            }
        } else {
            self.sequence_index = 0;
            self.state = State::InDeclaration;
            self.state_in_declaration(c);
        }
    }

    #[inline]
    fn finish_comment_like(&mut self, closing: Sequence) {
        let end = self.index.saturating_sub(2);
        match closing {
            Sequence::CdataEnd => self.sink.on_cdata(self.section_start, end),
            _ => self.sink.on_comment(self.section_start, end),
        }
        self.sequence_index = 0;
        self.current_sequence = None;
        self.section_start = self.index + 1;
        self.state = State::Text;
    }

    #[inline]
    fn current_sequence_with_bytes(&self) -> Option<(Sequence, &'static [u8])> {
        self.current_sequence
            .map(|sequence| (sequence, sequence.bytes()))
    }

    pub(super) fn state_in_comment_like(&mut self, c: u8) {
        let Some((sequence, sequence_bytes)) = self.current_sequence_with_bytes() else {
            self.state = State::Text;
            return;
        };

        if sequence == Sequence::CommentEnd
            && self.sequence_index == 2
            && c == GT
            && self.index.saturating_sub(self.section_start) < 2
        {
            self.sink
                .on_error(LexErrorCode::AbruptClosingOfEmptyComment, self.index);
            self.sink.on_comment(self.section_start, self.section_start);
            self.sequence_index = 0;
            self.current_sequence = None;
            self.section_start = self.index + 1;
            self.state = State::Text;
        } else if sequence_bytes.get(self.sequence_index) == Some(&c) {
            self.sequence_index += 1;
            if self.sequence_index == sequence_bytes.len() {
                self.finish_comment_like(sequence);
            }
        } else if self.sequence_index == 0 {
            if sequence == Sequence::CommentEnd
                && self.input.get(self.index..self.index + 4) == Some(b"<!--")
            {
                self.sink.on_error(LexErrorCode::NestedComment, self.index);
            } else if sequence != Sequence::CommentEnd
                && self.fast_forward_to(sequence.first_byte())
            {
                self.sequence_index = 1;
            }
        } else if sequence == Sequence::CommentEnd
            && self.sequence_index == 2
            && c == EXCLAMATION_MARK
        {
            self.sink
                .on_error(LexErrorCode::IncorrectlyClosedComment, self.index);
        } else if sequence_bytes.get(self.sequence_index - 1) != Some(&c) {
            // Allow long sequences, eg. --->, ]]]>
            self.sequence_index = 0;
        }
    }

    // </script
    // </style
    pub(super) fn state_before_special_s(&mut self, c: u8) {
        if Sequence::ScriptEnd.bytes().get(3) == Some(&c) {
            self.start_special(Sequence::ScriptEnd, 4);
        } else if Sequence::StyleEnd.bytes().get(3) == Some(&c) {
            self.start_special(Sequence::StyleEnd, 4);
        } else {
            self.state = State::InTagName;
            self.state_in_tag_name(c);
        }
    }

    // </title>
    // </textarea>
    pub(super) fn state_before_special_t(&mut self, c: u8) {
        if Sequence::TitleEnd.bytes().get(3) == Some(&c) {
            self.start_special(Sequence::TitleEnd, 4);
        } else if Sequence::TextareaEnd.bytes().get(3) == Some(&c) {
            self.start_special(Sequence::TextareaEnd, 4);
        } else {
            self.state = State::InTagName;
            self.state_in_tag_name(c);
        }
    }

    pub(super) fn enter_rcdata(&mut self, sequence: Sequence, offset: usize) {
        self.in_rcdata = true;
        self.current_sequence = Some(sequence);
        self.sequence_index = offset;
    }

    fn start_special(&mut self, sequence: Sequence, offset: usize) {
        self.enter_rcdata(sequence, offset);
        self.state = State::SpecialStartSequence;
    }

    pub(super) fn state_special_start_sequence(&mut self, c: u8) {
        let Some((_, sequence_bytes)) = self.current_sequence_with_bytes() else {
            self.state = State::InTagName;
            self.state_in_tag_name(c);
            return;
        };

        let is_end = self.sequence_index == sequence_bytes.len();
        let is_match = if is_end {
            is_end_of_tag_section(c)
        } else {
            sequence_bytes.get(self.sequence_index) == Some(&(c | 0x20))
        };

        if !is_match {
            self.in_rcdata = false;
        } else if !is_end {
            self.sequence_index += 1;
            return;
        }

        self.sequence_index = 0;
        self.state = State::InTagName;
        self.state_in_tag_name(c);
    }

    // Look for an end tag. For `<title>` and `<textarea>`, also decode entities and handle
    // interpolation
    pub(super) fn state_in_rcdata(&mut self, c: u8) {
        let Some((sequence, sequence_bytes)) = self.current_sequence_with_bytes() else {
            self.state = State::Text;
            self.state_text(c);
            return;
        };

        if self.sequence_index == sequence_bytes.len() {
            if c == GT || is_whitespace(c) {
                let end_of_text: usize = self.index - sequence_bytes.len();
                if self.section_start < end_of_text {
                    let actual_index = self.index;
                    self.index = end_of_text;
                    self.sink.on_text(self.section_start, end_of_text);
                    self.index = actual_index;
                }
                self.section_start = end_of_text + 2; // Skip over the `</`
                self.state_in_closing_tag_name(c);
                self.in_rcdata = false;
                return;
            }

            self.sequence_index = 0;
        }

        if sequence_bytes.get(self.sequence_index) == Some(&(c | 0x20)) {
            self.sequence_index += 1;
        } else if self.sequence_index == 0 {
            // vue-core skips this for an SFC-root `<textarea>`; armature has no SFC mode.
            if matches!(sequence, Sequence::TitleEnd | Sequence::TextareaEnd) {
                if c == AMP {
                    self.start_entity();
                } else if self.sink.mode() == LexMode::Normal && self.at_opening_delimiter(c) {
                    self.state = State::InterpolationOpen;
                    self.delimiter_index = 0;
                    self.state_interpolation_open(c);
                }
            } else if self.fast_forward_to(LT) {
                // Outside of `<title>` / `<textarea>`, skip ahead to the next `<` (script/style RAWTEXT).
                self.sequence_index = 1;
            }
        } else {
            // If we see `<`, set the sequence index to 1; useful for e.g. `<</script>`.
            self.sequence_index = if c == LT { 1 } else { 0 };
        }
    }

    pub(super) fn start_entity(&mut self) {
        self.base_state = self.state;
        self.state = State::InEntity;
        self.entity_start = self.index;
    }

    /// Vue `stateInEntity` (non-browser): `entityDecoder.write` uses signed length (`>0` done,
    /// `0` rewind, `<0` wait for more buffer). Here: `Some` → emit decoded value; `None` → rewind
    /// (like `0`); no `<0` path. `Context` follows `base_state` for htmlize attribute rules.
    pub(super) fn state_in_entity(&mut self) {
        let raw = self.input.get(self.entity_start..).unwrap_or_default();
        let context = match self.base_state {
            State::Text | State::InRCDATA => Context::General,
            _ => Context::Attribute,
        };

        if let Some((decoded, consumed)) = try_decode_entity(raw, context) {
            self.emit_entity_value(decoded, consumed);
        } else {
            self.index = self.entity_start;
        }
        self.state = self.base_state;
    }

    pub(super) fn emit_entity_value(&mut self, value: DecodedEntity, consumed: usize) {
        if self.base_state != State::Text && self.base_state != State::InRCDATA {
            if self.section_start < self.entity_start {
                self.sink
                    .on_attrib_data(self.section_start, self.entity_start);
            }
            self.section_start = self.entity_start + consumed;
            self.index = self.section_start - 1;
            self.sink
                .on_attrib_entity_value(value, self.entity_start, self.section_start);
        } else {
            if self.section_start < self.entity_start {
                self.sink.on_text(self.section_start, self.entity_start);
            }
            self.section_start = self.entity_start + consumed;
            self.index = self.section_start - 1;
            self.sink
                .on_text_entity_value(value, self.entity_start, self.section_start);
        }
    }
}
