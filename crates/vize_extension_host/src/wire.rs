//! The out-of-process wire: one JSON document per line over stdio.
//!
//! This is host-internal plumbing between a parent host and the child
//! process that runs a guest, not a published contract: the contract is the
//! WIT world, and both ends of this pipe are the host's own code carrying
//! the WIT values of [`crate::contract`] verbatim (JSON strings escape every
//! control byte, so a line is one message and text is lossless).

use std::io::{self, BufRead, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use crate::contract::{Capability, GuestError, InputDialectGuest, LoweredBlock, SourceBlock};
use crate::expression::{Analysis, ExpressionBatch, ExpressionDialectGuest};

/// A call from the parent host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "call", rename_all = "kebab-case")]
pub enum Request {
    GetCapability,
    LowerBlock { block: SourceBlock },
    Analyze { batch: ExpressionBatch },
}

/// The child's answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Response {
    /// The guest loaded; calls may follow.
    Ready,
    /// The guest did not load; the child exits after this line.
    LoadError(String),
    Capability(Capability),
    LoweredBlock(LoweredBlock),
    Analysis(Analysis),
    /// The call reached the guest and failed there.
    Guest(GuestError),
}

/// Write one message as one line and flush it.
///
/// # Errors
///
/// The underlying I/O or serialization error.
pub fn write_message<W: Write, T: Serialize>(out: &mut W, message: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *out, message)?;
    out.write_all(b"\n")?;
    out.flush()
}

/// Read one message; `None` at end of input.
///
/// # Errors
///
/// The underlying I/O error, or invalid data for a malformed line.
pub fn read_message<R: BufRead, T: DeserializeOwned>(input: &mut R) -> io::Result<Option<T>> {
    let mut line = Vec::new();
    if input.read_until(b'\n', &mut line)? == 0 {
        return Ok(None);
    }
    serde_json::from_slice(&line)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Answer requests from `input` with `answer` until end of input.
///
/// # Errors
///
/// The first I/O failure on either stream.
pub fn serve<R, W>(
    mut answer: impl FnMut(Request) -> Response,
    mut input: R,
    mut output: W,
) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    write_message(&mut output, &Response::Ready)?;
    while let Some(request) = read_message::<_, Request>(&mut input)? {
        write_message(&mut output, &answer(request))?;
    }
    Ok(())
}

fn wrong_world(world: &str) -> Response {
    Response::Guest(GuestError::Trap(cstr!(
        "the guest implements the {world} world"
    )))
}

/// Answer one request with an input-dialect guest.
pub fn answer_input<G: InputDialectGuest>(guest: &mut G, request: Request) -> Response {
    match request {
        Request::GetCapability => guest
            .get_capability()
            .map_or_else(Response::Guest, Response::Capability),
        Request::LowerBlock { block } => guest
            .lower_block(&block)
            .map_or_else(Response::Guest, Response::LoweredBlock),
        Request::Analyze { .. } => wrong_world("input-dialect"),
    }
}

/// Answer one request with an expression-dialect guest.
pub fn answer_expression<G: ExpressionDialectGuest>(guest: &mut G, request: Request) -> Response {
    match request {
        Request::GetCapability => guest
            .get_capability()
            .map_or_else(Response::Guest, Response::Capability),
        Request::Analyze { batch } => guest
            .analyze(&batch)
            .map_or_else(Response::Guest, Response::Analysis),
        Request::LowerBlock { .. } => wrong_world("expression-dialect"),
    }
}

pub(crate) fn transport(error: &io::Error) -> GuestError {
    GuestError::Transport(cstr!("{error}"))
}
