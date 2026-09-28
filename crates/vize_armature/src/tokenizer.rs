//! Compatibility path for the template tokenizer now owned by L1.
//!
//! Existing compiler callers keep their `vize_armature::tokenizer` imports
//! while the markup lexer and its event vocabulary live in `vize_l1`.

#[cfg(not(feature = "native-lex-parity"))]
#[path = "../../vize_l1/src/markup/lex/compat.rs"]
mod l1_compat;
#[cfg(not(feature = "native-lex-parity"))]
pub use l1_compat::*;

#[cfg(feature = "native-lex-parity")]
pub use vize_l1::markup::lex::compat::*;
