use std::{path::PathBuf, sync::Arc, time::Duration};

use super::ApiFileSystem;
use crate::process::CorsaCommand;
use corsa_core::{SharedObserver, fast::CompactString};

use super::profiling::SharedProfiler;

/// Transport mode used to talk to the Corsa API.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ApiMode {
    /// Async JSON-RPC over stdio.
    AsyncJsonRpcStdio,
    /// Sync msgpack tuples over stdio.
    SyncMsgpackStdio,
}

/// Process configuration for spawning a Corsa API worker.
///
/// A single config describes both the executable to launch and the transport
/// strategy used to communicate with it. The default is sync msgpack because
/// it minimizes framing overhead for latency-sensitive, repeated requests.
///
/// # Examples
///
/// ```
/// use corsa_client::{ApiMode, ApiSpawnConfig};
///
/// let config = ApiSpawnConfig::new("/opt/bin/corsa")
///     .with_cwd("/workspace")
///     .with_mode(ApiMode::AsyncJsonRpcStdio);
///
/// assert_eq!(config.mode, ApiMode::AsyncJsonRpcStdio);
/// assert_eq!(config.command.cwd().to_string_lossy(), "/workspace");
/// ```
#[derive(Clone)]
pub struct ApiSpawnConfig {
    /// Reusable command template used to launch the worker.
    pub command: CorsaCommand,
    /// Wire protocol used between the client and Corsa.
    pub mode: ApiMode,
    /// Optional filesystem callback implementation exposed to Corsa.
    ///
    /// This is primarily useful when the worker should consult an overlay or a
    /// virtualized filesystem instead of only reading from disk.
    pub filesystem: Option<Arc<dyn ApiFileSystem>>,
    /// Maximum time to wait for a single request before surfacing a timeout.
    pub request_timeout: Option<Duration>,
    /// Maximum time to wait for process shutdown before force-killing the worker.
    pub shutdown_timeout: Duration,
    /// Maximum number of queued outbound transport messages.
    pub outbound_capacity: usize,
    /// Maximum number of snapshot handles queued for background release.
    pub release_queue_capacity: usize,
    /// Allows calls to upstream endpoints that are known to be unstable.
    pub allow_unstable_upstream_calls: bool,
    /// Allows trusted projects to execute configured TypeScript content mapper processes.
    pub run_external_code: bool,
    /// Optional observer for structured transport events.
    pub observer: Option<SharedObserver>,
    /// Optional profiler for fine-grained request phase samples.
    pub profiler: Option<SharedProfiler>,
}

impl ApiSpawnConfig {
    /// Creates a new spawn config with the fastest stdio transport enabled by default.
    ///
    /// The sync msgpack transport avoids JSON framing and base64 binary payloads,
    /// which makes it the preferred mode for benchmark and production usage.
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            command: CorsaCommand::new(executable),
            mode: ApiMode::SyncMsgpackStdio,
            filesystem: None,
            request_timeout: Some(Duration::from_secs(30)),
            shutdown_timeout: Duration::from_secs(2),
            outbound_capacity: 256,
            release_queue_capacity: 256,
            allow_unstable_upstream_calls: false,
            run_external_code: false,
            observer: None,
            profiler: None,
        }
    }

    /// Sets the worker current directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.command = self.command.clone().with_cwd(cwd);
        self
    }

    /// Sets an environment variable on spawned workers.
    pub fn with_env(
        mut self,
        key: impl Into<CompactString>,
        value: impl Into<CompactString>,
    ) -> Self {
        self.command = self.command.clone().with_env(key, value);
        self
    }

    /// Selects the transport mode used for stdio communication.
    pub fn with_mode(mut self, mode: ApiMode) -> Self {
        self.mode = mode;
        self
    }

    /// Installs filesystem callbacks that Corsa can call back into.
    pub fn with_filesystem(mut self, filesystem: Arc<dyn ApiFileSystem>) -> Self {
        self.filesystem = Some(filesystem);
        self
    }

    /// Sets the per-request timeout applied by the client transport.
    pub fn with_request_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Sets the graceful shutdown timeout used when closing a worker.
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Sets the maximum number of queued outbound transport messages.
    pub fn with_outbound_capacity(mut self, capacity: usize) -> Self {
        self.outbound_capacity = capacity.max(1);
        self
    }

    /// Sets the maximum number of snapshot handles queued for background release.
    pub fn with_release_queue_capacity(mut self, capacity: usize) -> Self {
        self.release_queue_capacity = capacity.max(1);
        self
    }

    /// Allows calls to upstream endpoints marked unstable by this crate.
    pub fn with_allow_unstable_upstream_calls(mut self, allow: bool) -> Self {
        self.allow_unstable_upstream_calls = allow;
        self
    }

    /// Enables TypeScript's `--runExternalCode` gate for trusted content mappers.
    pub fn with_run_external_code(mut self, allow: bool) -> Self {
        self.run_external_code = allow;
        self
    }

    /// Sets the observer used for structured transport events.
    pub fn with_observer(mut self, observer: SharedObserver) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Sets the profiler used for fine-grained request phase samples.
    pub fn with_profiler(mut self, profiler: SharedProfiler) -> Self {
        self.profiler = Some(profiler);
        self
    }
}

/// Named API profile reused by orchestrators and caches.
///
/// Profiles give a stable identifier to a spawn configuration so higher-level
/// layers can pool, cache, or replicate work by profile name rather than by
/// comparing full command structures.
///
/// # Examples
///
/// ```
/// use corsa_client::{ApiProfile, ApiSpawnConfig};
///
/// let profile = ApiProfile::new("primary", ApiSpawnConfig::new("/opt/bin/corsa"));
/// assert_eq!(profile.id.as_str(), "primary");
/// ```
#[derive(Clone)]
pub struct ApiProfile {
    /// Stable profile identifier used as the cache/fleet key.
    pub id: CompactString,
    /// Spawn configuration for workers in this profile.
    pub spawn: ApiSpawnConfig,
}

impl ApiProfile {
    /// Creates a new named profile.
    pub fn new(id: impl Into<CompactString>, spawn: ApiSpawnConfig) -> Self {
        Self {
            id: id.into(),
            spawn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiMode, ApiSpawnConfig};

    #[test]
    fn new_prefers_msgpack_fast_path() {
        let config = ApiSpawnConfig::new("/opt/bin/corsa");
        assert_eq!(config.mode, ApiMode::SyncMsgpackStdio);
        assert!(!config.run_external_code);
    }
}
