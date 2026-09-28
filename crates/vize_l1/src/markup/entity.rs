//! Entity decoding for markup.

#[cfg(any(test, feature = "native-markup-lex"))]
mod native;
#[cfg(not(any(test, feature = "native-markup-lex")))]
mod skeleton;

#[cfg(any(test, feature = "native-markup-lex"))]
pub(crate) use native::decode;
#[cfg(any(test, feature = "native-markup-lex"))]
pub use native::{DecodedEntity, EntityContext, decode_one, needs_decoding};
#[cfg(not(any(test, feature = "native-markup-lex")))]
pub use skeleton::{EntityContext, decode_one, needs_decoding};
