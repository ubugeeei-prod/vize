//! Pairing a `--port` server with the editor that launched it.
//!
//! Loopback is reachable by every program on the machine, and the server
//! serves the first connection it takes. A launcher that shares a token with
//! the server (`--token-file`, `VIZE_LSP_TOKEN`) and sends it as
//! `initializationOptions.vizeToken` makes the server take only its own
//! client: a connection whose first message is not that `initialize` is
//! closed, and the server waits for the next one.

use std::io::{self, Read};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

/// The token the first `initialize` of a `--port` connection must carry.
#[derive(Eq, PartialEq)]
pub struct PairingToken(Box<str>);

impl PairingToken {
    /// The token in `text`, without surrounding whitespace, so a token file
    /// may end with a newline. `None` when nothing is left.
    pub fn new(text: &str) -> Option<Self> {
        let token = text.trim();
        (!token.is_empty()).then(|| Self(token.into()))
    }
}

impl std::fmt::Debug for PairingToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PairingToken(..)")
    }
}

/// How long a connection may take to send its first message.
const OPENING_DEADLINE: Duration = Duration::from_secs(10);
/// Longest header block accepted before the first message's body.
const HEADER_LIMIT: usize = 4096;
/// Largest first message accepted; `initialize` carries client capabilities
/// and workspace folders, which stay far below this.
const OPENING_LIMIT: usize = 16 * 1024 * 1024;

/// Why a connection was closed before the server took it.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Refusal {
    /// The connection ended or stalled before a whole message arrived.
    Incomplete,
    /// The first bytes were not one LSP message within the limits above.
    Malformed,
    /// The first message was not `initialize`.
    NotInitialize,
    /// `initializationOptions.vizeToken` was missing or different.
    TokenMismatch,
}

impl Refusal {
    pub(crate) fn describe(&self) -> &'static str {
        match self {
            Self::Incomplete => "it ended or stalled before its first message",
            Self::Malformed => "its first message was not a well-formed LSP message",
            Self::NotInitialize => "its first message was not initialize",
            Self::TokenMismatch => "its initialize did not carry the pairing token",
        }
    }
}

/// The first connection to `listener` that this server takes: opened by this
/// account where the OS tells, and, with a `token`, opened by an `initialize`
/// carrying it. Returns the stream, its peer, and the bytes already read from
/// it.
pub(crate) async fn accept_client(
    listener: &TcpListener,
    token: Option<&PairingToken>,
) -> io::Result<(TcpStream, SocketAddr, Vec<u8>)> {
    let local = listener.local_addr()?;
    loop {
        let (stream, peer) =
            crate::runtime::accept_tcp("vize-lsp-tcp-accept", listener.try_clone()?).await?;
        if !crate::loopback_peer::opened_by_this_account(local, peer) {
            tracing::warn!(
                "Closed a connection from {} opened by another account",
                peer
            );
            continue;
        }
        let Some(token) = token else {
            return Ok((stream, peer, Vec::new()));
        };
        match read_opening(&stream, token) {
            Ok(opening) => return Ok((stream, peer, opening)),
            Err(refusal) => {
                tracing::warn!("Closed a connection from {}: {}", peer, refusal.describe());
            }
        }
    }
}

/// Read the first message of `stream` and check that it is an `initialize`
/// carrying `token`. On success the bytes read so far are returned so they
/// can be served ahead of the rest of the stream.
pub(crate) fn read_opening(stream: &TcpStream, token: &PairingToken) -> Result<Vec<u8>, Refusal> {
    let deadline_set = stream.set_read_timeout(Some(OPENING_DEADLINE)).is_ok();
    let mut reader = stream;
    let opening = read_paired_message(&mut reader, token);
    if deadline_set && stream.set_read_timeout(None).is_err() {
        return Err(Refusal::Incomplete);
    }
    opening
}

fn read_paired_message(reader: &mut impl Read, token: &PairingToken) -> Result<Vec<u8>, Refusal> {
    let mut bytes = Vec::with_capacity(8192);
    let (body_start, body_len) = loop {
        if let Some(header_len) = header_block_len(&bytes) {
            let header = bytes.get(..header_len).ok_or(Refusal::Malformed)?;
            break (header_len, content_length(header)?);
        }
        if bytes.len() > HEADER_LIMIT {
            return Err(Refusal::Malformed);
        }
        read_more(reader, &mut bytes)?;
    };
    let body_end = body_start
        .checked_add(body_len)
        .filter(|end| *end <= OPENING_LIMIT)
        .ok_or(Refusal::Malformed)?;
    while bytes.len() < body_end {
        read_more(reader, &mut bytes)?;
    }
    let body = bytes.get(body_start..body_end).ok_or(Refusal::Malformed)?;
    check_initialize(body, token)?;
    Ok(bytes)
}

fn read_more(reader: &mut impl Read, bytes: &mut Vec<u8>) -> Result<(), Refusal> {
    let mut chunk = [0; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => return Err(Refusal::Incomplete),
            Ok(len) => {
                bytes.extend_from_slice(chunk.get(..len).ok_or(Refusal::Incomplete)?);
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Refusal::Incomplete),
        }
    }
}

/// Length of the header block including its closing blank line.
fn header_block_len(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|start| start + 4)
}

fn content_length(header: &[u8]) -> Result<usize, Refusal> {
    let header = std::str::from_utf8(header).map_err(|_| Refusal::Malformed)?;
    header
        .split("\r\n")
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse().ok())
        .ok_or(Refusal::Malformed)
}

fn check_initialize(body: &[u8], token: &PairingToken) -> Result<(), Refusal> {
    let message: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| Refusal::Malformed)?;
    if message.get("method").and_then(serde_json::Value::as_str) != Some("initialize") {
        return Err(Refusal::NotInitialize);
    }
    let presented = message
        .pointer("/params/initializationOptions/vizeToken")
        .and_then(serde_json::Value::as_str);
    if presented != Some(&*token.0) {
        return Err(Refusal::TokenMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
