//! The tokenizer-event stream the tree is built from.
//!
//! Construction is two-phase: a [`Recorder`] implements native
//! [`Sink`](crate::markup::token::Sink)
//! and preserved [`Callbacks`], flattening events into an arena `Vec` of
//! plain [`Event`]s (12 bytes, `Copy`), then `build` walks that slice
//! with lookahead. The split keeps the tree builder a straight-line
//! function over data instead of a callback state machine.

mod native;
#[cfg(test)]
mod tests;

use crate::markup::lex::compat::{Callbacks, QuoteType as CompatQuote};
use crate::markup::token::QuoteType;
use vize_l0::ErrorCode;
use vize_l0::Vec;

use crate::parse::SurfaceError;

/// What a tokenizer callback reported. Directive name pieces
/// (`on_dir_name` / `on_dir_arg` / `on_dir_modifier`) all record as
/// [`EventKind::AttrName`]: the raw attribute name token is bounded by
/// the first name piece's start and the `AttrNameEnd` offset, so the
/// pieces need no identity of their own at L1 (the semantic split is
/// P2-8's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventKind {
    /// Raw text bytes (entity ranges included, undecoded).
    Text,
    /// Interpolation content between the delimiters.
    Interpolation,
    /// An open tag's name (after `<`).
    OpenTagName,
    /// The open tag's `>` (`start` is its index; at EOF recovery the
    /// index is inferred and points at a non-`>` byte).
    OpenTagEnd,
    /// The `>` of a self-closing tag (`start` is its index).
    SelfClosingTag,
    /// An end tag's name (after `</`).
    CloseTag,
    /// An attribute (or directive) name piece.
    AttrName,
    /// The end offset of the whole raw attribute name.
    AttrNameEnd,
    /// Attribute value bytes (entity ranges included, undecoded).
    AttrData,
    /// End of an attribute; `aux` is the [`QuoteType`] as `u8`.
    AttrEnd,
    /// Comment content (normal `<!-- … -->` and recovered bogus forms).
    Comment,
    /// CDATA content.
    Cdata,
    /// Processing-instruction content.
    ProcessingInstruction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Event {
    pub kind: EventKind,
    /// Quote type on AttrEnd; resolved verbatim mode on opening-tag ends.
    /// Raw interpolation delimiter width on Interpolation; otherwise zero.
    /// These interpretations never overlap.
    pub aux: u8,
    pub start: u32,
    pub end: u32,
}

impl Event {
    fn new(kind: EventKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            aux: 0,
            start: start as u32,
            end: end as u32,
        }
    }

    pub(crate) fn quote(&self) -> QuoteType {
        debug_assert!(self.kind == EventKind::AttrEnd);
        match self.aux {
            1 => QuoteType::Unquoted,
            2 => QuoteType::Single,
            3 => QuoteType::Double,
            _ => QuoteType::NoValue,
        }
    }

    pub(crate) fn is_verbatim_opening(&self) -> bool {
        debug_assert!(matches!(
            self.kind,
            EventKind::OpenTagEnd | EventKind::SelfClosingTag
        ));
        self.aux != 0
    }

    pub(crate) fn interpolation_width(&self) -> usize {
        debug_assert!(self.kind == EventKind::Interpolation);
        if self.aux == 3 { 3 } else { 2 }
    }
}

/// Records authored syntax spans and diagnostics without interpreting them.
pub(crate) struct Recorder<'a, 'v> {
    pub events: &'v mut Vec<'a, Event>,
    pub errors: &'v mut Vec<'a, SurfaceError>,
}

impl Recorder<'_, '_> {
    fn push(&mut self, kind: EventKind, start: usize, end: usize) {
        self.events.push(Event::new(kind, start, end));
    }

    pub(crate) fn raw_interpolation(&mut self, start: usize, end: usize) {
        self.events.push(Event {
            kind: EventKind::Interpolation,
            aux: 3,
            start: start as u32,
            end: end as u32,
        });
    }

    fn attr_end(&mut self, quote: QuoteType, end: usize) {
        self.events.push(Event {
            kind: EventKind::AttrEnd,
            aux: quote as u8,
            start: end as u32,
            end: end as u32,
        });
    }

    pub(crate) fn opening_end(&mut self, kind: EventKind, end: usize, verbatim: bool) {
        debug_assert!(matches!(
            kind,
            EventKind::OpenTagEnd | EventKind::SelfClosingTag
        ));
        self.events.push(Event {
            kind,
            aux: u8::from(verbatim),
            start: end as u32,
            end: end as u32,
        });
    }
}

impl Callbacks for Recorder<'_, '_> {
    fn on_text(&mut self, start: usize, end: usize) {
        self.push(EventKind::Text, start, end);
    }

    fn on_text_entity(&mut self, _char: char, start: usize, end: usize) {
        // The surface keeps raw bytes; embed construction owns decoding.
        self.push(EventKind::Text, start, end);
    }

    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.push(EventKind::Interpolation, start, end);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.push(EventKind::OpenTagName, start, end);
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.push(EventKind::OpenTagEnd, end, end);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        self.push(EventKind::SelfClosingTag, end, end);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.push(EventKind::CloseTag, start, end);
    }

    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrData, start, end);
    }

    fn on_attrib_entity(&mut self, _char: char, start: usize, end: usize) {
        self.push(EventKind::AttrData, start, end);
    }

    fn on_attrib_end(&mut self, quote: CompatQuote, end: usize) {
        let quote = match quote {
            CompatQuote::NoValue => QuoteType::NoValue,
            CompatQuote::Unquoted => QuoteType::Unquoted,
            CompatQuote::Single => QuoteType::Single,
            CompatQuote::Double => QuoteType::Double,
        };
        self.attr_end(quote, end);
    }

    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_attrib_name_end(&mut self, end: usize) {
        self.push(EventKind::AttrNameEnd, end, end);
    }

    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.push(EventKind::AttrName, start, end);
    }

    fn on_comment(&mut self, start: usize, end: usize) {
        self.push(EventKind::Comment, start, end);
    }

    // `on_in_tag_comment` keeps its empty default: the experimental
    // in-tag comment's bytes ride in the next token's `leading` (hole
    // policy clause 3).

    fn on_cdata(&mut self, start: usize, end: usize) {
        self.push(EventKind::Cdata, start, end);
    }

    fn on_processing_instruction(&mut self, start: usize, end: usize) {
        self.push(EventKind::ProcessingInstruction, start, end);
    }

    fn on_end(&mut self) {}

    fn on_error(&mut self, code: ErrorCode, index: usize) {
        self.errors.push(SurfaceError {
            code,
            offset: index as u32,
        });
    }
}
