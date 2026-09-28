//! Compatibility path for the template tokenizer now owned by L1.
//!
//! Existing compiler callers keep their `vize_armature::tokenizer` imports
//! while the markup lexer and its event vocabulary live in `vize_l1`.

pub use vize_l1::markup::lex::compat::*;
