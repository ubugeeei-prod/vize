//! Markup lexer entry point.

/// Byte-identical template tokenizer moved from Armature. The compiler keeps
/// using this entry point while the generic profile lexer is validated.
pub mod compat;

#[cfg(any(test, feature = "native-markup-lex"))]
mod native;
#[cfg(not(any(test, feature = "native-markup-lex")))]
mod skeleton;

#[cfg(any(test, feature = "native-markup-lex"))]
pub use native::{
    Delimiters, LexOptions, Lexer, char_codes, is_end_of_tag_section, is_tag_start_char,
    is_whitespace,
};
#[cfg(not(any(test, feature = "native-markup-lex")))]
pub use skeleton::{Delimiters, LexOptions, Lexer};
