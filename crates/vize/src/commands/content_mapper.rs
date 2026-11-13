//! TypeScript content-mapper protocol server.
//!
//! Speaks the TypeScript content-mapper protocol:
//! `initialize` negotiates capabilities and the position encoding, `openProject` and
//! `closeProject` bracket each TypeScript project's mapper options and
//! compiler options, and `transform` projects one Vue SFC into virtual
//! TypeScript for an opened project handle.

use std::io::{self, BufRead, BufReader, BufWriter, Read};

use clap::Args;
use serde::Deserialize;
use serde_json::Value;
use vize_l0::String as CompactString;
#[cfg(test)]
use vize_l0::cstr;

mod dispatch;
mod helper_abi;
mod mode;
mod project;

const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 8 * 1024;

#[derive(Args, Default)]
pub struct ContentMapperArgs {
    /// The JS launcher already answered `initialize` on this stream, before
    /// loading the native addon, so the host's fixed `initialize` wait never
    /// covers that load (#12530).
    #[arg(long, hide = true)]
    initialized: bool,
    /// Write each reply once it is ready instead of in request order. The
    /// TypeScript hosts pair replies by id, so their launcher asks for this.
    #[arg(long, hide = true)]
    reply_when_ready: bool,
}

pub fn run(args: ContentMapperArgs) {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(io::stdout());
    let order = if args.reply_when_ready {
        dispatch::ReplyOrder::Ready
    } else {
        dispatch::ReplyOrder::Request
    };
    #[cfg(feature = "profiling")]
    let allocation_report = allocation_report::start();
    let served = dispatch::serve(&mut reader, &mut writer, args.initialized, order);
    #[cfg(feature = "profiling")]
    allocation_report::finish(allocation_report);
    if let Err(error) = served {
        eprintln!("vize content-mapper: {error}");
        std::process::exit(1);
    }
}

/// `--features profiling` builds count every allocation of a session and
/// write the totals to `$VIZE_CONTENT_MAPPER_ALLOC_REPORT` at EOF, for
/// tools/benchmarks/scripts/content-mapper-replay.ts.
#[cfg(feature = "profiling")]
mod allocation_report {
    use std::ffi::OsString;

    use vize_l0::profiler::{
        allocation_snapshot, reset_allocation_counters, set_allocation_tracking_enabled,
    };

    pub(super) fn start() -> Option<OsString> {
        let path = std::env::var_os("VIZE_CONTENT_MAPPER_ALLOC_REPORT")?;
        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        Some(path)
    }

    pub(super) fn finish(path: Option<OsString>) {
        let Some(path) = path else { return };
        set_allocation_tracking_enabled(false);
        let snapshot = allocation_snapshot();
        let report = serde_json::json!({
            "allocationCalls": snapshot.allocation_calls(),
            "allocatedBytes": snapshot.requested_bytes(),
            "allocCalls": snapshot.alloc_calls,
            "allocZeroedCalls": snapshot.alloc_zeroed_calls,
            "reallocCalls": snapshot.realloc_calls,
            "reallocOldBytes": snapshot.realloc_old_bytes,
            "reallocNewBytes": snapshot.realloc_new_bytes,
            "deallocCalls": snapshot.dealloc_calls,
        });
        let written = serde_json::to_vec(&report)
            .map_err(std::io::Error::from)
            .and_then(|bytes| std::fs::write(&path, bytes));
        if let Err(error) = written {
            eprintln!("vize content-mapper: allocation report: {error}");
        }
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

#[cfg(test)]
fn serve<R: BufRead, W: io::Write + Send>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    dispatch::serve(reader, writer, false, dispatch::ReplyOrder::Request)
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
mod helper_abi_tests;

#[cfg(test)]
mod helper_abi_fixture;

#[cfg(test)]
mod frame_tests;

#[cfg(test)]
mod typed_router_tests;
