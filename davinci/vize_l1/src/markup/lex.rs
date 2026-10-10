//! Markup lexer entry point.

/// Byte-identical template tokenizer moved from Armature. The compiler keeps
/// using this entry point while the generic profile lexer is validated.
pub mod compat;

mod native;

pub use native::{
    Delimiters, LexOptions, Lexer, char_codes, dynamic_argument_boundary, is_end_of_tag_section,
    is_tag_start_char, is_whitespace,
};
