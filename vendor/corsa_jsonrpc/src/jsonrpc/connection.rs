use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, FastMap};
use corsa_core::{CorsaEvent, SharedObserver, fast::compact_format, observe};
use corsa_runtime::{BroadcastReceiver, BroadcastSender, broadcast};
use log::warn;
use parking_lot::Mutex;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    io::{BufRead, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc::{self, TrySendError},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, Ordering},
    },
    thread,
    time::Duration,
};

use super::{
    RequestId,
    frame::{read_frame, write_frame},
    message::{MessageKind, RawMessage, RpcResponseError},
};

const DEFAULT_READER_JOIN_TIMEOUT: Duration = Duration::from_millis(250);
const JSONRPC_METHOD_NOT_FOUND: i64 = -32601;
const JSONRPC_INTERNAL_ERROR: i64 = -32603;
type ReaderResult = thread::Result<()>;

/// Locally registered callback for a JSON-RPC method.
///
/// The handler receives raw JSON parameters and returns either raw JSON result
/// data or a JSON-RPC error payload.
pub type RpcHandler =
    Arc<dyn Fn(Value) -> std::result::Result<Value, RpcResponseError> + Send + Sync + 'static>;
/// Handler map keyed by JSON-RPC method name.
pub type RpcHandlerMap = FastMap<CompactString, RpcHandler>;

/// Inbound JSON-RPC events that are not handled locally.
#[derive(Clone, Debug)]
pub enum InboundEvent {
    /// Request that did not match a locally registered handler.
    Request {
        /// Request identifier to answer with [`JsonRpcConnection::respond`].
        id: RequestId,
        /// JSON-RPC method name.
        method: CompactString,
        /// Raw JSON parameters payload.
        params: Value,
    },
    /// Notification that did not match a locally registered handler.
    Notification {
        /// JSON-RPC method name.
        method: CompactString,
        /// Raw JSON parameters payload.
        params: Value,
    },
}

/// Runtime policy applied to a [`JsonRpcConnection`].
#[derive(Clone)]
pub struct JsonRpcConnectionOptions {
    /// Maximum time to wait for a response before surfacing a timeout.
    pub request_timeout: Option<Duration>,
    /// Maximum number of queued outbound messages waiting on the writer thread.
    pub outbound_capacity: usize,
    /// Optional observer for structured transport events.
    pub observer: Option<SharedObserver>,
}

impl JsonRpcConnectionOptions {
    /// Creates the default production-oriented JSON-RPC transport policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the per-request timeout.
    pub fn with_request_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Sets the maximum number of queued outbound messages.
    pub fn with_outbound_capacity(mut self, capacity: usize) -> Self {
        self.outbound_capacity = capacity.max(1);
        self
    }

    /// Sets the observer used for structured transport events.
    pub fn with_observer(mut self, observer: SharedObserver) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Sets the observer when one is available.
    pub fn with_observer_if_some(mut self, observer: Option<SharedObserver>) -> Self {
        self.observer = observer;
        self
    }
}

impl Default for JsonRpcConnectionOptions {
    fn default() -> Self {
        Self {
            request_timeout: Some(Duration::from_secs(30)),
            outbound_capacity: 256,
            observer: None,
        }
    }
}

impl std::fmt::Debug for JsonRpcConnectionOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JsonRpcConnectionOptions")
            .field("request_timeout", &self.request_timeout)
            .field("outbound_capacity", &self.outbound_capacity)
            .field("observer", &self.observer.is_some())
            .finish()
    }
}

/// Thread-backed stdio JSON-RPC connection.
///
/// The connection owns a reader thread, a writer thread, and a pending-request
/// table. Requests and notifications are serialized on the writer thread, while
/// inbound frames are decoded on the reader thread.
///
/// Local handlers are useful for request/notification callbacks such as the
/// filesystem bridge used by the Corsa API client.
#[derive(Clone)]
pub struct JsonRpcConnection {
    inner: Arc<Inner>,
}

struct Inner {
    closed: AtomicBool,
    next_id: AtomicI64,
    request_timeout: Option<Duration>,
    observer: Option<SharedObserver>,
    events: BroadcastSender<InboundEvent>,
    handlers: RpcHandlerMap,
    outbound: Mutex<Option<mpsc::SyncSender<RawMessage>>>,
    pending: Mutex<FastMap<RequestId, mpsc::SyncSender<Result<Value>>>>,
    read_task: Mutex<Option<thread::JoinHandle<()>>>,
    read_done: Mutex<Option<mpsc::Receiver<ReaderResult>>>,
    write_task: Mutex<Option<thread::JoinHandle<()>>>,
}

impl JsonRpcConnection {
    /// Spawns a connection around a buffered reader and writer.
    ///
    /// The connection starts background reader/writer threads immediately.
    #[track_caller]
    pub fn spawn<R, W>(reader: R, writer: W, handlers: RpcHandlerMap) -> Self
    where
        R: BufRead + Send + 'static,
        W: Write + Send + 'static,
    {
        match Self::try_spawn(reader, writer, handlers) {
            Ok(connection) => connection,
            Err(error) => panic!(
                "failed to spawn jsonrpc connection threads: {error}. use `JsonRpcConnection::try_spawn` to handle this failure without panicking"
            ),
        }
    }

    /// Tries to spawn a connection around a buffered reader and writer.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if an operating-system thread cannot be created.
    pub fn try_spawn<R, W>(reader: R, writer: W, handlers: RpcHandlerMap) -> Result<Self>
    where
        R: BufRead + Send + 'static,
        W: Write + Send + 'static,
    {
        Self::try_spawn_with_options(
            reader,
            writer,
            handlers,
            JsonRpcConnectionOptions::default(),
        )
    }

    /// Spawns a connection around a buffered reader and writer with runtime options.
    #[track_caller]
    pub fn spawn_with_options<R, W>(
        reader: R,
        writer: W,
        handlers: RpcHandlerMap,
        options: JsonRpcConnectionOptions,
    ) -> Self
    where
        R: BufRead + Send + 'static,
        W: Write + Send + 'static,
    {
        match Self::try_spawn_with_options(reader, writer, handlers, options) {
            Ok(connection) => connection,
            Err(error) => panic!(
                "failed to spawn jsonrpc connection threads: {error}. use `JsonRpcConnection::try_spawn_with_options` to handle this failure without panicking"
            ),
        }
    }

    /// Tries to spawn a connection around a buffered reader and writer with runtime options.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if an operating-system thread cannot be created.
    pub fn try_spawn_with_options<R, W>(
        reader: R,
        writer: W,
        handlers: RpcHandlerMap,
        options: JsonRpcConnectionOptions,
    ) -> Result<Self>
    where
        R: BufRead + Send + 'static,
        W: Write + Send + 'static,
    {
        let (outbound_tx, outbound_rx) = mpsc::sync_channel(options.outbound_capacity.max(1));
        let (events, _) = broadcast();
        let (read_done_tx, read_done_rx) = mpsc::sync_channel(1);
        let inner = Arc::new(Inner {
            closed: AtomicBool::new(false),
            next_id: AtomicI64::new(0),
            request_timeout: options.request_timeout,
            observer: options.observer.clone(),
            events,
            handlers,
            outbound: Mutex::new(Some(outbound_tx)),
            pending: Mutex::new(FastMap::default()),
            read_task: Mutex::new(None),
            read_done: Mutex::new(Some(read_done_rx)),
            write_task: Mutex::new(None),
        });
        let write_inner = Arc::clone(&inner);
        let write_task = thread::Builder::new()
            .name("corsa-jsonrpc-writer".into())
            .spawn(move || {
                write_inner.write_loop(writer, outbound_rx);
            })
            .map_err(CorsaError::Io)?;
        *inner.write_task.lock() = Some(write_task);

        let read_inner = Arc::clone(&inner);
        let read_task = match thread::Builder::new()
            .name("corsa-jsonrpc-reader".into())
            .spawn(move || {
                let panic_inner = Arc::clone(&read_inner);
                let result = catch_unwind(AssertUnwindSafe(|| read_inner.read_loop(reader)));
                if result.is_err() {
                    panic_inner.fail_transport(CorsaError::Join(CompactString::from(
                        "jsonrpc reader thread panicked",
                    )));
                }
                let _ = read_done_tx.send(result);
            }) {
            Ok(task) => task,
            Err(error) => {
                inner.closed.store(true, Ordering::SeqCst);
                inner.outbound.lock().take();
                if let Some(task) = inner.write_task.lock().take() {
                    let _ = task.join();
                }
                return Err(CorsaError::Io(error));
            }
        };
        *inner.read_task.lock() = Some(read_task);
        Ok(Self { inner })
    }

    /// Subscribes to inbound events that are not matched by local handlers.
    pub fn subscribe(&self) -> BroadcastReceiver<InboundEvent> {
        self.inner.events.subscribe()
    }

    /// Sends a JSON-RPC request and deserializes the response.
    ///
    /// Each request is assigned a monotonically increasing integer ID.
    pub async fn request<Params, Response>(&self, method: &str, params: Params) -> Result<Response>
    where
        Params: Serialize,
        Response: DeserializeOwned,
    {
        let value = serde_json::to_value(params)?;
        let value = self.request_value(method, value).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Sends a JSON-RPC request with the `params` member omitted.
    ///
    /// This is distinct from passing `()` or [`Value::Null`], both of which
    /// serialize as `"params": null`.
    pub async fn request_without_params<Response>(&self, method: &str) -> Result<Response>
    where
        Response: DeserializeOwned,
    {
        let value = self.request_optional_value(method, None).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Sends a JSON-RPC request and returns the raw JSON value.
    ///
    /// Use this when a typed response model is not available yet.
    pub async fn request_value(&self, method: &str, params: Value) -> Result<Value> {
        self.request_optional_value(method, Some(params)).await
    }

    async fn request_optional_value(&self, method: &str, params: Option<Value>) -> Result<Value> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(CorsaError::Closed("jsonrpc connection"));
        }
        let id = RequestId::integer(self.inner.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        let (tx, rx) = mpsc::sync_channel(1);
        self.inner.pending.lock().insert(id.clone(), tx);
        let message = match params {
            Some(params) => RawMessage::request(id.clone(), CompactString::from(method), params),
            None => RawMessage::request_without_params(id.clone(), CompactString::from(method)),
        };
        if let Err(error) = self.inner.send_outbound(message) {
            self.inner.pending.lock().remove(&id);
            return Err(error);
        }
        if let Some(timeout) = self.inner.request_timeout {
            return rx.recv_timeout(timeout).map_err(|_| {
                self.inner.pending.lock().remove(&id);
                observe(
                    self.inner.observer.as_ref(),
                    CorsaEvent::JsonRpcRequestTimedOut {
                        method: CompactString::from(method),
                        timeout,
                    },
                );
                CorsaError::timeout(
                    compact_format(format_args!("jsonrpc request `{method}`")).as_str(),
                    timeout,
                )
            })?;
        }
        rx.recv()
            .map_err(|_| CorsaError::Closed("jsonrpc waiter"))?
    }

    /// Sends a JSON-RPC notification.
    pub fn notify<Params>(&self, method: &str, params: Params) -> Result<()>
    where
        Params: Serialize,
    {
        let params = serde_json::to_value(params)?;
        self.inner.send_outbound(RawMessage::notification(
            CompactString::from(method),
            params,
        ))?;
        Ok(())
    }

    /// Sends a JSON-RPC notification with the `params` member omitted.
    ///
    /// This is distinct from passing `()` or [`Value::Null`], both of which
    /// serialize as `"params": null`.
    pub fn notify_without_params(&self, method: &str) -> Result<()> {
        self.inner
            .send_outbound(RawMessage::notification_without_params(
                CompactString::from(method),
            ))?;
        Ok(())
    }

    /// Sends a successful response for an inbound request.
    pub fn respond<ResultBody>(&self, id: RequestId, body: ResultBody) -> Result<()>
    where
        ResultBody: Serialize,
    {
        let result = serde_json::to_value(body)?;
        self.inner.send_outbound(RawMessage::response(id, result))?;
        Ok(())
    }

    /// Sends an error response for an inbound request.
    pub fn respond_error(&self, id: RequestId, error: RpcResponseError) -> Result<()> {
        self.inner.send_outbound(RawMessage::error(id, error))?;
        Ok(())
    }

    /// Closes the connection and fails outstanding requests.
    ///
    /// After closure, new requests fail immediately with [`CorsaError::Closed`].
    pub async fn close(&self) -> Result<()> {
        self.begin_close()?;
        self.join_reader(DEFAULT_READER_JOIN_TIMEOUT)
    }

    /// Signals writer shutdown and fails outstanding requests without waiting for the reader.
    pub fn begin_close(&self) -> Result<()> {
        if !self.inner.closed.swap(true, Ordering::SeqCst) {
            self.inner
                .fail_pending(CorsaError::Closed("jsonrpc connection"));
        }
        self.inner.outbound.lock().take();
        if let Some(task) = self.inner.write_task.lock().take() {
            let _ = task.join();
        }
        Ok(())
    }

    /// Joins the reader thread, falling back after `timeout` if the transport stays blocked.
    pub fn join_reader(&self, timeout: Duration) -> Result<()> {
        self.inner.join_reader(timeout)
    }
}

impl Inner {
    fn join_reader(&self, timeout: Duration) -> Result<()> {
        let done = self.read_done.lock();
        let Some(done_rx) = done.as_ref() else {
            return Ok(());
        };
        match done_rx.recv_timeout(timeout) {
            Ok(result) => {
                drop(done);
                self.read_done.lock().take();
                if let Some(task) = self.read_task.lock().take() {
                    let _ = task.join();
                }
                result.map_err(|_| {
                    CorsaError::Join(CompactString::from("jsonrpc reader thread panicked"))
                })
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                warn!(
                    "jsonrpc reader thread did not stop within {} ms; continuing shutdown",
                    timeout.as_millis()
                );
                Ok(())
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(CorsaError::Closed("jsonrpc reader")),
        }
    }

    fn read_loop<R>(self: Arc<Self>, mut reader: R)
    where
        R: BufRead + Send + 'static,
    {
        loop {
            let payload = match read_frame(&mut reader) {
                Ok(payload) => payload,
                Err(err) => {
                    self.fail_transport(err);
                    return;
                }
            };
            let message: RawMessage = match serde_json::from_slice(&payload) {
                Ok(message) => message,
                Err(err) => {
                    self.fail_transport(err.into());
                    return;
                }
            };
            match message.into_kind() {
                Ok(MessageKind::Response { id, result, error }) => {
                    if let Some(tx) = self.pending.lock().remove(&id) {
                        let _ = tx.send(match error {
                            Some(error) => Err(CorsaError::Rpc(error)),
                            None => Ok(result.unwrap_or(Value::Null)),
                        });
                    }
                }
                Ok(MessageKind::Request { id, method, params }) => {
                    if let Some(handler) = self.handlers.get(method.as_str()) {
                        let response = self.invoke_handler(method.as_str(), handler, params);
                        let message = match response {
                            Ok(value) => RawMessage::response(id, value),
                            Err(error) => RawMessage::error(id, error),
                        };
                        let _ = self.send_outbound(message);
                    } else {
                        let delivered = self.events.send(InboundEvent::Request {
                            id: id.clone(),
                            method: method.clone(),
                            params,
                        });
                        if delivered == 0 {
                            let _ = self
                                .send_outbound(RawMessage::error(id, method_not_found(&method)));
                        }
                    }
                }
                Ok(MessageKind::Notification { method, params }) => {
                    if let Some(handler) = self.handlers.get(method.as_str()) {
                        let _ = self.invoke_handler(method.as_str(), handler, params);
                    } else {
                        let _ = self
                            .events
                            .send(InboundEvent::Notification { method, params });
                    }
                }
                Err(err) => {
                    self.fail_transport(err);
                    return;
                }
            }
        }
    }

    fn write_loop<W>(self: Arc<Self>, mut writer: W, outbound_rx: mpsc::Receiver<RawMessage>)
    where
        W: Write + Send + 'static,
    {
        while let Ok(message) = outbound_rx.recv() {
            let body = match serde_json::to_vec(&message) {
                Ok(body) => body,
                Err(err) => {
                    self.fail_transport(err.into());
                    return;
                }
            };
            if let Err(err) = write_frame(&mut writer, &body) {
                self.fail_transport(err);
                return;
            }
        }
    }

    fn send_outbound(&self, message: RawMessage) -> Result<()> {
        match self
            .outbound
            .lock()
            .as_ref()
            .ok_or(CorsaError::Closed("jsonrpc writer"))?
            .try_send(message)
        {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                observe(self.observer.as_ref(), CorsaEvent::JsonRpcOutboundQueueFull);
                Err(CorsaError::Protocol(
                    "jsonrpc outbound queue is full".into(),
                ))
            }
            Err(TrySendError::Disconnected(_)) => Err(CorsaError::Closed("jsonrpc writer")),
        }
    }

    fn invoke_handler(
        &self,
        method: &str,
        handler: &RpcHandler,
        params: Value,
    ) -> std::result::Result<Value, RpcResponseError> {
        match catch_unwind(AssertUnwindSafe(|| (handler)(params))) {
            Ok(result) => result,
            Err(_) => {
                warn!("jsonrpc local handler `{method}` panicked");
                observe(
                    self.observer.as_ref(),
                    CorsaEvent::JsonRpcLocalHandlerPanicked {
                        method: CompactString::from(method),
                    },
                );
                Err(handler_panic_error(method))
            }
        }
    }

    fn fail_transport(&self, error: CorsaError) {
        self.closed.store(true, Ordering::SeqCst);
        self.outbound.lock().take();
        self.fail_pending(error);
    }

    fn fail_pending(&self, error: CorsaError) {
        if !matches!(error, CorsaError::Closed(_)) {
            warn!("jsonrpc transport failing pending requests: {error}");
        }
        let mut pending = self.pending.lock();
        let count = pending.len();
        if count > 0 {
            observe(
                self.observer.as_ref(),
                CorsaEvent::JsonRpcPendingRequestsFailed {
                    error: CompactString::from(error.to_string().as_str()),
                    count,
                },
            );
        }
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(error.clone_for_pending()));
        }
    }
}

fn method_not_found(method: &str) -> RpcResponseError {
    RpcResponseError {
        code: JSONRPC_METHOD_NOT_FOUND,
        message: compact_format(format_args!("method not found: {method}")),
        data: None,
    }
}

fn handler_panic_error(method: &str) -> RpcResponseError {
    RpcResponseError {
        code: JSONRPC_INTERNAL_ERROR,
        message: compact_format(format_args!("local JSON-RPC handler `{method}` panicked")),
        data: None,
    }
}
