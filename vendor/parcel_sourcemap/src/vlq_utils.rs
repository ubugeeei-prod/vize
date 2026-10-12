// Based on https://github.com/fitzgen/source-map-mappings
use crate::sourcemap_error::{SourceMapError, SourceMapErrorType};
use vlq::decode;

#[inline]
pub fn read_relative_vlq<B>(previous: &mut i64, input: &mut B) -> Result<(), SourceMapError>
where
    B: Iterator<Item = u8>,
{
    let decoded = decode(input)?;
    let (new, overflowed) = (*previous as i64).overflowing_add(decoded);
    if overflowed || new > (u32::MAX as i64) {
        return Err(SourceMapError::new(
            SourceMapErrorType::UnexpectedlyBigNumber,
        ));
    }

    if new < 0 {
        return Err(SourceMapError::new(
            SourceMapErrorType::UnexpectedNegativeNumber,
        ));
    }

    *previous = new;

    Ok(())
}

#[inline]
pub fn is_mapping_separator(byte: u8) -> bool {
    byte == b';' || byte == b','
}
