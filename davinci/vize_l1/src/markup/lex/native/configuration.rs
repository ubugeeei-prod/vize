//! State-preserving configuration used by the published tokenizer facade.

use super::Lexer;
use crate::markup::{Profile, Sink};
use core::marker::PhantomData;

impl<'a, P: Profile, S: Sink> Lexer<'a, P, S> {
    /// Changing the profile moves every state field unchanged. The exhaustive
    /// destructuring makes a future field require an explicit preservation.
    #[inline]
    pub(crate) fn into_profile<Q: Profile>(self) -> Lexer<'a, Q, S> {
        let Self {
            input,
            state,
            base_state,
            section_start,
            index,
            sink,
            delimiter_open,
            delimiter_close,
            delimiter_index,
            entity_start,
            current_sequence,
            sequence_index,
            in_rcdata,
            after_quoted_attr_value,
            triple_mustache,
            in_raw_interpolation,
            in_tag_comments,
            profile: _,
        } = self;
        Lexer {
            input,
            state,
            base_state,
            section_start,
            index,
            sink,
            delimiter_open,
            delimiter_close,
            delimiter_index,
            entity_start,
            current_sequence,
            sequence_index,
            in_rcdata,
            after_quoted_attr_value,
            triple_mustache,
            in_raw_interpolation,
            in_tag_comments,
            profile: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn set_raw_interpolation(&mut self, enabled: bool) {
        self.triple_mustache = enabled && self.delimiter_open == b"{{";
    }

    #[inline]
    pub(crate) fn set_in_tag_comments(&mut self, enabled: bool) {
        self.in_tag_comments = enabled;
    }
}
