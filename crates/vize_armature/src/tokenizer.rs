//! The Vue template tokenizer.
//!
//! The tokenizer lives in `vize_l1::markup` (#6835): armature drives the same
//! lexer the L1 surface tree does. This module re-exports it.

pub use vize_l1::markup::lex::{
    char_codes, is_end_of_tag_section, is_tag_start_char, is_whitespace,
};
pub use vize_l1::markup::{
    Component, Delimiters, Document, LexErrorCode, LexMode, LexOptions, Lexer, Profile, QuoteType,
    Sink,
};
