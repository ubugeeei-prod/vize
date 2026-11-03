//! Published tokenizer vocabulary over the shared profile lexer.
//!
//! The facade preserves callbacks and per-file switches used by existing
//! parsers. Only the native lexer executes a state machine.

pub mod adapter;
pub mod char_codes;
pub mod entity_decode;
mod types;
pub use types::*;

use super::{Delimiters, LexOptions, Lexer};
use crate::markup::{Component, Document};
use adapter::CompatSink;

enum ProfileLexer<'a, C: Callbacks> {
    Component(Lexer<'a, Component, CompatSink<C>>),
    Document(Lexer<'a, Document, CompatSink<C>>),
}

/// Compatibility entry for the published tokenizer API.
///
/// This owns the shared lexer throughout repeated calls: EOF cleanup and
/// callback-controlled verbatim mode retain their original behavior.
pub struct Tokenizer<'a, C: Callbacks> {
    lexer: Option<ProfileLexer<'a, C>>,
}

impl<'a, C: Callbacks> Tokenizer<'a, C> {
    #[inline]
    pub fn new(input: &'a str, callbacks: C) -> Self {
        Self::with_delimiters(input, callbacks, b"{{", b"}}")
    }

    #[inline]
    pub fn with_delimiters(
        input: &'a str,
        callbacks: C,
        delimiter_open: &'a [u8],
        delimiter_close: &'a [u8],
    ) -> Self {
        Self {
            lexer: Some(ProfileLexer::Component(Lexer::new(
                input,
                CompatSink::new(callbacks),
                LexOptions {
                    delimiters: Delimiters {
                        open: delimiter_open,
                        close: delimiter_close,
                    },
                    ..LexOptions::default()
                },
            ))),
        }
    }

    /// Tolerate declarations using the Document lexer profile. Changing the
    /// profile retains the input cursor, EOF state and owned callbacks.
    #[inline]
    pub fn set_tolerate_declarations(&mut self, tolerate: bool) {
        match &self.lexer {
            Some(ProfileLexer::Component(_)) if !tolerate => return,
            Some(ProfileLexer::Document(_)) if tolerate => return,
            _ => {}
        }
        if let Some(lexer) = self.lexer.take() {
            self.lexer = Some(match lexer {
                ProfileLexer::Component(lexer) => ProfileLexer::Document(lexer.into_profile()),
                ProfileLexer::Document(lexer) => ProfileLexer::Component(lexer.into_profile()),
            });
        }
    }

    /// Enable the published raw-interpolation switch. Its admission checks
    /// only the opening delimiter, preserving custom-close compatibility.
    #[inline]
    pub fn set_triple_mustache(&mut self, enabled: bool) {
        match self.lexer.as_mut() {
            Some(ProfileLexer::Component(lexer)) => lexer.set_raw_interpolation(enabled),
            Some(ProfileLexer::Document(lexer)) => lexer.set_raw_interpolation(enabled),
            None => {}
        }
    }

    /// Enable experimental // comments inside opening tags.
    #[inline]
    pub fn set_in_tag_comments(&mut self, enabled: bool) {
        match self.lexer.as_mut() {
            Some(ProfileLexer::Component(lexer)) => lexer.set_in_tag_comments(enabled),
            Some(ProfileLexer::Document(lexer)) => lexer.set_in_tag_comments(enabled),
            None => {}
        }
    }

    /// Tokenize the source and emit EOF cleanup followed by on_end.
    #[inline]
    pub fn tokenize(&mut self) {
        match self.lexer.as_mut() {
            Some(ProfileLexer::Component(lexer)) => lexer.run(),
            Some(ProfileLexer::Document(lexer)) => lexer.run(),
            None => {}
        }
    }

    #[cfg(test)]
    fn into_callbacks(self) -> Option<C> {
        self.lexer.map(|lexer| match lexer {
            ProfileLexer::Component(lexer) => lexer.into_sink().into_callbacks(),
            ProfileLexer::Document(lexer) => lexer.into_sink().into_callbacks(),
        })
    }
}

#[cfg(test)]
mod empty_delimiter_tests;
#[cfg(test)]
mod facade_tests;
#[cfg(test)]
mod tests;
