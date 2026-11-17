//! Entity decoding for markup.
mod decoder;
mod value;

pub(crate) use decoder::decode;
pub use decoder::{EntityContext, decode_one, needs_decoding};
pub use value::DecodedEntity;
