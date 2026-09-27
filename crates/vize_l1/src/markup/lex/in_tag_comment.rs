use super::{
    Tokenizer,
    char_codes::{CARRIAGE_RETURN, NEWLINE, SLASH},
    types::{Callbacks, State},
};

impl<'a, C: Callbacks> Tokenizer<'a, C> {
    pub(super) fn try_start_in_tag_comment(&mut self, c: u8) -> bool {
        if c != SLASH || !self.in_tag_comments || self.input.get(self.index + 1) != Some(&SLASH) {
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
            self.callbacks
                .on_in_tag_comment(self.section_start, self.index);
            self.state = State::BeforeAttrName;
            self.section_start = self.index + 1;
        }
    }

    pub(super) fn recover_in_tag_comment_at_eof(&mut self, inferred_tag_end: usize) {
        self.callbacks
            .on_in_tag_comment(self.section_start, self.index);
        self.callbacks.on_open_tag_end(inferred_tag_end);
    }
}
