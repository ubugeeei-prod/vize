//! Real framed bytes enter the unchanged tower-lsp transport.
use futures::io::{AsyncRead, AsyncWrite};
use parking_lot::Mutex;
use serde_json::Value;
use std::{
    collections::VecDeque,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};
pub(super) struct InputState {
    pub(super) bytes: VecDeque<u8>,
    pub(super) ended: Option<bool>,
    pub(super) waker: Option<Waker>,
}
pub(super) struct Input(pub(super) Arc<Mutex<InputState>>);
impl AsyncRead for Input {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let mut state = self.0.lock();
        if !state.bytes.is_empty() {
            let count = state.bytes.len().min(buf.len());
            for byte in &mut buf[..count] {
                *byte = state.bytes.pop_front().unwrap();
            }
            return Poll::Ready(Ok(count));
        }
        match state.ended {
            Some(false) => Poll::Ready(Ok(0)),
            Some(true) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "actual module input termination",
            ))),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
pub(super) struct Output(pub(super) Arc<Mutex<Vec<u8>>>);
impl AsyncWrite for Output {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.lock().extend_from_slice(bytes);
        Poll::Ready(Ok(bytes.len()))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
pub(super) fn frame(value: &Value) -> Vec<u8> {
    let bytes = serde_json::to_vec(value).unwrap();
    let header = vize_l0::cstr!("Content-Length: {}\r\n\r\n", bytes.len());
    [header.as_bytes(), bytes.as_slice()].concat()
}
pub(super) fn frames(bytes: &[u8]) -> Vec<Value> {
    let mut bytes = bytes;
    let mut values = Vec::new();
    while !bytes.is_empty() {
        let end = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap();
        let count: usize = std::str::from_utf8(&bytes[..end])
            .unwrap()
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse()
            .unwrap();
        values.push(serde_json::from_slice(&bytes[end + 4..end + 4 + count]).unwrap());
        bytes = &bytes[end + 4 + count..];
    }
    values.sort_by_key(|value: &Value| value["id"].as_i64().unwrap_or(0));
    values
}
