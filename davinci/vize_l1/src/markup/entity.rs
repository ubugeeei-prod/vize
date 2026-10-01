//! Entity decoding for markup.

mod native;
mod value;

pub(crate) use native::decode;
pub use native::{EntityContext, decode_one, needs_decoding};
pub use value::DecodedEntity;
