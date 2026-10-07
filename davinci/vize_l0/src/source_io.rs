//! Borrowed UTF-8 validation for authored source bytes.
//! Validation never normalizes or repairs bytes, preserving exact offsets.

use core::str::Utf8Error;

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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
