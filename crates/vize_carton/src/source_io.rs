//! Owned source-file buffers at the host filesystem boundary.
//! Authored bytes and standard filesystem errors are preserved.

#[cfg(not(target_arch = "wasm32"))]
use std::{io, path::Path};

#[expect(
    clippy::disallowed_types,
    reason = "Preserve the std file-reader's owned buffer without a copy"
)]
#[cfg(not(target_arch = "wasm32"))]
use std::string::String as FileString;

pub use vize_l0::source_io::decode_utf8;

/// Read a UTF-8 file without copying or validating its owned buffer twice.
///
/// Filesystem errors and invalid-UTF-8 error kind/message match
/// [`std::fs::read_to_string`]. Empty files, BOMs and newlines are preserved.
#[expect(
    clippy::disallowed_types,
    reason = "Preserve the validated std file buffer without an additional allocation or copy"
)]
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
#[expect(
    clippy::disallowed_methods,
    reason = "The filesystem law compares the existing std diagnostic spelling"
)]
mod tests;
