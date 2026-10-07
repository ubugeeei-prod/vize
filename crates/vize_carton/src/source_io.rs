//! UTF-8 validation at owned external source boundaries.
//!
//! File buffers are owned independently of the compile arena. Validation never
//! decodes, normalizes or repairs bytes, so authored byte offsets stay exact.

use core::str::Utf8Error;
#[cfg(not(target_arch = "wasm32"))]
use std::{io, path::Path};

#[expect(
    clippy::disallowed_types,
    reason = "Preserve the std file-reader's owned buffer without a copy"
)]
#[cfg(not(target_arch = "wasm32"))]
use std::string::String as FileString;

/// Validate source bytes, preserving the standard library's exact error type.
///
/// SIMD handles large inputs. Invalid inputs are rechecked by the standard
/// library so `valid_up_to`, `error_len`, and diagnostic spelling stay exact.
#[inline]
pub fn decode_utf8(bytes: &[u8]) -> Result<&str, Utf8Error> {
    if bytes.len() < 64 {
        return core::str::from_utf8(bytes);
    }
    match simdutf8::compat::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(_) => core::str::from_utf8(bytes),
    }
}

/// Read a UTF-8 file without copying or validating its owned buffer twice.
///
/// Filesystem errors and invalid-UTF-8 error kind/message match
/// [`std::fs::read_to_string`]. Empty files, BOMs and newlines are preserved.
#[cfg(not(target_arch = "wasm32"))]
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<FileString> {
    let bytes = std::fs::read(path)?;
    if decode_utf8(&bytes).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    // SAFETY: decode_utf8 accepted this exact buffer above. The buffer is
    // exclusively owned and cannot change between validation and this move.
    Ok(unsafe { FileString::from_utf8_unchecked(bytes) })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
