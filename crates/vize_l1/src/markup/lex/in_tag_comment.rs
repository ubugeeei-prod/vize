use super::{
    Lexer,
    char_codes::{CARRIAGE_RETURN, NEWLINE, SLASH},
    types::State,
};
use crate::markup::profile::Profile;
use crate::markup::token::Sink;

impl<P: Profile, S: Sink> Lexer<'_, P, S> {
    pub(super) fn try_start_in_tag_comment(&mut self) -> bool {
        if !self.in_tag_comments || self.input.get(self.index + 1) != Some(&SLASH) {
            return false;
        }
        self.after_quoted_attr_value = false;
        self.state = State::InTagComment;
        self.section_start = self.index;
        self.index += 1;
        true
    }

    pub(super) fn state_in_tag_comment(&mut self, c: u8) {
        if c == NEWLINE || c == CARRIAGE_RETURN {
            self.sink.on_in_tag_comment(self.section_start, self.index);
            self.state = State::BeforeAttrName;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn recover_in_tag_comment_at_eof(&mut self, inferred_tag_end: usize) {
        self.sink.on_in_tag_comment(self.section_start, self.index);
        self.sink.on_open_tag_end(inferred_tag_end);
    }
}
