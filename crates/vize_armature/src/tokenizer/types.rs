use vize_relief::ErrorCode;

use super::char_codes::{
    CARRIAGE_RETURN, FORM_FEED, GT, LOWER_A, LOWER_Z, NEWLINE, SLASH, SPACE, TAB, UPPER_A, UPPER_Z,
};

/// All the states the tokenizer can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Text = 1,

    // Interpolation
    InterpolationOpen,
    Interpolation,
    InterpolationClose,

    // Tags
    BeforeTagName,
    InTagName,
    InSelfClosingTag,
    BeforeClosingTagName,
    InClosingTagName,
    AfterClosingTagName,

    // Attributes
    BeforeAttrName,
    InAttrName,
    InDirName,
    InDirArg,
    InDirDynamicArg,
    InDirModifier,
    AfterAttrName,
    BeforeAttrValue,
    InAttrValueDq,
    InAttrValueSq,
    InAttrValueNq,

    // Declarations
    BeforeDeclaration,
    InDeclaration,

    // Processing instructions
    InProcessingInstruction,

    // Comments & CDATA
    BeforeComment,
    CDATASequence,
    InSpecialComment,
    InCommentLike,

    // Special tags
    BeforeSpecialS,
    BeforeSpecialT,
    SpecialStartSequence,
    InRCDATA,

    InEntity,

    InSFCRootTagName,
}

/// Quote type for attribute values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuoteType {
    NoValue = 0,
    Unquoted = 1,
    Single = 2,
    Double = 3,
}

/// Tokenizer callbacks
pub trait Callbacks {
    fn on_text(&mut self, start: usize, end: usize);
    fn on_text_entity(&mut self, char: char, start: usize, end: usize);

    fn on_interpolation(&mut self, start: usize, end: usize);

    fn on_open_tag_name(&mut self, start: usize, end: usize);
    fn on_open_tag_end(&mut self, end: usize);
    fn on_self_closing_tag(&mut self, end: usize);
    fn on_close_tag(&mut self, start: usize, end: usize);

    fn on_attrib_data(&mut self, start: usize, end: usize);
    fn on_attrib_entity(&mut self, char: char, start: usize, end: usize);
    fn on_attrib_end(&mut self, quote: QuoteType, end: usize);
    fn on_attrib_name(&mut self, start: usize, end: usize);
    fn on_attrib_name_end(&mut self, end: usize);

    fn on_dir_name(&mut self, start: usize, end: usize);
    fn on_dir_arg(&mut self, start: usize, end: usize);
    fn on_dir_modifier(&mut self, start: usize, end: usize);

    fn on_comment(&mut self, start: usize, end: usize);
    fn on_cdata(&mut self, start: usize, end: usize);
    fn on_processing_instruction(&mut self, start: usize, end: usize);

    fn on_end(&mut self);
    fn on_error(&mut self, code: ErrorCode, index: usize);

    /// Check if the parser is currently inside a v-pre block.
    /// When true, the tokenizer skips directive parsing and treats all
    /// attributes as regular attributes, and skips interpolation detection.
    fn is_in_v_pre(&self) -> bool {
        false
    }
}

/// Check if character is a tag start character (a-z, A-Z)
#[inline]
pub fn is_tag_start_char(c: u8) -> bool {
    (LOWER_A..=LOWER_Z).contains(&c) || (UPPER_A..=UPPER_Z).contains(&c)
}

/// Check if character is whitespace
#[inline]
pub fn is_whitespace(c: u8) -> bool {
    c == SPACE || c == NEWLINE || c == TAB || c == FORM_FEED || c == CARRIAGE_RETURN
}

/// Check if character ends a tag section
#[inline]
pub fn is_end_of_tag_section(c: u8) -> bool {
    c == SLASH || c == GT || is_whitespace(c)
}
