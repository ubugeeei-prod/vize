//! Bounded acceptance bytes through the unchanged tower-lsp codec.
#![expect(
    clippy::disallowed_types,
    reason = "the actual framed acceptance transport shares its byte-stream owners"
)]

use futures::io::{AsyncRead, AsyncWrite};
use parking_lot::Mutex;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

const LIMIT: usize = 256 * 1024;

struct State {
    case: &'static str,
    incoming: VecDeque<u8>,
    sent: Vec<u8>,
    consumed: Vec<u8>,
    written: Vec<u8>,
    eof: bool,
    stopped: bool,
    read_waker: Option<Waker>,
    write_waker: Option<Waker>,
}

#[derive(Clone)]
pub(super) struct Peer(Arc<Mutex<State>>);
pub(super) struct Input(Arc<Mutex<State>>);
pub(super) struct Output(Arc<Mutex<State>>);

pub(super) fn pipes(case: &'static str) -> (Peer, Input, Output) {
    let state = Arc::new(Mutex::new(State {
        case,
        incoming: VecDeque::new(),
        sent: Vec::new(),
        consumed: Vec::new(),
        written: Vec::new(),
        eof: false,
        stopped: false,
        read_waker: None,
        write_waker: None,
    }));
    (
        Peer(Arc::clone(&state)),
        Input(Arc::clone(&state)),
        Output(state),
    )
}

fn chunk(case: &str, direction: &str, bytes: &[u8]) {
    // Every actual chunk is durable even if a later decode/assertion fails.
    println!(
        "VIZE_ARTIFACT_DEFINITION_WIRE {}",
        json!({
            "case":case,"direction":direction,"bytes":bytes
        })
    );
}

impl Peer {
    pub(super) fn send(&self, message: Value) {
        let body = serde_json::to_vec(&message).unwrap();
        let header = vize_l0::cstr!("Content-Length: {}\r\n\r\n", body.len());
        let bytes = [header.as_bytes(), body.as_slice()].concat();
        let wake = {
            let mut state = self.0.lock();
            assert!(state.sent.len() + bytes.len() <= LIMIT);
            state.sent.extend_from_slice(&bytes);
            state.incoming.extend(bytes);
            state.read_waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }

    pub(super) async fn response(&self, id: i64) -> Option<Value> {
        self.wait(|message| message.get("id").and_then(Value::as_i64) == Some(id))
            .await
    }

    pub(super) async fn ready(&self, uri: &str, version: i32) -> Option<Value> {
        self.wait(|message| {
            message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && message["params"]["uri"] == uri
                && message["params"]["version"] == version
        })
        .await
    }

    async fn wait(&self, matches: impl Fn(&Value) -> bool) -> Option<Value> {
        futures::future::poll_fn(|cx| {
            let mut state = self.0.lock();
            let Ok(messages) = decode(&state.written, false) else {
                return Poll::Ready(None);
            };
            if let Some(message) = messages.into_iter().find(&matches) {
                return Poll::Ready(Some(message));
            }
            if state.stopped {
                return Poll::Ready(None);
            }
            state.write_waker = Some(cx.waker().clone());
            Poll::Pending
        })
        .await
    }

    pub(super) fn finish(&self) {
        let wake = {
            let mut state = self.0.lock();
            state.eof = true;
            state.read_waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }

    pub(super) fn stopped(&self) {
        let wake = {
            let mut state = self.0.lock();
            state.stopped = true;
            state.write_waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }

    pub(super) fn tape(&self) -> Value {
        let state = self.0.lock();
        json!({
            "case":state.case,"clientOffered":state.sent,
            "clientConsumed":state.consumed,"serverWritten":state.written,
            "unreadBytes":state.incoming.len(),"transportStopped":state.stopped,
            "inputEof":state.eof
        })
    }
}

impl AsyncRead for Input {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if buffer.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let mut state = self.0.lock();
        if !state.incoming.is_empty() {
            let count = buffer.len().min(state.incoming.len());
            for byte in buffer.iter_mut().take(count) {
                *byte = state.incoming.pop_front().unwrap();
            }
            state.consumed.extend_from_slice(&buffer[..count]);
            let case = state.case;
            drop(state);
            chunk(case, "client-consumed", &buffer[..count]);
            return Poll::Ready(Ok(count));
        }
        if state.eof {
            Poll::Ready(Ok(0))
        } else {
            state.read_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl AsyncWrite for Output {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut state = self.0.lock();
        if state.written.len() + bytes.len() > LIMIT {
            return Poll::Ready(Err(io::Error::other("definition acceptance wire overflow")));
        }
        state.written.extend_from_slice(bytes);
        let case = state.case;
        let wake = state.write_waker.take();
        drop(state);
        chunk(case, "server-written", bytes);
        if let Some(wake) = wake {
            wake.wake();
        }
        Poll::Ready(Ok(bytes.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub(super) fn decode(mut bytes: &[u8], complete: bool) -> Result<Vec<Value>, &'static str> {
    let mut messages = Vec::new();
    while !bytes.is_empty() {
        let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
            return if complete {
                Err("partial frame header")
            } else {
                Ok(messages)
            };
        };
        let header = std::str::from_utf8(&bytes[..end]).map_err(|_| "non-UTF8 header")?;
        let count = header
            .strip_prefix("Content-Length: ")
            .ok_or("foreign header")?
            .parse::<usize>()
            .map_err(|_| "invalid content length")?;
        if count > LIMIT {
            return Err("oversize frame");
        }
        let body = end + 4;
        if bytes.len() - body < count {
            return if complete {
                Err("partial frame body")
            } else {
                Ok(messages)
            };
        }
        messages.push(
            serde_json::from_slice(&bytes[body..body + count]).map_err(|_| "invalid JSON frame")?,
        );
        bytes = &bytes[body + count..];
    }
    Ok(messages)
}
