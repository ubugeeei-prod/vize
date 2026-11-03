//! The shared markup lexer, `Lexer<P: Profile, S: Sink>`.
//!
//! One state machine serves every profile; profile rules are associated
//! constants, so the per-profile branches fold at compile time. The lexer
//! makes one pass over the bytes and allocates nothing per token.
//!
//! The state machine is adapted from htmlparser2 and Vue's compiler-core.

pub mod char_codes;
mod configuration;
mod dynamic_arg;
mod in_tag_comment;
mod sequences;
mod states_special;
mod states_tag;
mod states_text;
mod types;

use core::marker::PhantomData;

use super::super::profile::Profile;
use super::super::token::Sink;

use sequences::Sequence;
pub(crate) use types::State;
pub use types::{is_end_of_tag_section, is_tag_start_char, is_whitespace};

/// Find the closing `]` of a dynamic directive argument, using the lexer's
/// quote, escape, nested-delimiter, and HTML attribute recovery rules.
///
/// `text` starts immediately after the authored opening `[`. The returned byte
/// offset points at the closing `]`, an HTML boundary, or the end of `text`;
/// the boolean is true only when the argument's own closing `]` was found.
pub fn dynamic_argument_boundary(text: &str) -> (usize, bool) {
    dynamic_arg::scan_argument(text.as_bytes(), 0)
}

/// Interpolation delimiters, `{{`/`}}` by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delimiters<'a> {
    pub open: &'a [u8],
    pub close: &'a [u8],
}

impl Default for Delimiters<'_> {
    fn default() -> Self {
        Self {
            open: b"{{",
            close: b"}}",
        }
    }
}

/// Per-file lexer switches. Dialect-dependent switches are resolved once per
/// file by the dialect module and passed in as plain values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LexOptions<'a> {
    pub delimiters: Delimiters<'a>,
    /// Vue 1.x `{{{ expr }}}` raw-HTML interpolation. Admission checks only
    /// the default opening delimiter, preserving custom-close compatibility.
    pub raw_interpolation: bool,
    /// Experimental `//` comments inside open tags.
    pub in_tag_comments: bool,
}

/// The markup lexer over one source text.
pub struct Lexer<'a, P: Profile, S: Sink> {
    input: &'a [u8],
    state: State,
    /// The state to return to after an entity.
    base_state: State,
    section_start: usize,
    /// Original `<!` offset; recovery can advance `section_start` separately.
    declaration_start: usize,
    /// The preserved driver skipped an earlier declaration closing byte.
    declaration_recovered: bool,
    index: usize,
    sink: S,
    delimiter_open: &'a [u8],
    delimiter_close: &'a [u8],
    delimiter_index: usize,
    /// The start of the last entity.
    entity_start: usize,
    /// Closing sequence currently being matched.
    current_sequence: Option<Sequence>,
    /// Index of the next expected byte in that sequence.
    sequence_index: usize,
    /// Inside `<script>`, `<style>`, `<title>` or `<textarea>` content.
    in_rcdata: bool,
    /// True immediately after a quoted attribute value ended, so `<div a="b"c>`
    /// reports missing whitespace instead of silently starting `c`.
    after_quoted_attr_value: bool,
    /// Vue 1.x `{{{ … }}}`; admitted by the default opening delimiter.
    triple_mustache: bool,
    /// True while the open interpolation is a `{{{ … }}}` one.
    in_raw_interpolation: bool,
    in_tag_comments: bool,
    profile: PhantomData<P>,
}

impl<'a, P: Profile, S: Sink> Lexer<'a, P, S> {
    pub fn new(source: &'a str, sink: S, options: LexOptions<'a>) -> Self {
        let LexOptions {
            delimiters,
            raw_interpolation,
            in_tag_comments,
        } = options;
        Self {
            input: source.as_bytes(),
            state: State::Text,
            base_state: State::Text,
            section_start: 0,
            declaration_start: 0,
            declaration_recovered: false,
            index: 0,
            sink,
            delimiter_open: delimiters.open,
            delimiter_close: delimiters.close,
            delimiter_index: 0,
            entity_start: 0,
            current_sequence: None,
            sequence_index: 0,
            in_rcdata: false,
            after_quoted_attr_value: false,
            triple_mustache: raw_interpolation && delimiters.open == b"{{",
            in_raw_interpolation: false,
            in_tag_comments,
            profile: PhantomData,
        }
    }

    /// The source bytes being lexed.
    pub fn input(&self) -> &'a [u8] {
        self.input
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn into_sink(self) -> S {
        self.sink
    }

    /// Intrinsic EOF state of this actual driver. Silent pending states do
    /// not certify a normal lexical run merely because `on_end` was reached.
    pub(crate) fn normal_end_state(&self) -> bool {
        matches!(self.state, State::Text | State::InRCDATA)
    }

    #[inline]
    fn at_opening_delimiter(&self, c: u8) -> bool {
        self.delimiter_open.first() == Some(&c)
    }

    /// Skip through the buffer until a target byte is found.
    fn fast_forward_to(&mut self, c: u8) -> bool {
        while self.index + 1 < self.input.len() {
            self.index += 1;
            if self.input.get(self.index) == Some(&c) {
                return true;
            }
        }
        self.index = self.input.len().saturating_sub(1);
        false
    }

    /// Lex the whole input, pushing every event and finally `on_end` into
    /// the sink.
    pub fn run(&mut self) {
        let input = self.input;
        while let Some(&c) = input.get(self.index) {
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
                State::InTagComment => self.state_in_tag_comment(c),
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
                State::InCommentLike => self.state_in_comment_like(c),
                State::BeforeSpecialS => self.state_before_special_s(c),
                State::BeforeSpecialT => self.state_before_special_t(c),
                State::SpecialStartSequence => self.state_special_start_sequence(c),
                State::InRCDATA => self.state_in_rcdata(c),
                State::InEntity => self.state_in_entity(),
            }

            self.index += 1;
        }

        // Handle remaining content
        self.cleanup();
        self.sink.on_end();
    }
}

#[cfg(test)]
mod empty_delimiter_tests;
#[cfg(test)]
mod tests;
