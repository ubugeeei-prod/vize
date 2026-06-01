//! Minimal runtime utilities for the LSP server.
//!
//! This module intentionally stays tiny: a single-thread `block_on`, a shared
//! timer helper, and thread-backed adapters that let blocking stdio/TCP handles
//! satisfy `futures::io` traits without depending on Tokio.
#![allow(clippy::disallowed_types)]

use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;
use std::future::Future;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex, OnceLock, mpsc as std_mpsc};
use std::task::{Context, Poll, Waker, ready};
use std::thread;
use std::time::{Duration, Instant};

use futures::SinkExt;
use futures::channel::{mpsc, oneshot};
use futures::io::{AsyncRead, AsyncWrite};
use futures::stream::StreamExt;
use futures::task::{ArcWake, waker};
use vize_carton::{String, cstr};

const IO_CHANNEL_BOUND: usize = 16;

/// Runs a future to completion on the current thread.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    struct ThreadWaker {
        thread: thread::Thread,
    }

    impl ArcWake for ThreadWaker {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            arc_self.thread.unpark();
        }
    }

    let waker = waker(Arc::new(ThreadWaker {
        thread: thread::current(),
    }));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}

/// Error returned when a timeout expires before a future completes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TimeoutElapsed;

/// Resolves with `future`'s output, or `TimeoutElapsed` after `duration`.
pub async fn timeout<F>(duration: Duration, future: F) -> Result<F::Output, TimeoutElapsed>
where
    F: Future,
{
    let sleep = sleep(duration);

    futures::pin_mut!(future);
    futures::pin_mut!(sleep);

    futures::future::poll_fn(|cx| {
        if let Poll::Ready(output) = future.as_mut().poll(cx) {
            return Poll::Ready(Ok(output));
        }

        match sleep.as_mut().poll(cx) {
            Poll::Ready(()) => Poll::Ready(Err(TimeoutElapsed)),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

struct TimerState {
    fired: AtomicBool,
    cancelled: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

struct TimerRequest {
    deadline: Instant,
    sequence: u64,
    state: Arc<TimerState>,
}

struct TimerEntry(TimerRequest);

impl Eq for TimerEntry {}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.deadline == other.0.deadline && self.0.sequence == other.0.sequence
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        other
            .0
            .deadline
            .cmp(&self.0.deadline)
            .then_with(|| other.0.sequence.cmp(&self.0.sequence))
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

struct Sleep {
    state: Arc<TimerState>,
}

fn sleep(duration: Duration) -> Sleep {
    static TIMER_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let state = Arc::new(TimerState {
        fired: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
        waker: Mutex::new(None),
    });
    let request = TimerRequest {
        deadline: Instant::now() + duration,
        sequence: TIMER_SEQUENCE.fetch_add(1, AtomicOrdering::Relaxed),
        state: state.clone(),
    };

    if timer_sender().send(request).is_err() {
        state.fired.store(true, AtomicOrdering::Release);
    }

    Sleep { state }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.fired.load(AtomicOrdering::Acquire) {
            return Poll::Ready(());
        }

        if let Ok(mut waker) = self.state.waker.lock() {
            *waker = Some(cx.waker().clone());
        }

        if self.state.fired.load(AtomicOrdering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        self.state.cancelled.store(true, AtomicOrdering::Release);
        if let Ok(mut waker) = self.state.waker.lock() {
            *waker = None;
        }
    }
}

fn timer_sender() -> &'static std_mpsc::Sender<TimerRequest> {
    static TIMER_SENDER: OnceLock<std_mpsc::Sender<TimerRequest>> = OnceLock::new();

    TIMER_SENDER.get_or_init(|| {
        let (tx, rx) = std_mpsc::channel();
        let _ = thread::Builder::new()
            .name(std::string::String::from("vize-timer"))
            .spawn(move || timer_thread(rx));
        tx
    })
}

fn timer_thread(rx: std_mpsc::Receiver<TimerRequest>) {
    let mut heap = BinaryHeap::new();

    loop {
        let now = Instant::now();
        while heap
            .peek()
            .is_some_and(|entry: &TimerEntry| entry.0.deadline <= now)
        {
            if let Some(TimerEntry(request)) = heap.pop() {
                fire_timer(request);
            }
        }

        let wait = heap
            .peek()
            .map(|entry: &TimerEntry| entry.0.deadline.saturating_duration_since(Instant::now()));

        let received = match wait {
            Some(wait) => rx.recv_timeout(wait),
            None => rx
                .recv()
                .map_err(|_| std_mpsc::RecvTimeoutError::Disconnected),
        };

        match received {
            Ok(request) => heap.push(TimerEntry(request)),
            Err(std_mpsc::RecvTimeoutError::Timeout) => {}
            Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn fire_timer(request: TimerRequest) {
    if request.state.cancelled.load(AtomicOrdering::Acquire) {
        return;
    }

    request.state.fired.store(true, AtomicOrdering::Release);
    if let Ok(mut waker) = request.state.waker.lock()
        && let Some(waker) = waker.take()
    {
        waker.wake();
    }
}

enum ReadChunk {
    Data(Vec<u8>),
    Error(io::Error),
}

/// AsyncRead adapter backed by a blocking reader thread.
pub struct ThreadedReader {
    rx: mpsc::Receiver<ReadChunk>,
    pending: Vec<u8>,
    offset: usize,
}

impl ThreadedReader {
    fn new(rx: mpsc::Receiver<ReadChunk>) -> Self {
        Self {
            rx,
            pending: Vec::new(),
            offset: 0,
        }
    }
}

/// Wraps a blocking reader as a `futures::io::AsyncRead`.
pub fn threaded_reader<R>(name: &str, mut reader: R) -> io::Result<ThreadedReader>
where
    R: Read + Send + 'static,
{
    let (mut tx, rx) = mpsc::channel(IO_CHANNEL_BOUND);
    thread::Builder::new()
        .name(std::string::String::from(name))
        .spawn(move || {
            let mut buffer = [0; 8192];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(len) => {
                        if futures::executor::block_on(
                            tx.send(ReadChunk::Data(buffer[..len].to_vec())),
                        )
                        .is_err()
                        {
                            break;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => {
                        let _ = futures::executor::block_on(tx.send(ReadChunk::Error(error)));
                        break;
                    }
                }
            }
        })?;

    Ok(ThreadedReader::new(rx))
}

impl AsyncRead for ThreadedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        out: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if out.is_empty() {
            return Poll::Ready(Ok(0));
        }

        loop {
            if self.offset < self.pending.len() {
                let len = (self.pending.len() - self.offset).min(out.len());
                out[..len].copy_from_slice(&self.pending[self.offset..self.offset + len]);
                self.offset += len;

                if self.offset == self.pending.len() {
                    self.pending.clear();
                    self.offset = 0;
                }

                return Poll::Ready(Ok(len));
            }

            match self.rx.poll_next_unpin(cx) {
                Poll::Ready(Some(ReadChunk::Data(data))) => {
                    if !data.is_empty() {
                        self.pending = data;
                        self.offset = 0;
                    }
                }
                Poll::Ready(Some(ReadChunk::Error(error))) => {
                    return Poll::Ready(Err(error));
                }
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

enum WriteCommand {
    Write(Vec<u8>),
    Flush(oneshot::Sender<Result<(), String>>),
}

/// AsyncWrite adapter backed by a blocking writer thread.
pub struct ThreadedWriter {
    tx: std_mpsc::SyncSender<WriteCommand>,
    wake: Arc<WriterWake>,
    pending_flush: Option<oneshot::Receiver<Result<(), String>>>,
}

struct WriterWake {
    waker: Mutex<Option<Waker>>,
}

impl WriterWake {
    fn register(&self, cx: &Context<'_>) {
        if let Ok(mut waker) = self.waker.lock() {
            *waker = Some(cx.waker().clone());
        }
    }

    fn clear(&self) {
        if let Ok(mut waker) = self.waker.lock() {
            *waker = None;
        }
    }

    fn wake(&self) {
        if let Ok(mut waker) = self.waker.lock()
            && let Some(waker) = waker.take()
        {
            waker.wake();
        }
    }
}

impl ThreadedWriter {
    fn poll_pending_flush(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let Some(receiver) = self.pending_flush.as_mut() else {
            return Poll::Ready(Ok(()));
        };

        match Pin::new(receiver).poll(cx) {
            Poll::Ready(Ok(Ok(()))) => {
                self.pending_flush = None;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Ok(Err(error))) => {
                self.pending_flush = None;
                Poll::Ready(Err(io::Error::other(std::string::String::from(
                    error.as_str(),
                ))))
            }
            Poll::Ready(Err(_)) => {
                self.pending_flush = None;
                Poll::Ready(Err(writer_stopped()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Wraps a blocking writer as a `futures::io::AsyncWrite`.
pub fn threaded_writer<W>(name: &str, mut writer: W) -> io::Result<ThreadedWriter>
where
    W: Write + Send + 'static,
{
    let (tx, rx) = std_mpsc::sync_channel(IO_CHANNEL_BOUND);
    let wake = Arc::new(WriterWake {
        waker: Mutex::new(None),
    });
    let thread_wake = wake.clone();

    thread::Builder::new()
        .name(std::string::String::from(name))
        .spawn(move || {
            let mut failure: Option<String> = None;

            while let Ok(command) = rx.recv() {
                thread_wake.wake();

                match command {
                    WriteCommand::Write(bytes) => {
                        if failure.is_none()
                            && let Err(error) = writer.write_all(&bytes)
                        {
                            failure = Some(cstr!("{error}"));
                        }
                    }
                    WriteCommand::Flush(reply) => {
                        let result = if let Some(error) = failure.as_ref() {
                            Err(error.clone())
                        } else {
                            writer.flush().map_err(|error| {
                                let message = cstr!("{error}");
                                failure = Some(message.clone());
                                message
                            })
                        };
                        let _ = reply.send(result);
                    }
                }
            }

            let _ = writer.flush();
        })?;

    Ok(ThreadedWriter {
        tx,
        wake,
        pending_flush: None,
    })
}

impl AsyncWrite for ThreadedWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        ready!(self.poll_pending_flush(cx))?;

        let len = buf.len();
        self.wake.register(cx);
        match self.tx.try_send(WriteCommand::Write(buf.to_vec())) {
            Ok(()) => {
                self.wake.clear();
                Poll::Ready(Ok(len))
            }
            Err(std_mpsc::TrySendError::Full(_)) => Poll::Pending,
            Err(std_mpsc::TrySendError::Disconnected(_)) => {
                self.wake.clear();
                Poll::Ready(Err(writer_stopped()))
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.pending_flush.is_none() {
            let (tx, rx) = oneshot::channel();
            self.wake.register(cx);

            match self.tx.try_send(WriteCommand::Flush(tx)) {
                Ok(()) => {
                    self.wake.clear();
                    self.pending_flush = Some(rx);
                }
                Err(std_mpsc::TrySendError::Full(_)) => return Poll::Pending,
                Err(std_mpsc::TrySendError::Disconnected(_)) => {
                    self.wake.clear();
                    return Poll::Ready(Err(writer_stopped()));
                }
            }
        }

        self.poll_pending_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}

fn writer_stopped() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "writer thread stopped")
}

/// Accept a TCP connection without blocking the current async executor thread.
pub async fn accept_tcp(name: &str, listener: TcpListener) -> io::Result<(TcpStream, SocketAddr)> {
    let (tx, rx) = oneshot::channel();
    thread::Builder::new()
        .name(std::string::String::from(name))
        .spawn(move || {
            let _ = tx.send(listener.accept());
        })?;

    rx.await.map_err(|_| {
        io::Error::new(
            io::ErrorKind::BrokenPipe,
            "tcp accept thread stopped before accepting a connection",
        )
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc as std_mpsc;

    use futures::task::noop_waker;

    struct BlockingFlushWriter {
        writes: std_mpsc::Sender<Vec<u8>>,
        flush_started: std_mpsc::Sender<()>,
        flush_continue: std_mpsc::Receiver<()>,
    }

    impl Write for BlockingFlushWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.writes
                .send(buf.to_vec())
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "write log closed"))?;
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flush_started
                .send(())
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "flush log closed"))?;
            self.flush_continue
                .recv()
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "flush gate closed"))?;
            Ok(())
        }
    }

    #[test]
    fn threaded_writer_waits_for_pending_flush_before_accepting_more_writes() {
        let (writes_tx, writes_rx) = std_mpsc::channel();
        let (flush_started_tx, flush_started_rx) = std_mpsc::channel();
        let (flush_continue_tx, flush_continue_rx) = std_mpsc::channel();
        let blocking_writer = BlockingFlushWriter {
            writes: writes_tx,
            flush_started: flush_started_tx,
            flush_continue: flush_continue_rx,
        };
        let mut writer = threaded_writer("vize-test-writer", blocking_writer).unwrap();
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        assert!(matches!(
            Pin::new(&mut writer).poll_write(&mut cx, b"first"),
            Poll::Ready(Ok(5))
        ));
        assert!(matches!(
            Pin::new(&mut writer).poll_flush(&mut cx),
            Poll::Pending
        ));
        assert_eq!(
            writes_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            b"first"
        );
        flush_started_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap();

        assert!(matches!(
            Pin::new(&mut writer).poll_write(&mut cx, b"second"),
            Poll::Pending
        ));
        assert!(writes_rx.try_recv().is_err());

        flush_continue_tx.send(()).unwrap();
        futures::executor::block_on(futures::future::poll_fn(|cx| {
            Pin::new(&mut writer).poll_flush(cx)
        }))
        .unwrap();

        assert!(matches!(
            Pin::new(&mut writer).poll_write(&mut cx, b"second"),
            Poll::Ready(Ok(6))
        ));
        assert_eq!(
            writes_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            b"second"
        );

        drop(writer);
        flush_started_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        flush_continue_tx.send(()).unwrap();
    }
}
