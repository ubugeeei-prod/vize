use crate::{CorsaError, Result};
use corsa_core::fast::CompactString;
use parking_lot::Mutex;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, mpsc, mpsc::Receiver},
    task::{Context, Poll, Waker},
};
use std::{path::Path, thread};

#[cfg(unix)]
use crate::jsonrpc::JsonRpcConnection;
#[cfg(unix)]
use std::{
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use super::{
    capabilities::{CapabilitiesResponse, LspCapabilities, RuntimeCapabilities},
    changes::{UpdateSnapshotParams, UpdateSnapshotResponse},
    config::{ApiMode, ApiSpawnConfig},
    document::DocumentIdentifier,
    driver::ClientDriver,
    encoded::EncodedPayload,
    profiling::SharedProfiler,
    requests_core::{
        ParseConfigFileRequest, SnapshotFileRequest, SnapshotProjectFileRequest,
        UpdateSnapshotRequest,
    },
    responses::{ConfigResponse, InitializeResponse, ProjectResponse},
    snapshot::{ManagedSnapshot, SnapshotReleaseQueue},
    spawn_stdio::{spawn_jsonrpc_stdio, spawn_msgpack_stdio},
};

/// High-level client for the Corsa stdio API.
///
/// `ApiClient` owns a single worker connection and memoizes the result of the
/// `initialize` handshake so later requests can assume the session is ready.
/// Clone values are cheap and refer to the same underlying process/transport.
///
/// # Lifecycle
///
/// 1. Create a client with [`spawn`](Self::spawn) or [`connect_pipe`](Self::connect_pipe).
/// 2. Call [`initialize`](Self::initialize) explicitly, or let endpoint helpers
///    do it lazily on first use.
/// 3. Reuse the same client for multiple snapshot and query operations.
/// 4. Call [`close`](Self::close) when the worker is no longer needed.
///
/// # Examples
///
/// ```no_run
/// use corsa_client::{ApiClient, ApiSpawnConfig};
///
/// # async fn demo() -> Result<(), corsa_client::CorsaError> {
/// let client = ApiClient::spawn(ApiSpawnConfig::new("/opt/bin/corsa")).await?;
/// let _initialize = client.initialize().await?;
/// client.close().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ApiClient {
    driver: Arc<ClientDriver>,
    initialized: Arc<SingleflightCell<InitializeResponse>>,
    capabilities: Arc<SingleflightCell<CapabilitiesResponse>>,
    release_queue: Arc<SnapshotReleaseQueue>,
    runtime_capabilities: RuntimeCapabilities,
    allow_unstable_upstream_calls: bool,
    profiler: Option<SharedProfiler>,
}

struct SingleflightCell<T> {
    state: Mutex<SingleflightState<T>>,
}

enum SingleflightState<T> {
    Empty,
    InFlight(Vec<mpsc::SyncSender<Result<Arc<T>>>>),
    Ready(Arc<T>),
}

struct SingleflightWait<T> {
    state: Arc<Mutex<SingleflightWaitState<T>>>,
    closed_name: &'static str,
}

struct SingleflightWaitState<T> {
    receiver: Option<Receiver<Result<Arc<T>>>>,
    result: Option<Result<Arc<T>>>,
    spawned: bool,
    waker: Option<Waker>,
}

impl<T> Default for SingleflightCell<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(SingleflightState::Empty),
        }
    }
}

impl<T> SingleflightCell<T>
where
    T: Send + Sync + 'static,
{
    async fn get_or_try_init<F, Fut>(&self, task: F, closed_name: &'static str) -> Result<Arc<T>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let wait = {
            let mut state = self.state.lock();
            match &mut *state {
                SingleflightState::Ready(value) => return Ok(value.clone()),
                SingleflightState::InFlight(waiters) => {
                    let (tx, rx) = mpsc::sync_channel(1);
                    waiters.push(tx);
                    Some(rx)
                }
                SingleflightState::Empty => {
                    *state = SingleflightState::InFlight(Vec::new());
                    None
                }
            }
        };
        if let Some(rx) = wait {
            return SingleflightWait::new(rx, closed_name).await;
        }

        let result = task().await.map(Arc::new);
        let waiters = {
            let mut state = self.state.lock();
            match std::mem::replace(&mut *state, SingleflightState::Empty) {
                SingleflightState::InFlight(waiters) => {
                    if let Ok(value) = &result {
                        *state = SingleflightState::Ready(value.clone());
                    }
                    waiters
                }
                SingleflightState::Ready(value) => {
                    *state = SingleflightState::Ready(value);
                    Vec::new()
                }
                SingleflightState::Empty => Vec::new(),
            }
        };
        for waiter in waiters {
            let _ = waiter.send(clone_shared_result(&result));
        }
        result
    }
}

impl<T> SingleflightWait<T> {
    fn new(receiver: Receiver<Result<Arc<T>>>, closed_name: &'static str) -> Self {
        Self {
            state: Arc::new(Mutex::new(SingleflightWaitState {
                receiver: Some(receiver),
                result: None,
                spawned: false,
                waker: None,
            })),
            closed_name,
        }
    }
}

impl<T> Future for SingleflightWait<T>
where
    T: Send + Sync + 'static,
{
    type Output = Result<Arc<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock();
        if let Some(result) = state.result.take() {
            return Poll::Ready(result);
        }
        state.waker = Some(cx.waker().clone());
        if !state.spawned {
            state.spawned = true;
            let Some(receiver) = state.receiver.take() else {
                return Poll::Ready(Err(CorsaError::Closed(self.closed_name)));
            };
            let shared = Arc::clone(&self.state);
            let closed_name = self.closed_name;
            if let Err(error) = thread::Builder::new()
                .name("corsa-singleflight-wait".into())
                .spawn(move || {
                    let result = receiver
                        .recv()
                        .map_err(|_| CorsaError::Closed(closed_name))
                        .and_then(|result| result);
                    let waker = {
                        let mut state = shared.lock();
                        state.result = Some(result);
                        state.waker.take()
                    };
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                })
            {
                return Poll::Ready(Err(CorsaError::Io(error)));
            }
        }
        Poll::Pending
    }
}

fn clone_shared_result<T>(result: &Result<Arc<T>>) -> Result<Arc<T>> {
    match result {
        Ok(value) => Ok(value.clone()),
        Err(error) => Err(error.clone_for_pending()),
    }
}

impl ApiClient {
    /// Spawns a new Corsa API worker using the supplied configuration.
    ///
    /// The underlying transport depends on [`ApiSpawnConfig::mode`]. For
    /// production and benchmark workflows, sync msgpack is typically the
    /// preferred choice because it reduces per-request overhead.
    pub async fn spawn(config: ApiSpawnConfig) -> Result<Self> {
        let driver = match config.mode {
            ApiMode::AsyncJsonRpcStdio => {
                let driver = spawn_jsonrpc_stdio(
                    &config.command,
                    config.run_external_code,
                    config.filesystem.clone(),
                    config.request_timeout,
                    config.shutdown_timeout,
                    config.outbound_capacity,
                    config.observer.clone(),
                )
                .await?;
                Arc::new(driver)
            }
            ApiMode::SyncMsgpackStdio => {
                let driver = spawn_msgpack_stdio(
                    &config.command,
                    config.run_external_code,
                    config.filesystem.clone(),
                    config.request_timeout,
                    config.outbound_capacity,
                    config.observer.clone(),
                )?;
                Arc::new(driver)
            }
        };
        let release_queue = Arc::new(SnapshotReleaseQueue::spawn(
            driver.clone(),
            config.profiler.clone(),
            config.release_queue_capacity,
        )?);
        Ok(Self {
            driver,
            initialized: Arc::new(SingleflightCell::default()),
            capabilities: Arc::new(SingleflightCell::default()),
            release_queue,
            runtime_capabilities: RuntimeCapabilities::from_spawn_config(&config),
            allow_unstable_upstream_calls: config.allow_unstable_upstream_calls,
            profiler: config.profiler.clone(),
        })
    }

    #[cfg(unix)]
    /// Connects to an already-running JSON-RPC socket.
    ///
    /// This is useful when another process owns the server lifecycle and this
    /// client should only attach to the transport.
    pub async fn connect_pipe(path: impl Into<PathBuf>) -> Result<Self> {
        connect_pipe_socket(path.into()).await
    }

    /// Initializes the worker and returns the cached `initialize` response.
    ///
    /// Repeated calls are cheap: only the first call performs network I/O.
    pub async fn initialize(&self) -> Result<Arc<InitializeResponse>> {
        self.initialized
            .get_or_try_init(
                || async {
                    self.driver
                        .request_typed("initialize", &Value::Null, self.profiler.as_ref())
                        .await
                },
                "api initialize",
            )
            .await
    }

    /// Returns the advertised runtime capabilities for this client.
    ///
    /// When the remote runtime does not implement `describeCapabilities`, this
    /// falls back to local spawn metadata and marks all proposed endpoints as
    /// unsupported.
    pub async fn describe_capabilities(&self) -> Result<Arc<CapabilitiesResponse>> {
        self.capabilities
            .get_or_try_init(
                || async {
                    let capabilities = match self
                        .raw_json_request("describeCapabilities", Value::Null)
                        .await
                    {
                        Ok(value) => {
                            let mut parsed: CapabilitiesResponse = serde_json::from_value(value)?;
                            parsed.runtime = parsed
                                .runtime
                                .merge_with_local(self.runtime_capabilities.clone());
                            parsed.runtime.capability_endpoint = true;
                            parsed.lsp = parsed
                                .lsp
                                .merge_with_local(LspCapabilities::from_runtime(&parsed.runtime));
                            parsed
                        }
                        Err(CorsaError::Rpc(error))
                            if error.code == -32601
                                || is_unknown_api_method_message(&error.message) =>
                        {
                            CapabilitiesResponse::fallback(self.runtime_capabilities.clone())
                        }
                        Err(CorsaError::Protocol(message))
                            if is_unknown_api_method_message(&message) =>
                        {
                            CapabilitiesResponse::fallback(self.runtime_capabilities.clone())
                        }
                        Err(error) => return Err(error),
                    };
                    Ok(capabilities)
                },
                "api describeCapabilities",
            )
            .await
    }

    /// Parses a `tsconfig` file through Corsa.
    ///
    /// The returned [`ConfigResponse`] contains the normalized compiler options
    /// and the file set that Corsa resolved for that config file.
    pub async fn parse_config_file(
        &self,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<ConfigResponse> {
        self.initialize().await?;
        let request = ParseConfigFileRequest { file: file.into() };
        self.request_after_initialize("parseConfigFile", &request)
            .await
    }

    /// Applies file changes and returns a managed snapshot handle.
    ///
    /// Snapshots are the unit of reuse for project graphs inside Corsa. The
    /// returned [`ManagedSnapshot`] automatically releases its remote handle
    /// when dropped, but can also be released eagerly via
    /// [`ManagedSnapshot::release`](crate::ManagedSnapshot::release).
    pub async fn update_snapshot(&self, params: UpdateSnapshotParams) -> Result<ManagedSnapshot> {
        if params.overlay_changes.is_some() {
            self.require_overlay_update_capability().await?;
        }
        self.initialize().await?;
        let open_projects = params.open_project.into_iter().collect();
        let request = UpdateSnapshotRequest {
            open_projects,
            file_changes: params.file_changes,
            overlay_changes: params.overlay_changes,
        };
        let response: UpdateSnapshotResponse = self
            .request_after_initialize("updateSnapshot", &request)
            .await?;
        Ok(super::snapshot::ManagedSnapshot::new(
            self.clone(),
            self.release_queue.clone(),
            response,
        ))
    }

    /// Resolves the default project for a file inside a snapshot.
    ///
    /// Returns `Ok(None)` when the file does not belong to any known project in
    /// the snapshot.
    pub async fn get_default_project_for_file(
        &self,
        snapshot: super::SnapshotHandle,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<Option<ProjectResponse>> {
        self.initialize().await?;
        let request = SnapshotFileRequest {
            snapshot,
            file: file.into(),
        };
        self.request_optional_after_initialize("getDefaultProjectForFile", &request)
            .await
    }

    /// Fetches a source file via a binary endpoint.
    ///
    /// Binary endpoints avoid JSON/base64 expansion and are a good fit for
    /// large payloads such as serialized source files.
    pub async fn get_source_file(
        &self,
        snapshot: super::SnapshotHandle,
        project: super::ProjectHandle,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<Option<EncodedPayload>> {
        self.initialize().await?;
        let request = SnapshotProjectFileRequest {
            snapshot,
            project,
            file: file.into(),
        };
        self.request_binary_after_initialize("getSourceFile", &request)
            .await
    }

    /// Closes the client and shuts down the underlying worker process.
    ///
    /// This is idempotent. After closing, further requests return
    /// [`CorsaError::Closed`].
    pub async fn close(&self) -> Result<()> {
        self.release_queue
            .close(self.driver.shutdown_timeout())
            .await?;
        self.driver.close().await
    }

    /// Returns whether unstable upstream endpoints are allowed for this client.
    pub fn allows_unstable_upstream_calls(&self) -> bool {
        self.allow_unstable_upstream_calls
    }

    /// Sends a raw JSON endpoint request after initialization.
    ///
    /// Prefer the typed helpers where available, and use this escape hatch when
    /// experimenting with new upstream endpoints.
    pub async fn raw_json_request(&self, method: &str, params: Value) -> Result<Value> {
        self.initialize().await?;
        if self.profiler.is_some() {
            self.driver
                .request_typed(method, &params, self.profiler.as_ref())
                .await
        } else {
            self.driver.request_json(method, params).await
        }
    }

    /// Sends a raw binary endpoint request after initialization.
    ///
    /// The returned payload is wrapped in [`EncodedPayload`] for zero-surprise
    /// ownership semantics.
    pub async fn raw_binary_request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Option<EncodedPayload>> {
        self.initialize().await?;
        if self.profiler.is_some() {
            Ok(self
                .driver
                .request_binary_typed(method, &params, self.profiler.as_ref())
                .await?
                .map(EncodedPayload::new))
        } else {
            Ok(self
                .driver
                .request_binary(method, params)
                .await?
                .map(EncodedPayload::new))
        }
    }

    pub(crate) async fn release_handle(&self, handle: &super::SnapshotHandle) -> Result<()> {
        self.driver
            .release_handle(handle, self.profiler.as_ref())
            .await?;
        Ok(())
    }

    pub(crate) async fn call<T, P>(&self, method: &str, params: P) -> Result<T>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        self.initialize().await?;
        self.request_after_initialize(method, &params).await
    }

    pub(crate) async fn call_optional<T, P>(&self, method: &str, params: P) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        self.initialize().await?;
        self.request_optional_after_initialize(method, &params)
            .await
    }

    pub(crate) async fn call_optional_binary<P>(
        &self,
        method: &str,
        params: P,
    ) -> Result<Option<EncodedPayload>>
    where
        P: Serialize,
    {
        self.initialize().await?;
        self.request_binary_after_initialize(method, &params).await
    }

    pub(crate) async fn require_overlay_update_capability(&self) -> Result<()> {
        let capabilities = self.describe_capabilities().await?;
        if capabilities.overlay.update_snapshot_overlay_changes {
            return Ok(());
        }
        Err(CorsaError::Unsupported(
            "updateSnapshot.overlayChanges is not supported by this runtime; check describeCapabilities before sending in-memory overlays",
        ))
    }

    pub(crate) fn map_missing_method(
        error: CorsaError,
        unsupported_message: &'static str,
    ) -> CorsaError {
        match error {
            CorsaError::Rpc(rpc)
                if rpc.code == -32601 || is_unknown_api_method_message(&rpc.message) =>
            {
                CorsaError::Unsupported(unsupported_message)
            }
            CorsaError::Protocol(message) if is_unknown_api_method_message(&message) => {
                CorsaError::Unsupported(unsupported_message)
            }
            other => other,
        }
    }

    /// Reports whether an error means the server could not resolve a type handle.
    ///
    /// Upstream Corsa occasionally evicts a live handle mid-session (for
    /// example after a base-types query on a class with no explicit `extends`
    /// clause). A follow-up relation query on that handle then fails with
    /// "type handle ... not found in snapshot registry". Some upstream paths
    /// instead report "empty type handle" for the same missing-data condition.
    /// Relation endpoints treat both as missing data so analysis can continue.
    /// The message arrives as [`CorsaError::Protocol`] over msgpack and as
    /// [`CorsaError::Rpc`] over JSON-RPC, so both variants are inspected.
    ///
    /// This classification is public so binding layers can share one
    /// definition instead of re-matching upstream message fragments.
    pub fn is_stale_handle_error(error: &CorsaError) -> bool {
        match error {
            CorsaError::Rpc(rpc) => is_stale_handle_message(&rpc.message),
            CorsaError::Protocol(message) => is_stale_handle_message(message),
            _ => false,
        }
    }

    /// Reports whether an error came from an upstream panic.
    ///
    /// A few experimental type-relation endpoints can panic for type shapes
    /// they do not currently support. Higher-level typed helpers use this to
    /// fall back to adjacent upstream endpoints without exposing the panic to
    /// consumers.
    pub(crate) fn is_protocol_panic_error(error: &CorsaError) -> bool {
        match error {
            CorsaError::Rpc(rpc) => is_protocol_panic_message(&rpc.message),
            CorsaError::Protocol(message) => is_protocol_panic_message(message),
            _ => false,
        }
    }

    async fn request_after_initialize<T, P>(&self, method: &str, params: &P) -> Result<T>
    where
        T: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        self.driver
            .request_typed(method, params, self.profiler.as_ref())
            .await
    }

    async fn request_optional_after_initialize<T, P>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        // `Option<T>` deserializes `null` (and the msgpack lane's empty
        // response) to `None` directly, so the response is decoded in one
        // pass instead of materializing an intermediate `Value` first.
        self.request_after_initialize(method, params).await
    }

    async fn request_binary_after_initialize<P>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<Option<EncodedPayload>>
    where
        P: Serialize + ?Sized,
    {
        Ok(self
            .driver
            .request_binary_typed(method, params, self.profiler.as_ref())
            .await?
            .map(EncodedPayload::new))
    }
}

#[cfg(unix)]
async fn connect_pipe_socket(path: PathBuf) -> Result<ApiClient> {
    let stream = std::os::unix::net::UnixStream::connect(path)?;
    let reader = BufReader::new(stream.try_clone()?);
    let writer = BufWriter::new(stream);
    let rpc = JsonRpcConnection::try_spawn(reader, writer, Default::default())?;
    let driver = Arc::new(ClientDriver::JsonRpc {
        rpc,
        process: None,
        shutdown_timeout: std::time::Duration::from_secs(2),
    });
    let release_queue = Arc::new(SnapshotReleaseQueue::spawn(driver.clone(), None, 256)?);
    Ok(ApiClient {
        driver,
        initialized: Arc::new(SingleflightCell::default()),
        capabilities: Arc::new(SingleflightCell::default()),
        release_queue,
        runtime_capabilities: RuntimeCapabilities {
            kind: Some(CompactString::from("pipe")),
            executable: None,
            transport: Some(CompactString::from("jsonrpc")),
            capability_endpoint: false,
        },
        allow_unstable_upstream_calls: false,
        profiler: None,
    })
}

impl RuntimeCapabilities {
    fn from_spawn_config(config: &ApiSpawnConfig) -> Self {
        let executable = config.command.executable().to_string_lossy().to_string();
        Self {
            kind: infer_runtime_kind(config.command.executable()),
            executable: Some(CompactString::from(executable)),
            transport: Some(match config.mode {
                ApiMode::AsyncJsonRpcStdio => CompactString::from("jsonrpc"),
                ApiMode::SyncMsgpackStdio => CompactString::from("msgpack"),
            }),
            capability_endpoint: false,
        }
    }
}

fn infer_runtime_kind(path: &Path) -> Option<CompactString> {
    let normalized = path.to_string_lossy().to_ascii_lowercase();
    let file_name = normalized
        .rsplit(|character| ['/', '\\'].contains(&character))
        .next();
    let kind = if normalized.contains("mock_corsa") {
        "mock-corsa"
    } else if normalized.contains("native-preview") {
        "native-preview"
    } else if matches!(file_name, Some("tsc" | "tsc.exe" | "tsgo" | "tsgo.exe")) {
        "typescript"
    } else if normalized.ends_with("/corsa")
        || normalized.ends_with("\\corsa.exe")
        || normalized.ends_with("\\corsa")
        || normalized.ends_with("/corsa.exe")
    {
        "corsa"
    } else {
        "custom"
    };
    Some(CompactString::from(kind))
}

fn is_unknown_api_method_message(message: &str) -> bool {
    message.contains("unknown API method")
}

fn is_stale_handle_message(message: &str) -> bool {
    message.contains("not found in snapshot registry") || message.contains("empty type handle")
}

fn is_protocol_panic_message(message: &str) -> bool {
    message.contains("panic:")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{ApiClient, infer_runtime_kind, is_unknown_api_method_message};
    use crate::CorsaError;
    use corsa_core::{RpcResponseError, fast::CompactString};

    #[test]
    fn recognizes_msgpack_unknown_method_protocol_error() {
        assert!(is_unknown_api_method_message(
            "api: invalid request: unknown API method \"describeCapabilities\""
        ));
    }

    #[test]
    fn infers_typescript_runtime_from_standard_executable_names() {
        for path in [
            "/tmp/typescript/lib/tsc",
            "/tmp/typescript/lib/tsgo",
            r"C:\typescript\lib\tsc.exe",
            r"C:\typescript\lib\tsgo.exe",
        ] {
            assert_eq!(
                infer_runtime_kind(Path::new(path)).as_deref(),
                Some("typescript"),
                "{path}"
            );
        }
    }

    #[test]
    fn does_not_infer_typescript_runtime_from_wrapper_names() {
        assert_eq!(
            infer_runtime_kind(Path::new("/tmp/my-tsc-wrapper")).as_deref(),
            Some("custom")
        );
    }

    #[test]
    fn missing_msgpack_api_method_maps_to_unsupported() {
        let error = ApiClient::map_missing_method(
            CorsaError::Protocol(CompactString::from(
                "api: invalid request: unknown API method \"getDiagnosticsForFile\"",
            )),
            "file diagnostics are not supported",
        );

        assert!(matches!(
            error,
            CorsaError::Unsupported("file diagnostics are not supported")
        ));
    }

    #[test]
    fn missing_jsonrpc_api_method_maps_to_unsupported() {
        let error = ApiClient::map_missing_method(
            CorsaError::Rpc(RpcResponseError {
                code: -32603,
                message: CompactString::from(
                    "api: invalid request: unknown API method \"getDiagnosticsForFile\"",
                ),
                data: None,
            }),
            "file diagnostics are not supported",
        );

        assert!(matches!(
            error,
            CorsaError::Unsupported("file diagnostics are not supported")
        ));
    }

    #[test]
    fn recognizes_stale_handle_protocol_error() {
        assert!(ApiClient::is_stale_handle_error(&CorsaError::Protocol(
            CompactString::from(
                "api: client error: type handle \"t0000000000000057\" not found in snapshot registry",
            ),
        )));
    }

    #[test]
    fn recognizes_stale_handle_rpc_error() {
        assert!(ApiClient::is_stale_handle_error(&CorsaError::Rpc(
            RpcResponseError {
                code: -32603,
                message: CompactString::from(
                    "type handle \"t00000000000000c4\" not found in snapshot registry",
                ),
                data: None,
            },
        )));
    }

    #[test]
    fn recognizes_empty_type_handle_protocol_error() {
        assert!(ApiClient::is_stale_handle_error(&CorsaError::Protocol(
            CompactString::from("api: client error: empty type handle"),
        )));
    }

    #[test]
    fn unrelated_errors_are_not_stale_handles() {
        assert!(!ApiClient::is_stale_handle_error(&CorsaError::Protocol(
            CompactString::from("api: client error: unexpected failure"),
        )));
        assert!(!ApiClient::is_stale_handle_error(&CorsaError::Rpc(
            RpcResponseError {
                code: -32601,
                message: CompactString::from(
                    "api: invalid request: unknown API method \"getBaseTypes\"",
                ),
                data: None,
            },
        )));
        assert!(!ApiClient::is_stale_handle_error(&CorsaError::Closed(
            "api"
        )));
    }
}
