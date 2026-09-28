//! The shared markup lexer, `Lexer<P: Profile, S: Sink>`.
//!
//! One state machine serves every profile; profile rules are associated
//! constants, so the per-profile branches fold at compile time. The lexer
//! makes one pass over the bytes and allocates nothing per token.

#![expect(clippy::todo, reason = "skeleton: #6835")]

use core::marker::PhantomData;

use super::super::profile::Profile;
use super::super::token::Sink;

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
    /// Vue 1.x `{{{ expr }}}` raw-HTML interpolation. Honored only with the
    /// default delimiters.
    pub raw_interpolation: bool,
    /// Experimental `//` comments inside open tags.
    pub in_tag_comments: bool,
}

/// The markup lexer over one source text.
pub struct Lexer<'a, P: Profile, S: Sink> {
    input: &'a [u8],
    sink: S,
    options: LexOptions<'a>,
    profile: PhantomData<P>,
}

impl<'a, P: Profile, S: Sink> Lexer<'a, P, S> {
    pub fn new(source: &'a str, sink: S, options: LexOptions<'a>) -> Self {
        Self {
            input: source.as_bytes(),
            sink,
            options,
            profile: PhantomData,
        }
    }

    /// The source bytes being lexed.
    pub fn input(&self) -> &'a [u8] {
        self.input
    }

    pub fn options(&self) -> &LexOptions<'a> {
        &self.options
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn into_sink(self) -> S {
        self.sink
    }

    /// Lex the whole input, pushing every event and finally `on_end` into
    /// the sink.
    ///
    /// # Panics
    ///
    /// Always, until the tokenizer moves here from `vize_armature` (#6835).
    pub fn run(&mut self) {
        todo!("#6835: move the template tokenizer into vize_l1::markup::lex")
    }
}
