//! Dedicated worker thread for the sync msgpack transport.
//!
//! The msgpack protocol is synchronous over stdio: a request is written, and
//! the next relevant tuple on stdout is treated as the response. This module
//! wraps that blocking interaction in a worker thread so the public API can stay
//! async-friendly without pulling in a full async runtime.

use crate::{CorsaError, Result};
use corsa_core::{
    AsyncChildGuard, CorsaEvent, SharedObserver,
    fast::{CompactString, SmallVec, compact_format},
    observe,
};
use log::warn;
use parking_lot::Mutex;
use std::{
    io::{BufReader, BufWriter},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use super::{
    callbacks::{ApiFileSystem, invoke_callback},
    msgpack_codec::{
        MSG_CALL, MSG_CALL_ERROR, MSG_CALL_RESPONSE, MSG_ERROR, MSG_REQUEST, MSG_RESPONSE,
        MsgpackTuple, read_tuple, write_tuple,
    },
};

/// Thread-backed msgpack transport worker.
///
/// Requests are serialized through a single worker thread because the
/// underlying stdio protocol is strictly ordered.
pub(crate) struct MsgpackWorker {
    tx: Mutex<Option<mpsc::SyncSender<WorkerCommand>>>,
    join: Mutex<Option<thread::JoinHandle<()>>>,
    process: Arc<AsyncChildGuard>,
    request_timeout: Option<Duration>,
    observer: Option<SharedObserver>,
    terminated: AtomicBool,
}

/// Successful response returned from the worker thread.
pub(crate) struct WorkerResponse {
    pub bytes: Vec<u8>,
}

/// Commands sent to the worker thread.
enum WorkerCommand {
    Request {
        method: SmallVec<[u8; 32]>,
        payload: Vec<u8>,
        reply: mpsc::SyncSender<Result<WorkerResponse>>,
    },
    Shutdown,
}

impl MsgpackWorker {
    pub(crate) fn spawn(
        child: corsa_core::OwnedProcess,
        filesystem: Option<Arc<dyn ApiFileSystem>>,
        request_timeout: Option<Duration>,
        queue_capacity: usize,
        observer: Option<SharedObserver>,
    ) -> Result<Self> {
        let corsa_core::OwnedProcess {
            stdin,
            stdout,
            process,
        } = child;
        let worker_process = process.clone();
        let (tx, rx) = mpsc::sync_channel::<WorkerCommand>(queue_capacity.max(1));
        let join = thread::Builder::new()
            .name("corsa-msgpack-worker".into())
            .spawn(move || {
                let mut writer = BufWriter::new(stdin);
                let mut reader = BufReader::new(stdout);
                while let Ok(command) = rx.recv() {
                    match command {
                        WorkerCommand::Request {
                            method,
                            payload,
                            reply,
                        } => {
                            let result = write_tuple(&mut writer, MSG_REQUEST, &method, &payload)
                                .and_then(|_| {
                                    read_response(
                                        &mut reader,
                                        &mut writer,
                                        &method,
                                        filesystem.as_deref(),
                                    )
                                });
                            let _ = reply.send(result.map(|bytes| WorkerResponse { bytes }));
                        }
                        WorkerCommand::Shutdown => break,
                    }
                }
                let _ = worker_process.abort();
            })
            .map_err(|error| {
                process
                    .terminal_observer()
                    .record_error(format!("msgpack worker setup failed: {error}"));
                CorsaError::Io(error)
            })?;
        Ok(Self {
            tx: Mutex::new(Some(tx)),
            join: Mutex::new(Some(join)),
            process,
            request_timeout,
            observer,
            terminated: AtomicBool::new(false),
        })
    }

    pub(crate) async fn request(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        // A sync channel keeps request/response pairing simple: the caller
        // blocks until the worker thread has seen the matching response tuple.
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let sender = self
            .tx
            .lock()
            .clone()
            .ok_or(CorsaError::Closed("msgpack worker"))?;
        match sender.try_send(WorkerCommand::Request {
            method: SmallVec::from_slice(method.as_bytes()),
            payload,
            reply: reply_tx,
        }) {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(_)) => {
                observe(
                    self.observer.as_ref(),
                    CorsaEvent::MsgpackWorkerQueueFull {
                        method: CompactString::from(method),
                    },
                );
                return Err(CorsaError::Protocol("msgpack worker queue is full".into()));
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                return Err(CorsaError::Closed("msgpack worker"));
            }
        }
        let response = if let Some(timeout) = self.request_timeout {
            reply_rx.recv_timeout(timeout).map_err(|_| {
                warn!("msgpack request `{method}` timed out; terminating worker");
                observe(
                    self.observer.as_ref(),
                    CorsaEvent::MsgpackRequestTimedOut {
                        method: CompactString::from(method),
                        timeout,
                    },
                );
                if let Err(error) = self.terminate_process("request timeout") {
                    return CorsaError::Protocol(
                        format!(
                            "msgpack request `{method}` timed out; child abort failed: {error}"
                        )
                        .into(),
                    );
                }
                CorsaError::timeout(
                    compact_format(format_args!("msgpack request `{method}`")).as_str(),
                    timeout,
                )
            })??
        } else {
            reply_rx
                .recv()
                .map_err(|_| CorsaError::Closed("msgpack worker"))??
        };
        Ok(response.bytes)
    }

    pub(crate) fn process_ownership(&self) -> corsa_core::ProcessOwnership {
        corsa_core::ProcessOwnership::Owned(self.process.terminal_observer())
    }

    pub(crate) async fn close(&self) -> Result<()> {
        if let Some(sender) = self.tx.lock().take() {
            let _ = sender.try_send(WorkerCommand::Shutdown);
        }
        self.terminate_process("close")?;
        if let Some(join) = self.join.lock().take() {
            join.join()
                .map_err(|_| CorsaError::Join("msgpack worker".into()))?;
        }
        Ok(())
    }

    pub(crate) async fn abort_startup(&self) -> Result<Option<corsa_core::ProcessCloseReceipt>> {
        if let Some(sender) = self.tx.lock().take() {
            let _ = sender.try_send(WorkerCommand::Shutdown);
        }
        let receipt = self.process.abort_receipt()?;
        self.observe_termination("startup abort");
        let timeout = Duration::from_secs(2);
        let deadline = std::time::Instant::now() + timeout;
        let mut worker = self.join.lock();
        while worker.as_ref().is_some_and(|task| !task.is_finished()) {
            if std::time::Instant::now() >= deadline {
                return Err(CorsaError::timeout("msgpack startup worker close", timeout));
            }
            thread::sleep(Duration::from_millis(10));
        }
        if let Some(join) = worker.take() {
            join.join()
                .map_err(|_| CorsaError::Join("msgpack worker".into()))?;
        }
        Ok(receipt)
    }

    fn terminate_process(&self, reason: &str) -> Result<()> {
        self.process.abort()?;
        self.observe_termination(reason);
        Ok(())
    }

    fn observe_termination(&self, reason: &str) {
        if self.terminated.swap(true, Ordering::AcqRel) {
            return;
        }
        observe(
            self.observer.as_ref(),
            CorsaEvent::MsgpackWorkerTerminated {
                reason: CompactString::from(reason),
            },
        );
    }
}

/// Reads tuples until the matching response for `method` arrives.
///
/// Callback tuples are handled inline and may emit additional tuples on the
/// same stdio stream before the real response is observed.
fn read_response(
    reader: &mut BufReader<std::process::ChildStdout>,
    writer: &mut BufWriter<std::process::ChildStdin>,
    method: &[u8],
    filesystem: Option<&dyn ApiFileSystem>,
) -> Result<Vec<u8>> {
    loop {
        let message = read_tuple(reader)?;
        match message.kind {
            MSG_RESPONSE if message.method == method => return Ok(message.payload),
            MSG_ERROR if message.method == method => {
                return Err(CorsaError::Protocol(
                    String::from_utf8_lossy(&message.payload).into(),
                ));
            }
            // `corsa` can interleave filesystem callbacks while it computes the
            // request. Those callbacks must be answered before the final
            // response tuple can arrive.
            MSG_CALL => handle_callback(writer, filesystem, message)?,
            other => {
                return Err(CorsaError::UnexpectedMessage(compact_format(format_args!(
                    "msgpack type {other}"
                ))));
            }
        }
    }
}

/// Executes a filesystem callback received over the msgpack transport.
fn handle_callback(
    writer: &mut BufWriter<std::process::ChildStdin>,
    filesystem: Option<&dyn ApiFileSystem>,
    callback: MsgpackTuple,
) -> Result<()> {
    let method = std::str::from_utf8(&callback.method)
        .map_err(|_| CorsaError::Protocol("callback method must be utf-8".into()))?;
    let Some(filesystem) = filesystem else {
        return write_tuple(
            writer,
            MSG_CALL_ERROR,
            method.as_bytes(),
            b"no filesystem callbacks",
        );
    };
    let payload: serde_json::Value = serde_json::from_slice(&callback.payload)?;
    let value = match invoke_callback(filesystem, method, &payload) {
        Ok(value) => value,
        Err(error) => {
            return write_tuple(
                writer,
                MSG_CALL_ERROR,
                method.as_bytes(),
                error.message.as_bytes(),
            );
        }
    };
    // Callback results are encoded as JSON and then wrapped back into the
    // msgpack tuple protocol expected by `corsa`.
    match serde_json::to_vec(&value) {
        Ok(bytes) => write_tuple(writer, MSG_CALL_RESPONSE, method.as_bytes(), &bytes),
        Err(error) => write_tuple(
            writer,
            MSG_CALL_ERROR,
            method.as_bytes(),
            error.to_string().as_bytes(),
        ),
    }
}
