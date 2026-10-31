//! TypeScript content-mapper protocol server.
//!
//! Speaks the TypeScript content-mapper protocol:
//! `initialize` negotiates capabilities and the position encoding, `openProject` and
//! `closeProject` bracket each TypeScript project's mapper options and
//! compiler options, and `transform` projects one Vue SFC into virtual
//! TypeScript for an opened project handle.

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

use clap::Args;
use serde::Deserialize;
use serde_json::Value;
use vize_l0::String as CompactString;
#[cfg(test)]
use vize_l0::cstr;

mod dispatch;
mod project;

const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 8 * 1024;

#[derive(Args, Default)]
pub struct ContentMapperArgs {}

pub fn run(_: ContentMapperArgs) {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(io::stdout());
    if let Err(error) = serve(&mut reader, &mut writer) {
        eprintln!("vize content-mapper: {error}");
        std::process::exit(1);
    }
}

#[derive(Deserialize)]
struct Request {
    #[expect(dead_code, reason = "read via Debug snapshots")]
    jsonrpc: Option<CompactString>,
    id: Option<Value>,
    method: CompactString,
    #[serde(default)]
    params: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    #[expect(dead_code, reason = "read via Debug snapshots")]
    locale: Option<CompactString>,
    position_encodings: Vec<CompactString>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformParams {
    file_name: CompactString,
    content: CompactString,
    project_handle: CompactString,
}

fn serve<R: BufRead, W: Write + Send>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    dispatch::serve(reader, writer)
}

fn read_frame<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut content_length = None;
    let mut saw_header = false;
    let mut header_bytes = 0;
    loop {
        let remaining = MAX_HEADER_BYTES - header_bytes;
        if remaining == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "content-mapper headers exceed 8 KiB",
            ));
        }
        let mut line = Vec::new();
        // Limit the read itself: checking after read_until would already have
        // allocated an arbitrarily large line, or accepted unbounded lines.
        let bytes = (&mut *reader)
            .take(remaining as u64)
            .read_until(b'\n', &mut line)?;
        header_bytes += bytes;
        if bytes == remaining && !line.ends_with(b"\n") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "content-mapper headers exceed 8 KiB",
            ));
        }
        if bytes == 0 {
            if saw_header {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF inside content-mapper headers",
                ));
            }
            return Ok(None);
        }
        saw_header = true;
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        if let Some(separator) = line.iter().position(|byte| *byte == b':')
            && line
                .get(..separator)
                .unwrap_or_default()
                .eq_ignore_ascii_case(b"content-length")
        {
            if content_length.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate Content-Length header",
                ));
            }
            let value = std::str::from_utf8(line.get(separator + 1..).unwrap_or_default())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length"))?
                .trim();
            content_length = Some(value.parse::<usize>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length")
            })?);
        }
    }

    let content_length = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;
    if content_length > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "content-mapper message exceeds 64 MiB",
        ));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod project_tests;

#[cfg(test)]
mod shared_helper_tests;

#[cfg(test)]
mod frame_tests;

#[cfg(test)]
mod typed_router_tests;
