use super::{
    Lexer,
    char_codes::{
        AMP, AT, COLON, DASH, DOT, DOUBLE_QUOTE, EQ, EXCLAMATION_MARK, GRAVE_ACCENT, GT,
        LEFT_SQUARE, LOWER_V, LT, NUMBER, QUESTION_MARK, SINGLE_QUOTE, SLASH,
    },
    types::{State, is_end_of_tag_section, is_tag_start_char, is_whitespace},
};
use crate::markup::profile::Profile;
use crate::markup::token::{LexErrorCode, LexMode, QuoteType, Sink};

impl<P: Profile, S: Sink> Lexer<'_, P, S> {
    pub(super) fn state_before_tag_name(&mut self, c: u8) {
        if c == EXCLAMATION_MARK {
            self.state = State::BeforeDeclaration;
            self.section_start = self.index + 1;
        } else if c == QUESTION_MARK {
            self.sink.on_error(
                LexErrorCode::UnexpectedQuestionMarkInsteadOfTagName,
                self.index,
            );
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
            self.state = State::Text;
            self.state_text(c);
        }
    }

    pub(super) fn state_in_tag_name(&mut self, c: u8) {
        if is_end_of_tag_section(c) {
            self.sink.on_open_tag_name(self.section_start, self.index);
            self.section_start = self.index;
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        }
    }

    pub(super) fn state_in_self_closing_tag(&mut self, c: u8) {
        if c == GT {
            self.sink.on_self_closing_tag(self.index);
            self.current_sequence = None;
            self.sequence_index = 0;
            self.state = State::Text;
            self.section_start = self.index + 1;
        } else if !is_whitespace(c) {
            self.sink
                .on_error(LexErrorCode::UnexpectedSolidusInTag, self.section_start);
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        }
    }

    pub(super) fn state_before_closing_tag_name(&mut self, c: u8) {
        if is_whitespace(c) {
        } else if c == GT {
            self.sink
                .on_error(LexErrorCode::MissingEndTagName, self.index);
            self.state = State::Text;
            self.section_start = self.index + 1;
        } else if !is_tag_start_char(c) {
            self.sink
                .on_error(LexErrorCode::InvalidFirstCharacterOfTagName, self.index);
            self.state = State::InClosingTagName;
            self.section_start = self.index;
        } else {
            self.state = State::InClosingTagName;
            self.section_start = self.index;
        }
    }

    pub(super) fn state_in_closing_tag_name(&mut self, c: u8) {
        if c == GT || is_whitespace(c) {
            self.sink.on_close_tag(self.section_start, self.index);
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
            self.sink
                .on_error(LexErrorCode::EndTagWithTrailingSolidus, self.index);
        } else if !is_whitespace(c) {
            self.sink
                .on_error(LexErrorCode::EndTagWithAttributes, self.index);
        }
    }

    #[inline(always)]
    pub(super) fn state_before_attr_name(&mut self, c: u8) {
        if c == GT {
            self.after_quoted_attr_value = false;
            self.sink.on_open_tag_end(self.index);
            if self.in_rcdata {
                self.state = State::InRCDATA;
            } else {
                self.state = State::Text;
            }
            self.section_start = self.index + 1;
        } else if self.try_start_in_tag_comment(c) {
        } else if c == SLASH {
            self.after_quoted_attr_value = false;
            self.state = State::InSelfClosingTag;
        } else if is_whitespace(c) {
            self.after_quoted_attr_value = false;
        } else if c == EQ {
            self.after_quoted_attr_value = false;
            self.sink.on_error(
                LexErrorCode::UnexpectedEqualsSignBeforeAttributeName,
                self.index,
            );
        } else if !is_whitespace(c) {
            if self.after_quoted_attr_value {
                self.sink
                    .on_error(LexErrorCode::MissingWhitespaceBetweenAttributes, self.index);
            }
            self.after_quoted_attr_value = false;
            self.handle_attr_start(c);
        }
    }

    pub(super) fn handle_attr_start(&mut self, c: u8) {
        if self.sink.mode() == LexMode::Verbatim {
            self.state = State::InAttrName;
            self.section_start = self.index;
            return;
        }
        if c == LOWER_V && self.input.get(self.index + 1) == Some(&DASH) {
            self.state = State::InDirName;
            self.section_start = self.index;
        } else if c == DOT || c == COLON || c == AT || c == NUMBER {
            self.sink.on_dir_name(self.index, self.index + 1);
            self.state = State::InDirArg;
            self.section_start = self.index + 1;
        } else {
            self.state = State::InAttrName;
            self.section_start = self.index;
        }
    }

    pub(super) fn state_in_attr_name(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            self.sink.on_attrib_name(self.section_start, self.index);
            self.sink.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == DOUBLE_QUOTE || c == SINGLE_QUOTE {
            self.sink
                .on_error(LexErrorCode::UnexpectedCharacterInAttributeName, self.index);
            self.sink.on_attrib_name(self.section_start, self.index);
            self.sink.on_attrib_name_end(self.index);
            self.section_start = self.index + 1;
            self.state = if c == DOUBLE_QUOTE {
                State::InAttrValueDq
            } else {
                State::InAttrValueSq
            };
        } else if c == LT {
            self.sink
                .on_error(LexErrorCode::UnexpectedCharacterInAttributeName, self.index);
        }
    }

    pub(super) fn state_in_dir_name(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            self.sink.on_dir_name(self.section_start, self.index);
            self.sink.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == COLON {
            self.sink.on_dir_name(self.section_start, self.index);
            self.state = State::InDirArg;
            self.section_start = self.index + 1;
        } else if c == DOT {
            self.sink.on_dir_name(self.section_start, self.index);
            self.state = State::InDirModifier;
            self.section_start = self.index + 1;
        } else if c == LEFT_SQUARE {
            self.sink.on_dir_name(self.section_start, self.index);
            self.state = State::InDirDynamicArg;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_in_dir_arg(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            if self.section_start < self.index {
                self.sink.on_dir_arg(self.section_start, self.index);
            }
            self.sink.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == LEFT_SQUARE {
            if self.section_start < self.index {
                self.sink.on_dir_arg(self.section_start, self.index);
            }
            self.state = State::InDirDynamicArg;
            self.section_start = self.index + 1;
        } else if c == DOT {
            if self.section_start < self.index {
                self.sink.on_dir_arg(self.section_start, self.index);
            }
            self.state = State::InDirModifier;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_in_dir_modifier(&mut self, c: u8) {
        if c == EQ || is_end_of_tag_section(c) {
            self.sink.on_dir_modifier(self.section_start, self.index);
            self.sink.on_attrib_name_end(self.index);
            self.section_start = self.index;
            self.state = State::AfterAttrName;
            self.state_after_attr_name(c);
        } else if c == DOT {
            self.sink.on_dir_modifier(self.section_start, self.index);
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn state_after_attr_name(&mut self, c: u8) {
        if c == EQ {
            self.state = State::BeforeAttrValue;
        } else if c == SLASH || c == GT {
            self.sink.on_attrib_end(QuoteType::NoValue, self.index);
            self.state = State::BeforeAttrName;
            self.state_before_attr_name(c);
        } else if !is_whitespace(c) {
            self.sink.on_attrib_end(QuoteType::NoValue, self.index);
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
            self.sink.on_error(
                LexErrorCode::UnexpectedEqualsSignBeforeAttributeName,
                self.index,
            );
        } else if c == GT {
            self.sink
                .on_error(LexErrorCode::MissingAttributeValue, self.index);
            self.sink.on_attrib_end(QuoteType::Unquoted, self.index);
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
        } else if c == AMP {
            self.start_entity();
        } else if matches!(c, DOUBLE_QUOTE | SINGLE_QUOTE | LT | EQ | GRAVE_ACCENT) {
            self.sink.on_error(
                LexErrorCode::UnexpectedCharacterInUnquotedAttributeValue,
                self.index,
            );
        }
        // Per HTML spec, only whitespace and `>` terminate an unquoted
        // attribute value. `/` is an ordinary value character here; the
        // self-closing `/` is only recognized in the *before/after attribute*
        // states. See https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    }

    pub(super) fn emit_attr_value(&mut self, quote: QuoteType) {
        if self.section_start < self.index {
            self.sink.on_attrib_data(self.section_start, self.index);
        }
        self.sink.on_attrib_end(quote, self.index);
        self.section_start = self.index + 1;
        self.state = State::BeforeAttrName;
        self.after_quoted_attr_value = matches!(quote, QuoteType::Double | QuoteType::Single);
    }
}
