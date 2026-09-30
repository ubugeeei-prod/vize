//! Lexer vocabulary: the event sink, quote kinds, namespaces and lex errors.
//!
//! The lexer pushes events into a [`Sink`] (statically dispatched) instead of
//! materializing tokens, so lexing allocates nothing per token; a consumer
//! that wants a token stream records one itself (as the surface tree does).
//! All offsets are byte offsets into the lexed source.

use super::entity::DecodedEntity;

/// How an attribute value was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuoteType {
    NoValue = 0,
    Unquoted = 1,
    Single = 2,
    Double = 3,
}

/// Markup namespace an element is read in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Namespace {
    #[default]
    Html = 0,
    Svg = 1,
    MathMl = 2,
}

/// Lexing mode of the current content, switched by the tree builder.
///
/// `Verbatim` is `v-pre`: children are read without mustache or directive
/// interpretation. It is a question of what the text *is*, so it lives in L1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LexMode {
    #[default]
    Normal,
    Verbatim,
}

/// A recoverable lexical error. Names follow the WHATWG/Vue parse errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexErrorCode {
    AbruptClosingOfEmptyComment,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingDynamicDirectiveArgumentEnd,
    MissingEndTagName,
    MissingInterpolationEnd,
    MissingWhitespaceBetweenAttributes,
    NestedComment,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,
}

/// Receiver of lexer events. `start..end` ranges are half-open byte ranges.
pub trait Sink {
    fn on_text(&mut self, start: usize, end: usize);
    /// A decoded character reference inside text (`&amp;` → `&`).
    fn on_text_entity(&mut self, ch: char, start: usize, end: usize);
    /// The complete decoded value of one reference. `start..end` covers the
    /// authored reference, including its `&`; the decoded value may contain
    /// multiple scalars. Legacy scalar sinks receive each scalar by default.
    fn on_text_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        value.for_each(|ch| self.on_text_entity(ch, start, end));
    }

    fn on_interpolation(&mut self, start: usize, end: usize);
    /// A Vue 1.x raw-HTML interpolation, `{{{ expr }}}`. Only emitted when
    /// [`LexOptions::raw_interpolation`](super::LexOptions) is set.
    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        self.on_interpolation(start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize);
    fn on_open_tag_end(&mut self, end: usize);
    fn on_self_closing_tag(&mut self, end: usize);
    fn on_close_tag(&mut self, start: usize, end: usize);

    fn on_attrib_data(&mut self, start: usize, end: usize);
    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize);
    /// The complete decoded attribute reference, with its authored byte span.
    fn on_attrib_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        value.for_each(|ch| self.on_attrib_entity(ch, start, end));
    }
    fn on_attrib_end(&mut self, quote: QuoteType, end: usize);
    fn on_attrib_name(&mut self, start: usize, end: usize);
    fn on_attrib_name_end(&mut self, end: usize);

    fn on_dir_name(&mut self, start: usize, end: usize);
    fn on_dir_arg(&mut self, start: usize, end: usize);
    fn on_dir_modifier(&mut self, start: usize, end: usize);

    fn on_comment(&mut self, start: usize, end: usize);
    /// An experimental `//` comment inside an open tag's attribute list.
    fn on_in_tag_comment(&mut self, _start: usize, _end: usize) {}
    fn on_cdata(&mut self, start: usize, end: usize);
    fn on_processing_instruction(&mut self, start: usize, end: usize);

    fn on_end(&mut self);
    fn on_error(&mut self, code: LexErrorCode, index: usize);

    /// The mode for the content being lexed; the tree builder answers
    /// [`LexMode::Verbatim`] inside a `v-pre` element.
    fn mode(&self) -> LexMode {
        LexMode::Normal
    }
}
