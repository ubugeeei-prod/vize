use vize_l0::ErrorCode;

use crate::markup::entity::DecodedEntity;
pub use crate::markup::token::QuoteType;

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
    InSpecialComment,
    InCommentLike,

    // Special tags
    BeforeSpecialS,
    BeforeSpecialT,
    SpecialStartSequence,
    InRCDATA,

    InEntity,
}

/// Tokenizer callbacks
pub trait Callbacks {
    fn on_text(&mut self, start: usize, end: usize);
    fn on_text_entity(&mut self, char: char, start: usize, end: usize);

    fn on_interpolation(&mut self, start: usize, end: usize);

    /// A Vue 1.x raw-HTML (unescaped) interpolation, `{{{ expr }}}`.
    ///
    /// Only ever called when the tokenizer was built with triple-mustache
    /// recognition enabled (the `legacy` feature + a Vue 1.x dialect); the
    /// default Vue 3 path never emits it. The default implementation forwards to
    /// [`Callbacks::on_interpolation`] so callers that do not model raw HTML get
    /// the ordinary interpolation, but a `legacy`-aware parser overrides this to
    /// flag the node as raw (the pre-Vue-2 `v-html` equivalent).
    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        self.on_interpolation(start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize);

    /// Resolve SVG `title` content from the owner's existing parent scope.
    /// Queried after `on_open_tag_name`; stateless callbacks retain RCDATA.
    fn svg_title_children(&self) -> bool {
        false
    }

    fn on_open_tag_end(&mut self, end: usize);
    fn on_self_closing_tag(&mut self, end: usize);
    fn on_close_tag(&mut self, start: usize, end: usize);

    fn on_attrib_data(&mut self, start: usize, end: usize);
    fn on_attrib_entity(&mut self, char: char, start: usize, end: usize);

    /// Receive the already-decoded attribute value with its authored span.
    ///
    /// Existing callback implementations keep the original first-scalar event.
    /// A consumer that owns complete attribute values may explicitly override
    /// this provided method. Text entities retain their separate old contract.
    fn on_attrib_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        if let Some(first) = super::adapter::first_scalar(value) {
            self.on_attrib_entity(first, start, end);
        }
    }

    /// An authored equals sign reached EOF without an attribute value.
    /// The default preserves the published callback stream; a parser may opt
    /// into explicit empty-assignment semantics without changing other owners.
    fn on_missing_attribute_value_at_eof(&mut self, end: usize) {
        self.on_attrib_end(QuoteType::NoValue, end);
    }

    fn on_attrib_end(&mut self, quote: QuoteType, end: usize);
    fn on_attrib_name(&mut self, start: usize, end: usize);
    fn on_attrib_name_end(&mut self, end: usize);

    fn on_dir_name(&mut self, start: usize, end: usize);
    fn on_dir_arg(&mut self, start: usize, end: usize);
    fn on_dir_modifier(&mut self, start: usize, end: usize);

    fn on_comment(&mut self, start: usize, end: usize);
    fn on_in_tag_comment(&mut self, _start: usize, _end: usize) {}
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

    /// Opt in to directive-head framing while `is_in_v_pre` remains true.
    /// This changes lexical callbacks only; attribute semantics belong to the
    /// owner. The published facade's original stream remains the default.
    fn directive_heads_in_verbatim(&self) -> bool {
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
