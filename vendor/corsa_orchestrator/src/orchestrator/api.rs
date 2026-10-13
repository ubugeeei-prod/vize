use super::panic_payload::panic_payload_message;
use crate::Result;
use crate::api::{ApiClient, ApiProfile, ManagedSnapshot, UpdateSnapshotParams};
use corsa_core::{
    CorsaEvent, SharedObserver,
    fast::{CompactString, FastMap, SmallVec, compact_format},
    observe,
};
use corsa_runtime::block_on;
use log::warn;
use parking_lot::{Mutex, RwLock};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    any::Any,
    collections::VecDeque,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// Local pool/cache orchestrator for multiple Corsa API workers.
///
/// This type is where session reuse becomes a first-class concept. It can:
///
/// - prewarm multiple workers for a named [`ApiProfile`]
/// - round-robin lease clients for parallel work
/// - cache snapshots by a caller-provided key
/// - memoize arbitrary JSON-serializable results with an optional TTL
/// - fan work out across multiple workers while preserving input order
///
/// In other words, it optimizes for end-to-end workflow latency rather than
/// trying to outperform Corsa's own compiler internals directly.
#[derive(Clone)]
pub struct ApiOrchestratorConfig {
    /// Maximum number of workers allowed in a single profile fleet.
    pub max_workers_per_profile: usize,
    /// Maximum number of cached snapshots retained at once.
    pub max_cached_snapshots: usize,
    /// Maximum number of cached result entries retained at once.
    pub max_cached_results: usize,
    /// Maximum number of work items buffered for a batch execution.
    pub work_queue_capacity: usize,
    /// Optional observer for structured orchestrator events.
    pub observer: Option<SharedObserver>,
}

impl Default for ApiOrchestratorConfig {
    fn default() -> Self {
        Self {
            max_workers_per_profile: 8,
            max_cached_snapshots: 64,
            max_cached_results: 256,
            work_queue_capacity: 256,
            observer: None,
        }
    }
}

impl ApiOrchestratorConfig {
    /// Sets the observer used for structured orchestrator events.
    pub fn with_observer(mut self, observer: SharedObserver) -> Self {
        self.observer = Some(observer);
        self
    }
}

impl std::fmt::Debug for ApiOrchestratorConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApiOrchestratorConfig")
            .field("max_workers_per_profile", &self.max_workers_per_profile)
            .field("max_cached_snapshots", &self.max_cached_snapshots)
            .field("max_cached_results", &self.max_cached_results)
            .field("work_queue_capacity", &self.work_queue_capacity)
            .field("observer", &self.observer.is_some())
            .finish()
    }
}

/// Cheap operational snapshot for the local orchestrator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiOrchestratorStats {
    /// Number of named worker fleets.
    pub profile_count: usize,
    /// Total number of live workers across all fleets.
    pub worker_count: usize,
    /// Number of cached snapshots currently retained.
    pub cached_snapshot_count: usize,
    /// Number of cached result entries currently retained.
    pub cached_result_count: usize,
}

/// Local worker-pool orchestrator with snapshot and result caches.
///
/// `ApiOrchestrator` is cheap to clone and intended to be shared between
/// higher-level workflows. It lazily creates [`ApiClient`] fleets by
/// [`ApiProfile`], leases workers round-robin, and bounds its caches according
/// to [`ApiOrchestratorConfig`].
pub struct ApiOrchestrator {
    config: ApiOrchestratorConfig,
    fleets: RwLock<FastMap<CompactString, Arc<ClientFleet>>>,
    snapshots: RwLock<FastMap<CompactString, Arc<ManagedSnapshot>>>,
    snapshot_order: Mutex<VecDeque<CompactString>>,
    cached: RwLock<FastMap<CompactString, CachedValue>>,
    cached_order: Mutex<VecDeque<CompactString>>,
}

struct ClientFleet {
    next: AtomicUsize,
    clients: RwLock<SmallVec<[ApiClient; 4]>>,
}

struct CachedValue {
    expires_at: Option<Instant>,
    bytes: SmallVec<[u8; 256]>,
}

impl Default for ApiOrchestrator {
    fn default() -> Self {
        Self::new(ApiOrchestratorConfig::default())
    }
}

impl ApiOrchestrator {
    /// Creates a new local orchestrator with explicit resource limits.
    pub fn new(config: ApiOrchestratorConfig) -> Self {
        Self {
            config,
            fleets: RwLock::new(FastMap::default()),
            snapshots: RwLock::new(FastMap::default()),
            snapshot_order: Mutex::new(VecDeque::new()),
            cached: RwLock::new(FastMap::default()),
            cached_order: Mutex::new(VecDeque::new()),
        }
    }

    /// Returns the configured local safety limits.
    pub fn config(&self) -> &ApiOrchestratorConfig {
        &self.config
    }

    /// Returns a cheap operational snapshot of the orchestrator state.
    pub fn stats(&self) -> ApiOrchestratorStats {
        let fleets = self.fleets.read();
        ApiOrchestratorStats {
            profile_count: fleets.len(),
            worker_count: fleets
                .values()
                .map(|fleet| fleet.clients.read().len())
                .sum::<usize>(),
            cached_snapshot_count: self.snapshots.read().len(),
            cached_result_count: self.cached.read().len(),
        }
    }

    /// Ensures that at least `replicas` workers have been started for `profile`.
    ///
    /// This is useful for benchmark setup or for services that want to pay
    /// startup cost ahead of the first user request.
    pub async fn prewarm(&self, profile: &ApiProfile, replicas: usize) -> Result<()> {
        if replicas > self.config.max_workers_per_profile {
            return Err(crate::CorsaError::Protocol(compact_format(format_args!(
                "requested {replicas} workers for profile `{}` exceeds the configured maximum of {}",
                profile.id, self.config.max_workers_per_profile
            ))));
        }
        let fleet = self.fleet(profile).await?;
        while fleet.clients.read().len() < replicas {
            let client = ApiClient::spawn(profile.spawn.clone()).await?;
            let accepted = {
                let mut clients = fleet.clients.write();
                if clients.len() < replicas && clients.len() < self.config.max_workers_per_profile {
                    clients.push(client.clone());
                    true
                } else {
                    false
                }
            };
            if !accepted {
                // A concurrent prewarm satisfied the target while this worker
                // was spawning; shut the extra worker down instead of leaking
                // it past the requested replica count.
                let _ = client.close().await;
                break;
            }
        }
        Ok(())
    }

    /// Leases a worker using round-robin selection.
    ///
    /// The returned client is shared and cheaply clonable; leasing does not
    /// transfer ownership of the underlying process.
    pub async fn lease(&self, profile: &ApiProfile) -> Result<ApiClient> {
        self.prewarm(profile, 1).await?;
        let fleet = self.fleet(profile).await?;
        let clients = fleet.clients.read();
        let index = fleet.next.fetch_add(1, Ordering::Relaxed) % clients.len();
        Ok(clients[index].clone())
    }

    /// Returns a cached snapshot or creates one lazily.
    ///
    /// Snapshot keys are application-defined. Reusing a stable key for the same
    /// logical workspace lets callers amortize project graph construction.
    pub async fn cached_snapshot(
        &self,
        profile: &ApiProfile,
        key: impl Into<CompactString>,
        params: UpdateSnapshotParams,
    ) -> Result<Arc<ManagedSnapshot>> {
        let key = key.into();
        if let Some(snapshot) = self.snapshots.read().get(key.as_str()) {
            return Ok(snapshot.clone());
        }
        let client = self.lease(profile).await?;
        let snapshot: Arc<ManagedSnapshot> = Arc::new(client.update_snapshot(params).await?);
        self.snapshots.write().insert(key.clone(), snapshot.clone());
        self.remember_snapshot_key(key);
        Ok(snapshot)
    }

    /// Invalidates a cached snapshot by key.
    ///
    /// This only removes the local cache entry. Existing `Arc<ManagedSnapshot>`
    /// clones continue to live until all references are dropped.
    pub fn invalidate_snapshot(&self, key: &str) {
        self.snapshots.write().remove(key);
        self.snapshot_order
            .lock()
            .retain(|entry| entry.as_str() != key);
    }

    /// Memoizes the result of an async task for an optional TTL.
    ///
    /// Values are stored as JSON bytes so cache hits do not need the original
    /// worker process or closure again.
    pub async fn cached<T, F, Fut>(
        &self,
        profile: &ApiProfile,
        key: impl Into<CompactString>,
        ttl: Option<Duration>,
        task: F,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce(ApiClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let key = key.into();
        if let Some(value) = self.cached.read().get(key.as_str())
            && value
                .expires_at
                .map(|deadline| deadline > Instant::now())
                .unwrap_or(true)
        {
            return Ok(serde_json::from_slice(&value.bytes)?);
        }
        let value = task(self.lease(profile).await?).await?;
        self.cached.write().insert(
            key.clone(),
            CachedValue {
                expires_at: ttl.map(|ttl| Instant::now() + ttl),
                bytes: serde_json::to_vec(&value)?.into_iter().collect(),
            },
        );
        self.remember_cached_key(key);
        Ok(value)
    }

    /// Removes a cached result entry by key.
    pub fn invalidate_cached(&self, key: &str) {
        self.cached.write().remove(key);
        self.cached_order
            .lock()
            .retain(|entry| entry.as_str() != key);
    }

    /// Executes the same task across a batch using multiple workers.
    ///
    /// Results preserve the original input order even though execution may
    /// happen concurrently across different workers.
    pub async fn execute_all<T, F, Fut, I, R>(
        &self,
        profile: &ApiProfile,
        replicas: usize,
        inputs: I,
        task: F,
    ) -> Result<SmallVec<[R; 8]>>
    where
        I: IntoIterator<Item = T>,
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(ApiClient, T) -> Fut + Send + Sync + Copy + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
    {
        let inputs = inputs.into_iter().collect::<SmallVec<[T; 8]>>();
        if inputs.is_empty() {
            return Ok(SmallVec::new());
        }
        self.prewarm(profile, replicas).await?;
        let work_count = inputs.len();
        let queue_capacity = self.config.work_queue_capacity.max(1);
        let (work_tx, work_rx) = mpsc::sync_channel::<(usize, T)>(queue_capacity);
        let work_rx = Arc::new(std::sync::Mutex::new(work_rx));
        let (result_tx, result_rx) = mpsc::channel::<Result<(usize, R)>>();
        thread::scope(|scope| {
            for _ in 0..replicas.max(1) {
                let profile = profile.clone();
                let work_rx = work_rx.clone();
                let result_tx = result_tx.clone();
                scope.spawn(move || {
                    loop {
                        let job = match work_rx.lock() {
                            Ok(work_rx) => work_rx.recv(),
                            Err(_) => {
                                let _ = result_tx.send(Err(crate::CorsaError::Join(
                                    CompactString::from("orchestrator work queue lock poisoned"),
                                )));
                                break;
                            }
                        };
                        let Ok((index, input)) = job else {
                            break;
                        };
                        let output = catch_unwind(AssertUnwindSafe(|| {
                            block_on(async {
                                let client = self.lease(&profile).await?;
                                Ok::<_, crate::CorsaError>((index, task(client, input).await?))
                            })
                        }))
                        .unwrap_or_else(|panic| Err(batch_worker_panic(panic)));
                        if result_tx.send(output).is_err() {
                            break;
                        }
                    }
                });
            }
            drop(result_tx);
            for (index, input) in inputs.into_iter().enumerate() {
                work_tx
                    .send((index, input))
                    .map_err(|_| crate::CorsaError::Closed("orchestrator work"))?;
            }
            drop(work_tx);
            Ok::<_, crate::CorsaError>(())
        })?;
        let mut values = SmallVec::<[(usize, R); 8]>::new();
        for _ in 0..work_count {
            values.push(
                result_rx
                    .recv()
                    .map_err(|_| crate::CorsaError::Closed("orchestrator result"))??,
            );
        }
        values.sort_by_key(|(index, _)| *index);
        Ok(values.into_iter().map(|(_, value)| value).collect())
    }

    async fn fleet(&self, profile: &ApiProfile) -> Result<Arc<ClientFleet>> {
        if let Some(fleet) = self.fleets.read().get(profile.id.as_str()) {
            return Ok(fleet.clone());
        }
        // Re-check under the write lock so two racing callers agree on one
        // fleet instead of the loser replacing (and orphaning) the winner's.
        let mut fleets = self.fleets.write();
        let fleet = fleets
            .entry(profile.id.clone())
            .or_insert_with(|| {
                Arc::new(ClientFleet {
                    next: AtomicUsize::new(0),
                    clients: RwLock::new(SmallVec::new()),
                })
            })
            .clone();
        Ok(fleet)
    }

    fn remember_snapshot_key(&self, key: CompactString) {
        let mut order = self.snapshot_order.lock();
        order.retain(|existing| existing != key);
        order.push_back(key);
        while self.snapshots.read().len() > self.config.max_cached_snapshots.max(1) {
            let Some(evicted) = order.pop_front() else {
                break;
            };
            if self.snapshots.write().remove(evicted.as_str()).is_some() {
                observe(
                    self.config.observer.as_ref(),
                    CorsaEvent::OrchestratorSnapshotEvicted {
                        key: evicted.clone(),
                    },
                );
                warn!("evicted cached snapshot `{evicted}` to stay within configured limits");
            }
        }
    }

    fn remember_cached_key(&self, key: CompactString) {
        let mut order = self.cached_order.lock();
        order.retain(|existing| existing != key);
        order.push_back(key);
        while self.cached.read().len() > self.config.max_cached_results.max(1) {
            let Some(evicted) = order.pop_front() else {
                break;
            };
            if self.cached.write().remove(evicted.as_str()).is_some() {
                observe(
                    self.config.observer.as_ref(),
                    CorsaEvent::OrchestratorResultEvicted {
                        key: evicted.clone(),
                    },
                );
                warn!("evicted cached result `{evicted}` to stay within configured limits");
            }
        }
    }
}

fn batch_worker_panic(panic: Box<dyn Any + Send>) -> crate::CorsaError {
    let message = panic_payload_message(panic.as_ref());
    crate::CorsaError::Join(compact_format(format_args!(
        "orchestrator batch worker panicked: {message}"
    )))
}
