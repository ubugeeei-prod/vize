//! Entity decoding for markup.

#[cfg(any(test, feature = "native-markup-lex"))]
mod native;
#[cfg(not(any(test, feature = "native-markup-lex")))]
mod skeleton;
mod value;

#[cfg(any(test, feature = "native-markup-lex"))]
pub(crate) use native::decode;
#[cfg(any(test, feature = "native-markup-lex"))]
pub use native::{EntityContext, decode_one, needs_decoding};
#[cfg(not(any(test, feature = "native-markup-lex")))]
pub use skeleton::{EntityContext, decode_one, needs_decoding};
pub use value::DecodedEntity;
