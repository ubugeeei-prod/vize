use vize_relief::ErrorCode;

use crate::char_codes::AMP;
use crate::tokenizer::sequences::Sequence;

use super::{
    Tokenizer,
    char_codes::{
        AT, COLON, DASH, DOT, DOUBLE_QUOTE, EQ, EXCLAMATION_MARK, GRAVE_ACCENT, GT, LEFT_SQUARE,
        LOWER_V, LT, NUMBER, QUESTION_MARK, RIGHT_SQUARE, SINGLE_QUOTE, SLASH,
    },
    types::{Callbacks, QuoteType, State, is_end_of_tag_section, is_tag_start_char, is_whitespace},
};

use super::entity_decode::try_decode_entity;
use htmlize::Context;

impl<'a, C: Callbacks> Tokenizer<'a, C> {
    pub(super) fn cleanup(&mut self) {
        let has_section = self.section_start < self.index;

        match self.state {
            State::Text if has_section => {
                self.callbacks.on_text(self.section_start, self.index);
            }
            State::Interpolation | State::InterpolationClose if has_section => {
                self.callbacks
                    .on_error(ErrorCode::MissingInterpolationEnd, self.index);
                let start = self.section_start.saturating_sub(self.delimiter_open.len());
                self.callbacks.on_text(start, self.index);
            }
            State::BeforeTagName if has_section => {
                self.callbacks
                    .on_error(ErrorCode::EofBeforeTagName, self.index);
                self.callbacks.on_text(self.section_start, self.index);
            }
            State::InTagName
            | State::InSFCRootTagName
            | State::BeforeSpecialS
            | State::BeforeSpecialT
            | State::SpecialStartSequence
            | State::BeforeClosingTagName
            | State::InClosingTagName
            | State::AfterClosingTagName
            | State::BeforeAttrName
            | State::InAttrName
            | State::InDirName
            | State::InDirArg
            | State::InDirDynamicArg
            | State::InDirModifier
            | State::AfterAttrName
            | State::BeforeAttrValue
            | State::InAttrValueDq
            | State::InAttrValueSq
            | State::InAttrValueNq => {
                self.callbacks.on_error(ErrorCode::EofInTag, self.index);
                self.recover_incomplete_tag_at_eof();
            }
            _ => {}
        }

        if self.state == State::InCommentLike {
            let code = match self.current_sequence {
                Some(Sequence::CdataEnd) => ErrorCode::EofInCdata,
                _ => ErrorCode::EofInComment,
            };
            self.callbacks.on_error(code, self.index);
            match self.current_sequence {
                Some(Sequence::CdataEnd) => {
                    self.callbacks.on_cdata(self.section_start, self.index);
                }
                _ => {
                    self.callbacks.on_comment(self.section_start, self.index);
                }
            }
        }
    }

    fn recover_incomplete_tag_at_eof(&mut self) {
        let inferred_tag_end = self.index.saturating_sub(1);

        match self.state {
            State::InTagName
            | State::InSFCRootTagName
            | State::BeforeSpecialS
            | State::BeforeSpecialT
            | State::SpecialStartSequence => {
                self.callbacks
                    .on_open_tag_name(self.section_start, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::BeforeAttrName => {
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrName => {
                self.callbacks
                    .on_attrib_name(self.section_start, self.index);
                self.callbacks.on_attrib_name_end(self.index);
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InDirName => {
                self.callbacks.on_dir_name(self.section_start, self.index);
                self.callbacks.on_attrib_name_end(self.index);
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InDirArg => {
                if self.section_start < self.index {
                    self.callbacks.on_dir_arg(self.section_start, self.index);
                }
                self.callbacks.on_attrib_name_end(self.index);
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InDirDynamicArg => {
                self.callbacks
                    .on_error(ErrorCode::MissingDynamicDirectiveArgumentEnd, self.index);
                if self.section_start < self.index {
                    self.callbacks.on_dir_arg(self.section_start, self.index);
                }
                self.callbacks.on_attrib_name_end(self.index);
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InDirModifier => {
                self.callbacks
                    .on_dir_modifier(self.section_start, self.index);
                self.callbacks.on_attrib_name_end(self.index);
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::AfterAttrName => {
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::BeforeAttrValue => {
                self.callbacks
                    .on_error(ErrorCode::MissingAttributeValue, self.index);
                self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrValueDq => {
                self.emit_unclosed_attr_value(QuoteType::Double);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrValueSq => {
                self.emit_unclosed_attr_value(QuoteType::Single);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrValueNq => {
                self.emit_unclosed_attr_value(QuoteType::Unquoted);
                self.callbacks.on_open_tag_end(inferred_tag_end);
            }
            State::InClosingTagName if self.section_start < self.index => {
                self.callbacks.on_close_tag(self.section_start, self.index);
            }
            State::InClosingTagName | State::BeforeClosingTagName | State::AfterClosingTagName => {}
            _ => {}
        }

        self.state = State::Text;
        self.section_start = self.index;
    }

    fn emit_unclosed_attr_value(&mut self, quote: QuoteType) {
        if self.section_start < self.index {
            self.callbacks
                .on_attrib_data(self.section_start, self.index);
        }
        self.callbacks.on_attrib_end(quote, self.index);
    }

    // ========== State handlers ==========

    pub(super) fn state_text(&mut self, c: u8) {
        if c == LT {
            if self.index > self.section_start {
                self.callbacks.on_text(self.section_start, self.index);
            }
            self.state = State::BeforeTagName;
            self.section_start = self.index;
        } else if c == AMP {
            self.start_entity();
        } else if !self.callbacks.is_in_v_pre() && c == self.delimiter_open[0] {
            self.state = State::InterpolationOpen;
            self.delimiter_index = 0;
            self.state_interpolation_open(c);
        }
    }

    pub(super) fn state_interpolation_open(&mut self, c: u8) {
        if c == self.delimiter_open[self.delimiter_index] {
            self.delimiter_index += 1;
            if self.delimiter_index == self.delimiter_open.len() {
                let start = self.index + 1 - self.delimiter_open.len();
                if start > self.section_start {
                    self.callbacks.on_text(self.section_start, start);
                }
                self.section_start = self.index + 1;
                self.state = State::Interpolation;
                self.delimiter_index = 0;
            }
        } else if self.in_rcdata {
            self.state = State::InRCDATA;
            self.state_in_rcdata(c);
        } else {
            self.state = State::Text;
            self.state_text(c);
        }
    }

    pub(super) fn state_interpolation(&mut self, c: u8) {
        if c == self.delimiter_close[0] {
            self.state = State::InterpolationClose;
            self.delimiter_index = 0;
            self.state_interpolation_close(c);
        }
    }

    pub(super) fn state_interpolation_close(&mut self, c: u8) {
        if c == self.delimiter_close[self.delimiter_index] {
            self.delimiter_index += 1;
            if self.delimiter_index == self.delimiter_close.len() {
                self.callbacks.on_interpolation(
                    self.section_start,
                    self.index + 1 - self.delimiter_close.len(),
                );
                if self.in_rcdata {
                    self.state = State::InRCDATA
                } else {
                    self.state = State::Text
                }
                self.section_start = self.index + 1;
            }
        } else {
            self.state = State::Interpolation;
            self.state_interpolation(c);
        }
    }

    pub(super) fn state_before_tag_name(&mut self, c: u8) {
        if c == EXCLAMATION_MARK {
            self.state = State::BeforeDeclaration;
            self.section_start = self.index + 1;
        } else if c == QUESTION_MARK {
            self.state = State::InProcessingInstruction;
            self.section_start = self.index + 1;
        } else if is_tag_start_char(c) {
            self.section_start = self.index;
            if c == b't' {
                self.state = State::BeforeSpecialT;
            } else if c == b's' {
                self.state = State::BeforeSpecialS;
            } else {
                self.state = State::InTagName;
            }
        } else if c == SLASH {
            self.state = State::BeforeClosingTagName;
        } else {
            self.callbacks
                .on_error(ErrorCode::InvalidFirstCharacterOfTagName, self.index);
            self.state = State::Text;
            self.state_text(c);
        }
    }

    pub(super) fn state_in_tag_name(&mut self, c: u8) {
        if is_end_of_tag_section(c) {
            self.callbacks
                .on_open_tag_name(self.section_start, self.index);
            self.section_start = self.index;
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        }
    }

    pub(super) fn state_in_self_closing_tag(&mut self, c: u8) {
        if c == GT {
            self.callbacks.on_self_closing_tag(self.index);
            self.in_rcdata = false;
            self.current_sequence = None;
            self.sequence_index = 0;
            self.state = State::Text;
            self.section_start = self.index + 1;
        } else if !is_whitespace(c) {
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        }
    }

    pub(super) fn state_before_closing_tag_name(&mut self, c: u8) {
        if is_whitespace(c) {
            // Skip
        } else if c == GT {
            self.callbacks
                .on_error(ErrorCode::MissingEndTagName, self.index);
            self.state = State::Text;
            self.section_start = self.index + 1;
        } else if !is_tag_start_char(c) {
            self.callbacks
                .on_error(ErrorCode::InvalidFirstCharacterOfTagName, self.index);
            self.state = State::InClosingTagName;
            self.section_start = self.index;
        } else {
            self.state = State::InClosingTagName;
            self.section_start = self.index;
        }
    }

    pub(super) fn state_in_closing_tag_name(&mut self, c: u8) {
        if c == GT || is_whitespace(c) {
            self.callbacks.on_close_tag(self.section_start, self.index);
            self.section_start = self.index + 1;
            self.state = if c == GT {
                State::Text
            } else {
                State::AfterClosingTagName
            };
        }
    }

    pub(super) fn state_after_closing_tag_name(&mut self, c: u8) {
        if c == GT {
            self.state = State::Text;
            self.section_start = self.index + 1;
        } else if c == SLASH {
            self.callbacks
                .on_error(ErrorCode::EndTagWithTrailingSolidus, self.index);
        } else if !is_whitespace(c) {
            self.callbacks
                .on_error(ErrorCode::EndTagWithAttributes, self.index);
        }
    }

    pub(super) fn state_before_attr_name(&mut self, c: u8) {
        if c == GT {
            self.after_quoted_attr_value = false;
            self.callbacks.on_open_tag_end(self.index);
            if self.in_rcdata {
                self.state = State::InRCDATA;
            } else {
                self.state = State::Text;
            }
            self.section_start = self.index + 1;
        } else if c == SLASH {
            self.after_quoted_attr_value = false;
            self.state = State::InSelfClosingTag;
        } else if is_whitespace(c) {
            self.after_quoted_attr_value = false;
        } else if c == EQ {
            self.after_quoted_attr_value = false;
            self.callbacks.on_error(
                ErrorCode::UnexpectedEqualsSignBeforeAttributeName,
                self.index,
            );
        } else if !is_whitespace(c) {
            if self.after_quoted_attr_value {
                self.callbacks
                    .on_error(ErrorCode::MissingWhitespaceBetweenAttributes, self.index);
            }
            self.after_quoted_attr_value = false;
            self.handle_attr_start(c);
        }
    }

    pub(super) fn handle_attr_start(&mut self, c: u8) {
        if self.callbacks.is_in_v_pre() {
            self.state = State::InAttrName;
            self.section_start = self.index;
            return;
        }
        if c == LOWER_V && self.index + 1 < self.input.len() && self.input[self.index + 1] == DASH {
            self.state = State::InDirName;
            self.section_start = self.index;
        } else if c == DOT || c == COLON || c == AT || c == NUMBER {
            self.callbacks.on_dir_name(self.index, self.index + 1);
            self.state = State::InDirArg;
            self.section_start = self.index + 1;
        } else {
            self.state = State::InAttrName;
            self.section_start = self.index;
        }
    }

    pub(super) fn state_in_attr_name(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            self.callbacks
                .on_attrib_name(self.section_start, self.index);
            self.callbacks.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == DOUBLE_QUOTE || c == SINGLE_QUOTE {
            self.callbacks
                .on_error(ErrorCode::UnexpectedCharacterInAttributeName, self.index);
            self.callbacks
                .on_attrib_name(self.section_start, self.index);
            self.callbacks.on_attrib_name_end(self.index);
            self.section_start = self.index + 1;
            self.state = if c == DOUBLE_QUOTE {
                State::InAttrValueDq
            } else {
                State::InAttrValueSq
            };
        } else if c == LT {
            self.callbacks
                .on_error(ErrorCode::UnexpectedCharacterInAttributeName, self.index);
        }
    }

    pub(super) fn state_in_dir_name(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            self.callbacks.on_dir_name(self.section_start, self.index);
            self.callbacks.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == COLON {
            self.callbacks.on_dir_name(self.section_start, self.index);
            self.state = State::InDirArg;
            self.section_start = self.index + 1;
        } else if c == DOT {
            self.callbacks.on_dir_name(self.section_start, self.index);
            self.state = State::InDirModifier;
            self.section_start = self.index + 1;
        } else if c == LEFT_SQUARE {
            self.callbacks.on_dir_name(self.section_start, self.index);
            self.state = State::InDirDynamicArg;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_in_dir_arg(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            if self.section_start < self.index {
                self.callbacks.on_dir_arg(self.section_start, self.index);
            }
            self.callbacks.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == LEFT_SQUARE {
            if self.section_start < self.index {
                self.callbacks.on_dir_arg(self.section_start, self.index);
            }
            self.state = State::InDirDynamicArg;
            self.section_start = self.index + 1;
        } else if c == DOT {
            if self.section_start < self.index {
                self.callbacks.on_dir_arg(self.section_start, self.index);
            }
            self.state = State::InDirModifier;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_in_dir_dynamic_arg(&mut self, c: u8) {
        if c == RIGHT_SQUARE {
            self.callbacks.on_dir_arg(self.section_start, self.index);
            self.state = State::InDirArg;
            self.section_start = self.index + 1;
        } else if c == EQ || is_end_of_tag_section(c) {
            self.callbacks
                .on_error(ErrorCode::MissingDynamicDirectiveArgumentEnd, self.index);
            if self.section_start < self.index {
                self.callbacks.on_dir_arg(self.section_start, self.index);
            }
            self.callbacks.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        }
    }

    pub(super) fn state_in_dir_modifier(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            self.callbacks
                .on_dir_modifier(self.section_start, self.index);
            self.callbacks.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == DOT {
            self.callbacks
                .on_dir_modifier(self.section_start, self.index);
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_after_attr_name(&mut self, c: u8) {
        if c == EQ {
            self.state = State::BeforeAttrValue;
        } else if c == SLASH || c == GT {
            self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        } else if !is_whitespace(c) {
            self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
            self.handle_attr_start(c);
        }
    }

    pub(super) fn state_before_attr_value(&mut self, c: u8) {
        if c == DOUBLE_QUOTE {
            self.state = State::InAttrValueDq;
            self.section_start = self.index + 1;
        } else if c == SINGLE_QUOTE {
            self.state = State::InAttrValueSq;
            self.section_start = self.index + 1;
        } else if c == EQ {
            self.callbacks.on_error(
                ErrorCode::UnexpectedEqualsSignBeforeAttributeName,
                self.index,
            );
        } else if c == GT || c == SLASH {
            self.callbacks
                .on_error(ErrorCode::MissingAttributeValue, self.index);
            self.callbacks.on_attrib_end(QuoteType::NoValue, self.index);
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        } else if !is_whitespace(c) {
            self.section_start = self.index;
            self.state = State::InAttrValueNq;
            self.state_in_attr_value_nq(c);
        }
    }

    fn handle_in_attr_value(&mut self, c: u8, quote: u8, quote_type: QuoteType) {
        if c == quote {
            self.emit_attr_value(quote_type);
        } else if c == AMP {
            self.start_entity();
        }
    }

    pub(super) fn state_in_attr_value_dq(&mut self, c: u8) {
        self.handle_in_attr_value(c, DOUBLE_QUOTE, QuoteType::Double);
    }

    pub(super) fn state_in_attr_value_sq(&mut self, c: u8) {
        self.handle_in_attr_value(c, SINGLE_QUOTE, QuoteType::Single);
    }

    pub(super) fn state_in_attr_value_nq(&mut self, c: u8) {
        if is_whitespace(c) || c == GT {
            self.emit_attr_value(QuoteType::Unquoted);
            self.state_before_attr_name(c);
        } else if c == SLASH {
            self.emit_attr_value(QuoteType::Unquoted);
        } else if c == AMP {
            self.start_entity();
        } else if matches!(c, DOUBLE_QUOTE | SINGLE_QUOTE | LT | EQ | GRAVE_ACCENT) {
            self.callbacks.on_error(
                ErrorCode::UnexpectedCharacterInUnquotedAttributeValue,
                self.index,
            );
        }
    }

    pub(super) fn emit_attr_value(&mut self, quote: QuoteType) {
        if self.section_start < self.index {
            self.callbacks
                .on_attrib_data(self.section_start, self.index);
        }
        self.callbacks.on_attrib_end(quote, self.index);
        self.section_start = self.index + 1;
        self.state = State::BeforeAttrName;
        self.after_quoted_attr_value = matches!(quote, QuoteType::Double | QuoteType::Single);
    }

    pub(super) fn state_before_declaration(&mut self, c: u8) {
        if c == DASH {
            self.state = State::BeforeComment;
            self.section_start = self.index + 1;
        } else if c == LEFT_SQUARE {
            self.sequence_index = 0;
            self.state = State::CDATASequence;
            self.section_start = self.index + 1;
        } else {
            self.callbacks.on_error(
                ErrorCode::IncorrectlyOpenedComment,
                self.section_start.saturating_sub(2),
            );
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
            self.callbacks
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
        if c == prefix[self.sequence_index] {
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

    pub(super) fn state_in_special_comment(&mut self, c: u8) {
        if c == GT {
            self.callbacks.on_comment(self.section_start, self.index);
            self.state = State::Text;
            self.section_start = self.index + 1;
        }
    }

    #[inline]
    fn finish_comment_like(&mut self, closing: Sequence) {
        let end = self.index.saturating_sub(2);
        match closing {
            Sequence::CdataEnd => self.callbacks.on_cdata(self.section_start, end),
            Sequence::CommentEnd => self.callbacks.on_comment(self.section_start, end),
            _ => unreachable!("InCommentLike only closes CommentEnd or CdataEnd"),
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

        if c == sequence_bytes[self.sequence_index] {
            self.sequence_index += 1;
            if self.sequence_index == sequence_bytes.len() {
                self.finish_comment_like(sequence);
            }
        } else if self.sequence_index == 0 {
            // Fast-forward to the first character of the sequence
            if self.fast_forward_to(sequence_bytes[0]) {
                self.sequence_index = 1;
            }
        } else if sequence == Sequence::CommentEnd
            && self.sequence_index == 2
            && c == EXCLAMATION_MARK
        {
            self.callbacks
                .on_error(ErrorCode::IncorrectlyClosedComment, self.index);
        } else if c != sequence_bytes[self.sequence_index - 1] {
            // Allow long sequences, eg. --->, ]]]>
            self.sequence_index = 0;
        }
    }

    // </script
    // </style
    pub(super) fn state_before_special_s(&mut self, c: u8) {
        if c == Sequence::ScriptEnd.bytes()[3] {
            self.start_special(Sequence::ScriptEnd, 4);
        } else if c == Sequence::StyleEnd.bytes()[3] {
            self.start_special(Sequence::StyleEnd, 4);
        } else {
            self.state = State::InTagName;
            self.state_in_tag_name(c);
        }
    }

    // </title>
    // </textarea>
    pub(super) fn state_before_special_t(&mut self, c: u8) {
        if c == Sequence::TitleEnd.bytes()[3] {
            self.start_special(Sequence::TitleEnd, 4);
        } else if c == Sequence::TextareaEnd.bytes()[3] {
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
            (c | 0x20) == sequence_bytes[self.sequence_index]
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
                    self.callbacks.on_text(self.section_start, end_of_text);
                    self.index = actual_index;
                }
                self.section_start = end_of_text + 2; // Skip over the `</`
                self.state_in_closing_tag_name(c);
                self.in_rcdata = false;
                return;
            }

            self.sequence_index = 0;
        }

        if (c | 0x20) == sequence_bytes[self.sequence_index] {
            self.sequence_index += 1;
        } else if self.sequence_index == 0 {
            // TODO(SFC root): align with vue-core `(TextareaEnd && !inSFCRoot)` — `<textarea>` at SFC
            // file root should behave as RAWTEXT (no `&`/interpolation here); not distinguished yet.
            if matches!(sequence, Sequence::TitleEnd | Sequence::TextareaEnd) {
                if c == AMP {
                    self.start_entity();
                } else if !self.callbacks.is_in_v_pre() && c == self.delimiter_open[0] {
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
    /// `0` rewind, `<0` wait for more buffer). Here: `Some` → `emit_entity_char`; `None` → rewind
    /// (like `0`); no `<0` path. `Context` follows `base_state` for htmlize attribute rules.
    pub(super) fn state_in_entity(&mut self) {
        let raw = &self.input[self.entity_start..];
        let context = match self.base_state {
            State::Text | State::InRCDATA => Context::General,
            _ => Context::Attribute,
        };

        if let Some((ch, consumed)) = try_decode_entity(raw, context) {
            self.emit_entity_char(ch, consumed);
        } else {
            self.index = self.entity_start;
        }
        self.state = self.base_state;
    }

    pub(super) fn emit_entity_char(&mut self, ch: char, consumed: usize) {
        if self.base_state != State::Text && self.base_state != State::InRCDATA {
            if self.section_start < self.entity_start {
                self.callbacks
                    .on_attrib_data(self.section_start, self.entity_start);
            }
            self.section_start = self.entity_start + consumed;
            self.index = self.section_start - 1;
            self.callbacks
                .on_attrib_entity(ch, self.entity_start, self.section_start);
        } else {
            if self.section_start < self.entity_start {
                self.callbacks
                    .on_text(self.section_start, self.entity_start);
            }
            self.section_start = self.entity_start + consumed;
            self.index = self.section_start - 1;
            self.callbacks
                .on_text_entity(ch, self.entity_start, self.section_start);
        }
    }

    pub(super) fn state_in_sfc_root_tag_name(&mut self, c: u8) {
        if is_end_of_tag_section(c) {
            self.callbacks
                .on_open_tag_name(self.section_start, self.index);
            self.section_start = self.index;
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        }
    }
}
