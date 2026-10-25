use std::{
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use crate::{CorsaError, Result};
use corsa_core::fast::CompactString;
use log::warn;

use super::{
    ApiClient, DocumentIdentifier, ProjectResponse, SnapshotChanges, SnapshotHandle,
    changes::UpdateSnapshotResponse, driver::ClientDriver, profiling::SharedProfiler,
};

const DEFAULT_RELEASE_QUEUE_CAPACITY: usize = 256;
type WorkerResult = thread::Result<()>;

/// Bounded background release worker shared by all snapshots from one client.
pub(crate) struct SnapshotReleaseQueue {
    driver: Arc<ClientDriver>,
    profiler: Option<SharedProfiler>,
    sender: Mutex<Option<mpsc::SyncSender<SnapshotHandle>>>,
    done: Mutex<Option<mpsc::Receiver<WorkerResult>>>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl SnapshotReleaseQueue {
    pub(crate) fn spawn(
        driver: Arc<ClientDriver>,
        profiler: Option<SharedProfiler>,
        capacity: usize,
    ) -> Result<Self> {
        let (tx, rx) =
            mpsc::sync_channel::<SnapshotHandle>(capacity.clamp(1, DEFAULT_RELEASE_QUEUE_CAPACITY));
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        let worker_driver = Arc::clone(&driver);
        let worker_profiler = profiler.clone();
        let worker = thread::Builder::new()
            .name("corsa-snapshot-release".into())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    while let Ok(handle) = rx.recv() {
                        release_handle(&worker_driver, worker_profiler.as_ref(), handle);
                    }
                }));
                let _ = done_tx.send(result);
            })
            .map_err(CorsaError::Io)?;
        Ok(Self {
            driver,
            profiler,
            sender: Mutex::new(Some(tx)),
            done: Mutex::new(Some(done_rx)),
            worker: Mutex::new(Some(worker)),
        })
    }

    pub(crate) fn enqueue(&self, handle: SnapshotHandle) {
        let Some(sender) = lock_unpoisoned(&self.sender).as_ref().cloned() else {
            warn!(
                "failed to release corsa snapshot `{}`: release queue is closed",
                handle.as_str()
            );
            return;
        };
        match sender.try_send(handle) {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(handle)) => {
                warn!(
                    "snapshot release queue is full; releasing `{}` on the current thread",
                    handle.as_str()
                );
                release_handle(&self.driver, self.profiler.as_ref(), handle);
            }
            Err(mpsc::TrySendError::Disconnected(handle)) => {
                warn!(
                    "failed to release corsa snapshot `{}`: release queue is disconnected",
                    handle.as_str()
                );
            }
        }
    }

    pub(crate) fn is_closed(&self) -> bool {
        lock_unpoisoned(&self.sender).is_none()
    }

    pub(crate) fn is_drained(&self) -> bool {
        lock_unpoisoned(&self.done).is_none()
    }

    pub(crate) async fn close(&self, timeout: Duration) -> Result<()> {
        lock_unpoisoned(&self.sender).take();
        wait_for_worker(self, timeout, "snapshot release queue")
    }
}

fn release_handle(
    driver: &ClientDriver,
    profiler: Option<&SharedProfiler>,
    handle: SnapshotHandle,
) {
    if let Err(error) = corsa_runtime::block_on(driver.release_handle(&handle, profiler)) {
        warn!(
            "failed to release corsa snapshot `{}`: {error}",
            handle.as_str()
        );
    }
}

fn wait_for_worker(
    queue: &SnapshotReleaseQueue,
    timeout: Duration,
    operation: &'static str,
) -> Result<()> {
    let done = lock_unpoisoned(&queue.done);
    let Some(done_rx) = done.as_ref() else {
        return Ok(());
    };
    match done_rx.recv_timeout(timeout) {
        Ok(result) => {
            drop(done);
            lock_unpoisoned(&queue.done).take();
            if let Some(worker) = lock_unpoisoned(&queue.worker).take() {
                let _ = worker.join();
            }
            result
                .map_err(|_| CorsaError::Join(CompactString::from(format!("{operation} panicked"))))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            warn!("{operation} did not stop within {} ms", timeout.as_millis());
            Ok(())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(CorsaError::Closed(operation)),
    }
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One reference to a server-side snapshot handle.
///
/// The server keeps a snapshot alive until its handle is released. A lease
/// releases the handle when the last holder lets go, which lets a
/// [`ManagedSnapshot`] and the client's own [`SnapshotLineage`] share one
/// handle without either of them releasing it underneath the other.
pub(crate) struct SnapshotLease {
    handle: SnapshotHandle,
    release_queue: Arc<SnapshotReleaseQueue>,
    released: AtomicBool,
}

impl SnapshotLease {
    pub(crate) fn new(handle: SnapshotHandle, release_queue: Arc<SnapshotReleaseQueue>) -> Self {
        Self {
            handle,
            release_queue,
            released: AtomicBool::new(false),
        }
    }

    pub(crate) fn handle(&self) -> &SnapshotHandle {
        &self.handle
    }
}

impl Drop for SnapshotLease {
    fn drop(&mut self) {
        if self.released.swap(true, Ordering::SeqCst) {
            return;
        }
        self.release_queue.enqueue(self.handle.clone());
    }
}

/// The snapshot a connection's next update derives from.
///
/// On the [`DerivedSnapshots`](super::ApiDialect::DerivedSnapshots) dialect
/// the server no longer remembers which projects a connection opened: every
/// update names the snapshot it builds on. [`ApiClient::update_snapshot`]
/// still promises that changes accumulate, so the client keeps the head of that
/// chain here and pins it with its own [`SnapshotLease`].
#[derive(Default)]
pub(crate) struct SnapshotLineage {
    gate: AsyncGate,
    head: parking_lot::Mutex<Option<LineageHead>>,
}

struct LineageHead {
    lease: Arc<SnapshotLease>,
    /// Every project in the head snapshot. Updates only report the projects
    /// they added or replaced, so the full list has to be carried forward.
    projects: Vec<ProjectResponse>,
}

impl SnapshotLineage {
    /// Waits for exclusive access to the lineage.
    ///
    /// Updates are serialized because each one derives from the result of the
    /// previous one; two concurrent updates from one base would lose a change.
    pub(crate) async fn advance(&self) -> LineageAdvance<'_> {
        let permit = self.gate.acquire().await;
        let base = self
            .head
            .lock()
            .as_ref()
            .map(|head| (Arc::clone(&head.lease), head.projects.clone()));
        LineageAdvance {
            lineage: self,
            base,
            _permit: permit,
        }
    }

    /// Reports whether `handle` is the snapshot the next update derives from.
    pub(crate) fn pins(&self, handle: &str) -> bool {
        self.head
            .lock()
            .as_ref()
            .is_some_and(|head| head.lease.handle().as_str() == handle)
    }

    /// Lets go of the head snapshot, releasing it once nothing else holds it.
    pub(crate) fn clear(&self) {
        // Taken out first so the lease is released outside the lock.
        let head = self.head.lock().take();
        drop(head);
    }
}

/// Exclusive access to a lineage while one update is in flight.
pub(crate) struct LineageAdvance<'a> {
    lineage: &'a SnapshotLineage,
    base: Option<(Arc<SnapshotLease>, Vec<ProjectResponse>)>,
    _permit: GatePermit<'a>,
}

impl LineageAdvance<'_> {
    /// Handle of the snapshot the update derives from, when there is one.
    pub(crate) fn base(&self) -> Option<&SnapshotHandle> {
        self.base.as_ref().map(|(lease, _)| lease.handle())
    }

    /// Installs the update's result as the new head.
    ///
    /// `response.projects` is replaced with the complete project list of the
    /// new snapshot before the lease and response are handed back.
    pub(crate) fn commit(
        self,
        release_queue: Arc<SnapshotReleaseQueue>,
        mut response: UpdateSnapshotResponse,
    ) -> (Arc<SnapshotLease>, UpdateSnapshotResponse) {
        if let Some((_, base_projects)) = self.base {
            response.projects = merge_projects(
                base_projects,
                std::mem::take(&mut response.projects),
                response.changes.as_ref(),
            );
        }
        let lease = Arc::new(SnapshotLease::new(response.snapshot.clone(), release_queue));
        let head = LineageHead {
            lease: Arc::clone(&lease),
            projects: response.projects.clone(),
        };
        // Swapped out first so the previous head is released outside the lock.
        let previous = self.lineage.head.lock().replace(head);
        drop(previous);
        (lease, response)
    }
}

/// Applies an update's project delta to the base snapshot's project list.
fn merge_projects(
    mut projects: Vec<ProjectResponse>,
    updated: Vec<ProjectResponse>,
    changes: Option<&SnapshotChanges>,
) -> Vec<ProjectResponse> {
    if let Some(changes) = changes {
        projects.retain(|project| !changes.removed_projects.contains(&project.id));
    }
    for project in updated {
        match projects.iter_mut().find(|known| known.id == project.id) {
            Some(known) => *known = project,
            None => projects.push(project),
        }
    }
    projects
}

/// Minimal async mutex that is safe to hold across `.await`.
///
/// The client does not assume a particular executor, so a blocking mutex held
/// across a request could deadlock two futures polled from one thread.
#[derive(Default)]
struct AsyncGate {
    state: parking_lot::Mutex<GateState>,
}

#[derive(Default)]
struct GateState {
    held: bool,
    waiters: Vec<Waker>,
}

impl AsyncGate {
    fn acquire(&self) -> GateAcquire<'_> {
        GateAcquire { gate: self }
    }
}

struct GateAcquire<'a> {
    gate: &'a AsyncGate,
}

impl<'a> Future for GateAcquire<'a> {
    type Output = GatePermit<'a>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.gate.state.lock();
        if state.held {
            state.waiters.push(cx.waker().clone());
            return Poll::Pending;
        }
        state.held = true;
        Poll::Ready(GatePermit { gate: self.gate })
    }
}

struct GatePermit<'a> {
    gate: &'a AsyncGate,
}

impl Drop for GatePermit<'_> {
    fn drop(&mut self) {
        let waiters = {
            let mut state = self.gate.state.lock();
            state.held = false;
            std::mem::take(&mut state.waiters)
        };
        for waiter in waiters {
            waiter.wake();
        }
    }
}

/// Live snapshot handle with automatic release-on-drop semantics.
///
/// A managed snapshot bundles the opaque remote handle together with the
/// project list and optional change summary returned by `updateSnapshot`. When
/// the wrapper is dropped, it schedules a best-effort handle release so callers
/// do not leak server-side snapshot state accidentally.
pub struct ManagedSnapshot {
    client: ApiClient,
    lease: parking_lot::Mutex<Option<Arc<SnapshotLease>>>,
    /// Opaque snapshot handle used by follow-up API requests.
    pub handle: SnapshotHandle,
    /// Projects visible inside the snapshot at creation time.
    pub projects: Vec<ProjectResponse>,
    /// Optional project-level delta information returned by Corsa.
    pub changes: Option<SnapshotChanges>,
}

impl ManagedSnapshot {
    pub(crate) fn new(
        client: ApiClient,
        release_queue: Arc<SnapshotReleaseQueue>,
        response: UpdateSnapshotResponse,
    ) -> Self {
        let lease = Arc::new(SnapshotLease::new(response.snapshot.clone(), release_queue));
        Self::from_lease(client, lease, response)
    }

    pub(crate) fn from_lease(
        client: ApiClient,
        lease: Arc<SnapshotLease>,
        response: UpdateSnapshotResponse,
    ) -> Self {
        Self {
            client,
            lease: parking_lot::Mutex::new(Some(lease)),
            handle: response.snapshot,
            projects: response.projects,
            changes: response.changes,
        }
    }

    /// Looks up a project by its `tsconfig` path.
    ///
    /// This is a convenience helper for the common "find the project that owns
    /// this config file" flow after snapshot creation.
    pub fn project(&self, config_file_name: &str) -> Option<&ProjectResponse> {
        self.projects
            .iter()
            .find(|project| project.config_file_name == config_file_name)
    }

    /// Delegates to [`ApiClient::get_default_project_for_file`] using this snapshot.
    pub async fn get_default_project_for_file(
        &self,
        file: impl Into<DocumentIdentifier>,
    ) -> Result<Option<ProjectResponse>> {
        self.client
            .get_default_project_for_file(self.handle.clone(), file)
            .await
    }

    /// Releases the snapshot handle if it has not already been released.
    ///
    /// Calling this eagerly can reduce remote memory usage in long-lived
    /// processes when the snapshot is known to be dead before Rust drop runs.
    ///
    /// On runtimes where every update derives from the previous snapshot, the
    /// client keeps the most recent snapshot alive as the base for the next
    /// one. Releasing that snapshot here gives up this handle's share; the
    /// server-side handle goes away once a later update replaces it or the
    /// client closes.
    pub async fn release(&self) -> Result<()> {
        let Some(lease) = self.lease.lock().take() else {
            return Ok(());
        };
        let lease = match Arc::try_unwrap(lease) {
            Ok(lease) => lease,
            Err(_shared) => return Ok(()),
        };
        // Claim the release so dropping the lease does not queue a second one.
        lease.released.store(true, Ordering::SeqCst);
        match self.client.release_handle(&lease.handle).await {
            Ok(()) => Ok(()),
            Err(error) => {
                lease.released.store(false, Ordering::SeqCst);
                *self.lease.lock() = Some(Arc::new(lease));
                Err(error)
            }
        }
    }
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
