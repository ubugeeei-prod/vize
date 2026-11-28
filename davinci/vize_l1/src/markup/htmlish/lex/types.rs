use super::char_codes::{
    CARRIAGE_RETURN, FORM_FEED, GT, LOWER_A, LOWER_Z, NEWLINE, SLASH, SPACE, TAB, UPPER_A, UPPER_Z,
};

/// All the states the tokenizer can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum State {
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
    InTagComment,
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
    // Keep Text=1 and InRCDATA=33 so their hot transition can use a shift and
    // add. The unused historical state at 28 needs no runtime arm.
    InCommentLike = 29,

    // Special tags
    BeforeSpecialS,
    BeforeSpecialT,
    SpecialStartSequence,
    InRCDATA,

    InEntity,
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
