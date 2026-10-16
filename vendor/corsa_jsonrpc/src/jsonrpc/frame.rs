//! Content-Length framing helpers for stdio JSON-RPC.
//!
//! Corsa uses the same header-based framing style as LSP: an ASCII
//! header block terminated by `\r\n\r\n`, followed by an exact number of body
//! bytes. The helpers in this module focus on that framing layer only; they do
//! not parse or validate the JSON payload itself.

use crate::{CorsaError, Result};
use corsa_core::fast::{SmallVec, memchr, memmem};
use std::io::{BufRead, Write};

const HEADER_END: &[u8] = b"\r\n\r\n";
const CONTENT_LENGTH: &[u8] = b"content-length";
const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 512 * 1024 * 1024;

/// Reads a single stdio JSON-RPC frame.
///
/// The reader is consumed only as far as the end of the current frame, so it
/// can be reused for subsequent calls.
///
/// # Errors
///
/// Returns [`CorsaError::Protocol`] when the header is malformed or exceeds the
/// maximum allowed size, and [`CorsaError::Closed`] when EOF is reached before a
/// complete header can be read.
///
/// # Examples
///
/// ```
/// use std::io::{BufReader, Cursor};
/// use corsa_jsonrpc::read_frame;
///
/// let bytes = b"Content-Length: 17\r\n\r\n{\"jsonrpc\":\"2.0\"}";
/// let mut reader = BufReader::new(Cursor::new(bytes.as_slice()));
/// let payload = read_frame(&mut reader)?;
/// assert_eq!(payload, br#"{"jsonrpc":"2.0"}"#);
/// # Ok::<(), corsa_jsonrpc::CorsaError>(())
/// ```
pub fn read_frame<R>(reader: &mut R) -> Result<Vec<u8>>
where
    R: BufRead,
{
    let content_length = read_content_length(reader)?;
    if content_length > MAX_BODY_BYTES {
        return Err(CorsaError::Protocol(
            format!(
                "jsonrpc body is too large: {content_length} bytes exceeds {MAX_BODY_BYTES} byte safety limit"
            )
            .into(),
        ));
    }
    let mut payload = Vec::new();
    payload.try_reserve_exact(content_length).map_err(|err| {
        CorsaError::Protocol(
            format!("failed to reserve jsonrpc body buffer for {content_length} bytes: {err}")
                .into(),
        )
    })?;
    payload.resize(content_length, 0);
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

/// Writes a single stdio JSON-RPC frame.
///
/// The function always flushes the writer before returning so request/response
/// protocols do not remain buffered indefinitely.
///
/// # Errors
///
/// Returns any I/O error encountered while writing the header, payload, or
/// flush operation.
///
/// # Examples
///
/// ```
/// use corsa_jsonrpc::write_frame;
///
/// let mut buffer = Vec::new();
/// write_frame(&mut buffer, br#"{"jsonrpc":"2.0"}"#)?;
/// assert!(buffer.starts_with(b"Content-Length: "));
/// assert!(buffer.ends_with(br#"{"jsonrpc":"2.0"}"#));
/// # Ok::<(), corsa_jsonrpc::CorsaError>(())
/// ```
pub fn write_frame<W>(writer: &mut W, body: &[u8]) -> Result<()>
where
    W: Write,
{
    let mut header = SmallVec::<[u8; 32]>::new();
    header.extend_from_slice(b"Content-Length: ");
    append_usize(&mut header, body.len());
    header.extend_from_slice(HEADER_END);
    writer.write_all(&header)?;
    writer.write_all(body)?;
    writer.flush()?;
    Ok(())
}

fn read_content_length<R>(reader: &mut R) -> Result<usize>
where
    R: BufRead,
{
    let mut header = SmallVec::<[u8; 32]>::new();
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Err(CorsaError::Closed("jsonrpc reader"));
        }
        let previous_len = header.len();
        header.extend_from_slice(chunk);
        if let Some(index) = memmem::find(&header, HEADER_END) {
            // Search the accumulated header rather than only the latest
            // `fill_buf` slice: stdio transports may split `\r\n\r\n` across
            // reads. Consume only bytes that belong to this frame's header so
            // any already-buffered body bytes remain available to `read_exact`.
            let header_len = index + HEADER_END.len();
            let consumed = header_len.saturating_sub(previous_len);
            reader.consume(consumed);
            header.truncate(header_len);
            return parse_content_length(&header);
        }
        if header.len() > MAX_HEADER_BYTES {
            return Err(CorsaError::Protocol("jsonrpc header is too large".into()));
        }
        let consumed = chunk.len();
        reader.consume(consumed);
    }
}

/// Extracts the `Content-Length` header from a raw header block.
fn parse_content_length(header: &[u8]) -> Result<usize> {
    for line in header.split(|byte| *byte == b'\n') {
        let line = trim_ascii(trim_eol(line));
        if line.is_empty() {
            continue;
        }
        let Some(index) = memchr(b':', line) else {
            continue;
        };
        let key = trim_ascii(&line[..index]);
        let value = trim_ascii(&line[index + 1..]);
        if key.eq_ignore_ascii_case(CONTENT_LENGTH) {
            return parse_ascii_usize(value);
        }
    }
    Err(CorsaError::Protocol("missing Content-Length".into()))
}

/// Removes a trailing carriage return left over from splitting on `\n`.
fn trim_eol(bytes: &[u8]) -> &[u8] {
    bytes.strip_suffix(b"\r").unwrap_or(bytes)
}

/// Trims leading and trailing ASCII whitespace.
fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while start < end && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &bytes[start..end]
}

/// Parses a non-empty ASCII decimal integer into `usize`.
fn parse_ascii_usize(bytes: &[u8]) -> Result<usize> {
    if bytes.is_empty() {
        return Err(CorsaError::Protocol("empty Content-Length".into()));
    }
    let mut value = 0_usize;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return Err(CorsaError::Protocol("invalid Content-Length".into()));
        }
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add((byte - b'0') as usize))
            .ok_or_else(|| CorsaError::Protocol("Content-Length overflow".into()))?;
    }
    Ok(value)
}

/// Appends the ASCII decimal representation of `value` to `buffer`.
///
/// This avoids allocating a temporary `String` on the hot write path.
fn append_usize(buffer: &mut SmallVec<[u8; 32]>, mut value: usize) {
    if value == 0 {
        buffer.push(b'0');
        return;
    }
    let mut digits = [0_u8; 20];
    let mut len = 0;
    while value > 0 {
        digits[len] = b'0' + (value % 10) as u8;
        value /= 10;
        len += 1;
    }
    while len > 0 {
        len -= 1;
        buffer.push(digits[len]);
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod tests;
