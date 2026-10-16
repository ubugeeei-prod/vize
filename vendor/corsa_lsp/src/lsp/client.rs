use crate::{
    CorsaError, Result,
    jsonrpc::{InboundEvent, JsonRpcConnection, JsonRpcConnectionOptions, RequestId},
    process::{AsyncChildGuard, CorsaCommand},
};
use corsa_core::{
    SharedObserver,
    fast::{CompactString, SmallVec, compact_format},
};
use corsa_runtime::BroadcastReceiver;
use lsp_types::{
    notification::{Exit, Notification},
    request::{Request, Shutdown},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use super::{
    InitializeApiSessionParams, InitializeApiSessionRequest, InitializeApiSessionResult, LspOverlay,
};

/// LSP client backed by the Corsa stdio server.
///
/// `LspClient` is the transport-facing half of editor-style workflows. It owns
/// the server process, provides typed request/notification helpers via
/// [`lsp_types`], and can create an [`LspOverlay`] for mirrored in-memory
/// document state.
#[derive(Clone)]
pub struct LspClient {
    rpc: JsonRpcConnection,
    process: Arc<AsyncChildGuard>,
    shutdown_timeout: Duration,
    shutdown: Arc<ShutdownCoordinator>,
}

#[derive(Default)]
struct ShutdownCoordinator {
    phase: Mutex<ShutdownPhase>,
    completed: Condvar,
}

#[derive(Default)]
enum ShutdownPhase {
    #[default]
    Open,
    Closing,
    Closed(Result<()>),
}

impl ShutdownCoordinator {
    fn begin(&self) -> Option<Result<()>> {
        let mut phase = self.phase.lock().unwrap_or_else(|error| error.into_inner());
        loop {
            match &*phase {
                ShutdownPhase::Open => {
                    *phase = ShutdownPhase::Closing;
                    return None;
                }
                ShutdownPhase::Closing => {
                    phase = self
                        .completed
                        .wait(phase)
                        .unwrap_or_else(|error| error.into_inner());
                }
                ShutdownPhase::Closed(result) => return Some(clone_result(result)),
            }
        }
    }

    fn finish(&self, result: &Result<()>) {
        let mut phase = self.phase.lock().unwrap_or_else(|error| error.into_inner());
        *phase = ShutdownPhase::Closed(clone_result(result));
        self.completed.notify_all();
    }
}

fn clone_result(result: &Result<()>) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(error.clone_for_pending()),
    }
}

/// Spawn-time options for [`LspClient`].
///
/// # Examples
///
/// ```
/// use corsa_lsp::LspSpawnConfig;
///
/// let config = LspSpawnConfig::new("/opt/bin/corsa")
///     .with_cwd("/workspace")
///     .with_arg("--logToFile");
///
/// assert_eq!(config.extra_args.as_slice(), &["--logToFile"]);
/// ```
#[derive(Clone)]
pub struct LspSpawnConfig {
    /// Reusable command template used to launch Corsa over LSP stdio.
    pub command: CorsaCommand,
    /// Additional CLI flags appended after `--lsp --stdio`.
    pub extra_args: SmallVec<[CompactString; 4]>,
    /// Maximum time to wait for a single request before surfacing a timeout.
    pub request_timeout: Option<Duration>,
    /// Maximum time to wait for process shutdown before force-killing the server.
    pub shutdown_timeout: Duration,
    /// Maximum number of queued outbound transport messages.
    pub outbound_capacity: usize,
    /// Optional observer for structured transport events.
    pub observer: Option<SharedObserver>,
}

impl std::fmt::Debug for LspSpawnConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LspSpawnConfig")
            .field("command", &self.command)
            .field("extra_args", &self.extra_args)
            .field("request_timeout", &self.request_timeout)
            .field("shutdown_timeout", &self.shutdown_timeout)
            .field("outbound_capacity", &self.outbound_capacity)
            .field("observer", &self.observer.is_some())
            .finish()
    }
}

impl LspSpawnConfig {
    /// Creates a new LSP spawn configuration.
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            command: CorsaCommand::new(executable),
            extra_args: SmallVec::new(),
            request_timeout: Some(Duration::from_secs(30)),
            shutdown_timeout: Duration::from_secs(2),
            outbound_capacity: 256,
            observer: None,
        }
    }

    /// Sets the server working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.command = self.command.clone().with_cwd(cwd);
        self
    }

    /// Appends an extra CLI argument passed to Corsa.
    pub fn with_arg(mut self, arg: impl Into<CompactString>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Sets the per-request timeout applied by the JSON-RPC transport.
    pub fn with_request_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Sets the graceful shutdown timeout used when closing the server.
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Sets the maximum number of queued outbound transport messages.
    pub fn with_outbound_capacity(mut self, capacity: usize) -> Self {
        self.outbound_capacity = capacity.max(1);
        self
    }

    /// Sets the observer used for structured transport events.
    pub fn with_observer(mut self, observer: SharedObserver) -> Self {
        self.observer = Some(observer);
        self
    }
}

impl LspClient {
    /// Spawns a Corsa LSP server over stdio.
    ///
    /// The client owns the spawned process and will terminate it when
    /// [`close`](Self::close) is called.
    pub async fn spawn(config: LspSpawnConfig) -> Result<Self> {
        let mut args = SmallVec::<[CompactString; 6]>::new();
        args.push(CompactString::from("--lsp"));
        args.push(CompactString::from("--stdio"));
        args.extend(config.extra_args);
        let corsa_core::OwnedProcess {
            stdin,
            stdout,
            process,
        } = config
            .command
            .spawn_guarded(args.iter().map(CompactString::as_str))?;
        let terminal = process.terminal_observer();
        Ok(Self {
            rpc: JsonRpcConnection::try_spawn_with_options(
                BufReader::new(stdout),
                stdin,
                Default::default(),
                JsonRpcConnectionOptions::new()
                    .with_request_timeout(config.request_timeout)
                    .with_outbound_capacity(config.outbound_capacity)
                    .with_observer_if_some(config.observer.clone()),
            )
            .map_err(|error| {
                terminal.record_error(
                    compact_format(format_args!("LSP JSON-RPC setup failed: {error}")).as_str(),
                );
                error
            })?,
            process,
            shutdown_timeout: config.shutdown_timeout,
            shutdown: Arc::new(ShutdownCoordinator::default()),
        })
    }

    /// Subscribes to inbound requests and notifications from the server.
    ///
    /// Only messages without a local handler are forwarded to subscribers.
    pub fn subscribe(&self) -> BroadcastReceiver<InboundEvent> {
        self.rpc.subscribe()
    }

    pub fn process_ownership(&self) -> corsa_core::ProcessOwnership {
        corsa_core::ProcessOwnership::Owned(self.process.terminal_observer())
    }

    /// Creates a virtual-document overlay synchronized with this client.
    ///
    /// The overlay helps keep `didOpen`, `didChange`, and `didClose`
    /// notifications consistent with local in-memory state.
    pub fn overlay(&self) -> LspOverlay {
        LspOverlay::new(self.clone())
    }

    /// Sends a typed LSP request.
    ///
    /// Request/response payloads are described by the [`lsp_types::request::Request`]
    /// implementation supplied as `R`.
    pub async fn request<R>(&self, params: R::Params) -> Result<R::Result>
    where
        R: Request,
        R::Params: Serialize,
        R::Result: DeserializeOwned,
    {
        self.rpc.request(R::METHOD, params).await
    }

    /// Sends a typed LSP request with the `params` member omitted.
    ///
    /// Use this for protocol methods such as [`Shutdown`] whose wire envelope
    /// must not conflate an absent member with `"params": null`.
    pub async fn request_without_params<R>(&self) -> Result<R::Result>
    where
        R: Request,
        R::Result: DeserializeOwned,
    {
        self.rpc.request_without_params(R::METHOD).await
    }

    /// Sends a typed LSP notification.
    ///
    /// This is the preferred way to emit standard protocol notifications such
    /// as `textDocument/didOpen`.
    pub fn notify<N>(&self, params: N::Params) -> Result<()>
    where
        N: Notification,
        N::Params: Serialize,
    {
        self.rpc.notify(N::METHOD, params)
    }

    /// Sends a typed LSP notification with the `params` member omitted.
    pub fn notify_without_params<N>(&self) -> Result<()>
    where
        N: Notification,
    {
        self.rpc.notify_without_params(N::METHOD)
    }

    /// Responds to an inbound request.
    ///
    /// Use this when consuming [`InboundEvent::Request`](crate::jsonrpc::InboundEvent::Request)
    /// from [`subscribe`](Self::subscribe).
    pub fn respond<ResultBody>(&self, id: RequestId, body: ResultBody) -> Result<()>
    where
        ResultBody: Serialize,
    {
        self.rpc.respond(id, body)
    }

    /// Calls the custom `initializeAPISession` request exposed by Corsa.
    ///
    /// This is useful when an LSP session needs to bootstrap a separate API
    /// session and exchange its pipe information with another component.
    pub async fn initialize_api_session(
        &self,
        params: InitializeApiSessionParams,
    ) -> Result<InitializeApiSessionResult> {
        self.request::<InitializeApiSessionRequest>(params).await
    }

    /// Closes the LSP transport and terminates the server process.
    ///
    /// The underlying process is shut down safely through [`AsyncChildGuard`],
    /// which ensures the child is reaped even if it needs to be killed.
    pub async fn close(&self) -> Result<()> {
        if let Some(result) = self.shutdown.begin() {
            return result;
        }
        let result = self.close_transport_and_process(Ok(())).await;
        self.shutdown.finish(&result);
        result
    }

    /// Abort an initializing transport without acquiring handshake state.
    /// Returns only after this owned child has been killed and reaped.
    pub fn abort_startup(&self) -> Result<Option<corsa_core::ProcessCloseReceipt>> {
        let receipt = self.process.abort_receipt()?;
        let transport = self.rpc.begin_abort(self.shutdown_timeout);
        let reader = self.rpc.join_reader_for_abort(self.shutdown_timeout);
        let errors: SmallVec<[CompactString; 2]> = [transport, reader]
            .into_iter()
            .filter_map(Result::err)
            .map(|error| compact_format(format_args!("{error}")))
            .collect();
        if errors.is_empty() {
            Ok(receipt)
        } else {
            let mut joined_errors = CompactString::default();
            for (index, error) in errors.iter().enumerate() {
                if index != 0 {
                    joined_errors.push_str("; ");
                }
                joined_errors.push_str(error);
            }
            Err(CorsaError::Protocol(compact_format(format_args!(
                "startup abort terminal {receipt:?}; transport close: {joined_errors}"
            ))))
        }
    }

    /// Gracefully closes the LSP session with the protocol `shutdown` / `exit` sequence.
    ///
    /// The params-less `shutdown` request is answered before the params-less
    /// `exit` notification is sent. The transport is then drained and closed,
    /// and the child is reaped or force-killed within `shutdown_timeout`.
    /// Concurrent or repeated calls across cloned clients share one shutdown
    /// attempt and return the same outcome.
    pub async fn graceful_close(&self) -> Result<()> {
        if let Some(result) = self.shutdown.begin() {
            return result;
        }

        let protocol_result = match self.request_without_params::<Shutdown>().await {
            Ok(()) => self.notify_without_params::<Exit>(),
            Err(error) => Err(error),
        };
        let result = self.close_transport_and_process(protocol_result).await;
        self.shutdown.finish(&result);
        result
    }

    async fn close_transport_and_process(&self, protocol_result: Result<()>) -> Result<()> {
        let transport_result = self.rpc.begin_close();
        let process_result = self.process.shutdown(self.shutdown_timeout).await;
        let reader_result = self.rpc.join_reader(self.shutdown_timeout);

        let errors: SmallVec<[CorsaError; 4]> = [
            protocol_result,
            transport_result,
            process_result,
            reader_result,
        ]
        .into_iter()
        .filter_map(Result::err)
        .collect();
        for error in &errors {
            self.process
                .terminal_observer()
                .record_error(compact_format(format_args!("LSP close: {error}")).as_str());
        }
        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.into_iter().next().unwrap()),
            _ => Err(CorsaError::Protocol(compact_format(format_args!(
                "LSP close errors: {errors:?}"
            )))),
        }
    }
}
