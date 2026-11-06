//! Explicit head RCDATA/RAWTEXT ancestry from the existing owned lexer tape.
//!
//! Head title/style/script text never enters body insertion rules. Slashed starts refuse:
//! the current lexer ends its special-text state on that slash, whereas an
//! HTML parser ignores the slash and continues reading the element's text.

use super::{Builder, Mode, Refusal, Tag};
use crate::markup::Profile;

impl<P: Profile> Builder<'_, '_, P> {
    pub(super) fn head_text_start(&mut self, tag: Tag) -> Result<(), Refusal> {
        if self.mode != Mode::InHead || self.current() != Some("head") || tag.slash {
            return Err(Refusal::UnsupportedElement(tag.span));
        }
        self.insert(tag)
    }

    pub(super) fn in_head_text(&self) -> bool {
        matches!(self.current(), Some("title" | "style" | "script"))
    }
}

#[cfg(test)]
mod tests;
