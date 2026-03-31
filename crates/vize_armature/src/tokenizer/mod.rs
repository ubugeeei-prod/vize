//! HTML tokenizer for Vue templates.
//!
//! This tokenizer is adapted from htmlparser2 and Vue's compiler-core.
//! It uses a state machine to tokenize HTML/Vue templates.

pub mod char_codes;
mod entity_decode;
mod states;
mod types;

pub use types::*;

use char_codes::NEWLINE;
use vize_relief::Position;

/// HTML tokenizer
pub struct Tokenizer<'a, C: Callbacks> {
    /// Input source
    input: &'a [u8],
    /// Current state
    state: State,
    /// Some behavior, eg. when decoding entities, is done while we are in another state. This keeps track of the other state type.
    base_state: State,
    /// Buffer start position
    section_start: usize,
    /// Current index
    index: usize,
    /// Newline positions for line/column calculation
    newlines: Vec<usize>,
    /// Callbacks
    callbacks: C,
    /// Delimiter open sequence
    delimiter_open: &'a [u8],
    /// Delimiter close sequence
    delimiter_close: &'a [u8],
    /// Current delimiter index
    delimiter_index: usize,
    /// In pre tag
    #[allow(dead_code)]
    in_pre: bool,
    /// The start of the last entity.
    entity_start: usize,
}

impl<'a, C: Callbacks> Tokenizer<'a, C> {
    /// Create a new tokenizer
    pub fn new(input: &'a str, callbacks: C) -> Self {
        Self::with_delimiters(input, callbacks, b"{{", b"}}")
    }

    /// Create a new tokenizer with custom delimiters
    pub fn with_delimiters(
        input: &'a str,
        callbacks: C,
        delimiter_open: &'a [u8],
        delimiter_close: &'a [u8],
    ) -> Self {
        Self {
            input: input.as_bytes(),
            state: State::Text,
            section_start: 0,
            index: 0,
            newlines: Vec::new(),
            callbacks,
            delimiter_open,
            delimiter_close,
            delimiter_index: 0,
            in_pre: false,
            entity_start: 0,
            base_state: State::Text,
        }
    }

    /// Get the position for a given index
    pub fn get_pos(&self, index: usize) -> Position {
        // Binary search for line number
        let line = match self.newlines.binary_search(&index) {
            Ok(i) => i + 1,
            Err(i) => i + 1,
        };

        let column = if line == 1 {
            index + 1
        } else {
            index - self.newlines[line - 2]
        };

        Position {
            offset: index as u32,
            line: line as u32,
            column: column as u32,
        }
    }

    /// Tokenize the input
    pub fn tokenize(&mut self) {
        while self.index < self.input.len() {
            let c = self.input[self.index];

            // Track newlines
            if c == NEWLINE {
                self.newlines.push(self.index);
            }

            match self.state {
                State::Text => self.state_text(c),
                State::InterpolationOpen => self.state_interpolation_open(c),
                State::Interpolation => self.state_interpolation(c),
                State::InterpolationClose => self.state_interpolation_close(c),
                State::BeforeTagName => self.state_before_tag_name(c),
                State::InTagName => self.state_in_tag_name(c),
                State::InSelfClosingTag => self.state_in_self_closing_tag(c),
                State::BeforeClosingTagName => self.state_before_closing_tag_name(c),
                State::InClosingTagName => self.state_in_closing_tag_name(c),
                State::AfterClosingTagName => self.state_after_closing_tag_name(c),
                State::BeforeAttrName => self.state_before_attr_name(c),
                State::InAttrName => self.state_in_attr_name(c),
                State::InDirName => self.state_in_dir_name(c),
                State::InDirArg => self.state_in_dir_arg(c),
                State::InDirDynamicArg => self.state_in_dir_dynamic_arg(c),
                State::InDirModifier => self.state_in_dir_modifier(c),
                State::AfterAttrName => self.state_after_attr_name(c),
                State::BeforeAttrValue => self.state_before_attr_value(c),
                State::InAttrValueDq => self.state_in_attr_value_dq(c),
                State::InAttrValueSq => self.state_in_attr_value_sq(c),
                State::InAttrValueNq => self.state_in_attr_value_nq(c),
                State::BeforeDeclaration => self.state_before_declaration(c),
                State::InDeclaration => self.state_in_declaration(c),
                State::InProcessingInstruction => self.state_in_processing_instruction(c),
                State::BeforeComment => self.state_before_comment(c),
                State::CDATASequence => self.state_cdata_sequence(c),
                State::InSpecialComment => self.state_in_special_comment(c),
                State::InCommentLike => self.state_in_comment_like(c),
                State::BeforeSpecialS => self.state_before_special_s(c),
                State::BeforeSpecialT => self.state_before_special_t(c),
                State::SpecialStartSequence => self.state_special_start_sequence(c),
                State::InRCDATA => self.state_in_rcdata(c),
                State::InEntity => self.state_in_entity(),
                State::InSFCRootTagName => self.state_in_sfc_root_tag_name(c),
            }

            self.index += 1;
        }

        // Handle remaining content
        self.cleanup();
        self.callbacks.on_end();
    }
}

#[cfg(test)]
mod tests;
