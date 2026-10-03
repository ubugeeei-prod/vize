use super::{
    Lexer,
    char_codes::{AMP, LT},
    sequences::Sequence,
    types::State,
};
use crate::markup::profile::Profile;
use crate::markup::token::{LexErrorCode, LexMode, QuoteType, Sink};

impl<P: Profile, S: Sink> Lexer<'_, P, S> {
    pub(super) fn cleanup(&mut self) {
        let has_section = self.section_start < self.index;

        match self.state {
            State::BeforeDeclaration
            | State::InDeclaration
            | State::BeforeComment
            | State::CDATASequence => {
                self.finish_declaration(self.index, false);
            }
            State::Text if has_section => {
                self.sink.on_text(self.section_start, self.index);
            }
            State::InRCDATA if has_section => {
                self.sink.on_text(self.section_start, self.index);
            }
            State::Interpolation | State::InterpolationClose if has_section => {
                self.sink
                    .on_error(LexErrorCode::MissingInterpolationEnd, self.index);
                let start = self.section_start.saturating_sub(self.delimiter_open.len());
                self.sink.on_text(start, self.index);
            }
            State::BeforeTagName if has_section => {
                self.sink
                    .on_error(LexErrorCode::EofBeforeTagName, self.index);
                self.sink.on_text(self.section_start, self.index);
            }
            State::InTagName
            | State::BeforeSpecialS
            | State::BeforeSpecialT
            | State::SpecialStartSequence
            | State::BeforeClosingTagName
            | State::InClosingTagName
            | State::AfterClosingTagName
            | State::BeforeAttrName
            | State::InTagComment
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
                self.sink.on_error(LexErrorCode::EofInTag, self.index);
                self.recover_incomplete_tag_at_eof();
            }
            _ => {}
        }

        if self.state == State::InCommentLike {
            let code = match self.current_sequence {
                Some(Sequence::CdataEnd) => LexErrorCode::EofInCdata,
                _ => LexErrorCode::EofInComment,
            };
            let error_index = match self.current_sequence {
                Some(Sequence::CdataEnd) => self.section_start.saturating_sub(9),
                _ => self.section_start.saturating_sub(4),
            };
            self.sink.on_error(code, error_index);
            match self.current_sequence {
                Some(Sequence::CdataEnd) => {
                    self.sink.on_cdata(self.section_start, self.index);
                }
                _ => {
                    self.sink.on_comment(self.section_start, self.index);
                }
            }
        }
    }

    fn recover_incomplete_tag_at_eof(&mut self) {
        let inferred_tag_end = self.index.saturating_sub(1);

        match self.state {
            State::InTagName
            | State::BeforeSpecialS
            | State::BeforeSpecialT
            | State::SpecialStartSequence => {
                self.sink.on_open_tag_name(self.section_start, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::BeforeAttrName => {
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InTagComment => {
                self.recover_in_tag_comment_at_eof(inferred_tag_end);
            }
            State::InAttrName => {
                self.sink.on_attrib_name(self.section_start, self.index);
                self.sink.on_attrib_name_end(self.index);
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InDirName => {
                self.sink.on_dir_name(self.section_start, self.index);
                self.sink.on_attrib_name_end(self.index);
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InDirArg => {
                if self.section_start < self.index {
                    self.sink.on_dir_arg(self.section_start, self.index);
                }
                self.sink.on_attrib_name_end(self.index);
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InDirDynamicArg => {
                self.sink
                    .on_error(LexErrorCode::MissingDynamicDirectiveArgumentEnd, self.index);
                if self.section_start < self.index {
                    self.sink.on_dir_arg(self.section_start, self.index);
                }
                self.sink.on_attrib_name_end(self.index);
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InDirModifier => {
                self.sink.on_dir_modifier(self.section_start, self.index);
                self.sink.on_attrib_name_end(self.index);
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::AfterAttrName => {
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::BeforeAttrValue => {
                self.sink
                    .on_error(LexErrorCode::MissingAttributeValue, self.index);
                self.sink.on_attrib_end(QuoteType::NoValue, self.index);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrValueDq => {
                self.emit_unclosed_attr_value(QuoteType::Double);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrValueSq => {
                self.emit_unclosed_attr_value(QuoteType::Single);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InAttrValueNq => {
                self.emit_unclosed_attr_value(QuoteType::Unquoted);
                self.sink.on_open_tag_end(inferred_tag_end);
            }
            State::InClosingTagName if self.section_start < self.index => {
                self.sink.on_close_tag(self.section_start, self.index);
            }
            State::InClosingTagName | State::BeforeClosingTagName | State::AfterClosingTagName => {}
            _ => {}
        }

        self.state = State::Text;
        self.section_start = self.index;
    }

    fn emit_unclosed_attr_value(&mut self, quote: QuoteType) {
        if self.section_start < self.index {
            self.sink.on_attrib_data(self.section_start, self.index);
        }
        self.sink.on_attrib_end(quote, self.index);
    }

    pub(super) fn state_text(&mut self, c: u8) {
        if c == LT {
            if self.index > self.section_start {
                self.sink.on_text(self.section_start, self.index);
            }
            self.state = State::BeforeTagName;
            self.section_start = self.index;
        } else if c == AMP {
            self.start_entity();
        } else if self.sink.mode() == LexMode::Normal && self.at_opening_delimiter(c) {
            self.state = State::InterpolationOpen;
            self.delimiter_index = 0;
            self.state_interpolation_open(c);
        }
    }

    pub(super) fn state_interpolation_open(&mut self, c: u8) {
        if self.delimiter_open.get(self.delimiter_index) == Some(&c) {
            self.delimiter_index += 1;
            if self.delimiter_index == self.delimiter_open.len() {
                let start = self.index + 1 - self.delimiter_open.len();
                if start > self.section_start {
                    self.sink.on_text(self.section_start, start);
                }
                self.section_start = self.index + 1;
                self.state = State::Interpolation;
                self.delimiter_index = 0;
                // Vue 1.x triple-mustache (`{{{ expr }}}`): if a third `{`
                // immediately follows the opened `{{`, treat this as a raw-HTML
                // interpolation and drop the extra brace from the expression
                // span. Gated on `triple_mustache`, so the default Vue 3 path is
                // byte-identical (it falls through with the brace kept in the
                // expression, exactly as today).
                if self.triple_mustache && self.input.get(self.index + 1) == Some(&b'{') {
                    self.in_raw_interpolation = true;
                    self.section_start = self.index + 2;
                }
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
        let Some(&closing_byte) = self.delimiter_close.first() else {
            return;
        };
        // Expression bytes emit no callbacks or mode polls. Reuse the bounded
        // byte loop rather than dispatching their unchanged state per byte.
        if c != closing_byte && !self.fast_forward_to(closing_byte) {
            return;
        }
        self.state = State::InterpolationClose;
        self.delimiter_index = 0;
        self.state_interpolation_close(closing_byte);
    }

    pub(super) fn state_interpolation_close(&mut self, c: u8) {
        if self.delimiter_close.get(self.delimiter_index) == Some(&c) {
            self.delimiter_index += 1;
            if self.delimiter_index == self.delimiter_close.len() {
                let expr_end = self.index + 1 - self.delimiter_close.len();
                if self.in_raw_interpolation {
                    // Vue 1.x `{{{ expr }}}`: consume a trailing third `}` (if
                    // present) so the raw closing brace is not left as text, then
                    // emit a raw-HTML interpolation. This branch only runs behind
                    // `triple_mustache`, so the default path is untouched.
                    self.in_raw_interpolation = false;
                    self.sink.on_raw_interpolation(self.section_start, expr_end);
                    let consumed_brace = self.input.get(self.index + 1) == Some(&b'}');
                    self.section_start = self.index + 1 + usize::from(consumed_brace);
                    if consumed_brace {
                        self.index += 1;
                    }
                } else {
                    self.sink.on_interpolation(self.section_start, expr_end);
                    self.section_start = self.index + 1;
                }
                if self.in_rcdata {
                    self.state = State::InRCDATA
                } else {
                    self.state = State::Text
                }
            }
        } else {
            self.state = State::Interpolation;
            self.state_interpolation(c);
        }
    }
}
