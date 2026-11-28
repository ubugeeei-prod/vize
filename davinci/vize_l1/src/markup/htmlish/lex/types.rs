use super::char_codes::{
    BACKSLASH, CARRIAGE_RETURN, COLON, DOT, DOUBLE_QUOTE, EQ, FORM_FEED, GT, LEFT_SQUARE, LOWER_A,
    LOWER_Z, LT, NEWLINE, SINGLE_QUOTE, SLASH, SPACE, TAB, UPPER_A, UPPER_Z,
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

/// Bytes an attribute or directive name state looks at: whitespace, `/`,
/// `>`, `=`, the quotes, `<`, `:`, `.`, `[` and the backslash the lexer
/// records, all at or below the backslash. Every other byte continues a name
/// in every name state.
#[expect(
    clippy::indexing_slicing,
    reason = "evaluated at compile time: an index past the table fails the build"
)]
const NAME_STOPS: [bool; BACKSLASH as usize + 1] = {
    let mut table = [false; BACKSLASH as usize + 1];
    let mut index = 0usize;
    while index < table.len() {
        table[index] = matches!(
            index as u8,
            SPACE
                | NEWLINE
                | TAB
                | FORM_FEED
                | CARRIAGE_RETURN
                | SLASH
                | GT
                | EQ
                | DOUBLE_QUOTE
                | SINGLE_QUOTE
                | LT
                | COLON
                | DOT
                | LEFT_SQUARE
                | BACKSLASH
        );
        index += 1;
    }
    table
};

/// Whether `c` may end, split or mark a name; see [`NAME_STOPS`]. A
/// lowercase letter, the commonest name byte, is settled by the first
/// comparison and every other byte by one lookup.
#[inline(always)]
pub(super) fn may_stop_name(c: u8) -> bool {
    c <= BACKSLASH && NAME_STOPS.get(usize::from(c)).copied().unwrap_or(false)
}
