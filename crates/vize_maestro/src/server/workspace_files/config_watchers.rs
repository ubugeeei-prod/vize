//! Root-owned, coalesced configuration watches; no server state enters the worker.

use std::future::Future;
use std::path::{Path, PathBuf};

use futures::FutureExt;
use futures::channel::oneshot;
use futures::future::{BoxFuture, Either, Shared as SharedFuture, select};
use parking_lot::{Condvar, Mutex};
use tower_lsp::Client;
use tower_lsp::lsp_types::{
    ClientCapabilities, DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern,
    OneOf, Registration, RelativePattern, Unregistration, Url, WatchKind,
};
use vize_l0::{String, cstr};

const METHOD: &str = "workspace/didChangeWatchedFiles";

#[derive(Default)]
struct Queue {
    supported: bool,
    started: bool,
    closed: bool,
    latest: Option<Vec<PathBuf>>,
}

#[derive(Default)]
struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
}

impl Shared {
    fn shutdown(&self) {
        let discarded = {
            let mut queue = self.queue.lock();
            queue.closed = true;
            queue.latest.take()
        };
        drop(discarded);
        self.ready.notify_one();
    }
}

#[expect(
    clippy::disallowed_types,
    reason = "the root and its OS worker require a shared queue; Carton exports no Arc"
)]
type SharedQueue = std::sync::Arc<Shared>;
type WeakQueue = std::sync::Weak<Shared>;

type TransportEnded = SharedFuture<BoxFuture<'static, ()>>;

/// Dropped only after the transport can no longer dispatch client responses.
/// Shutdown and root-owner release are deliberately not transport termination.
#[must_use = "keep the lease alive until the complete response transport has stopped"]
pub struct TransportLease {
    ended: Option<oneshot::Sender<()>>,
    shared: WeakQueue,
}

impl Drop for TransportLease {
    fn drop(&mut self) {
        if let Some(shared) = self.shared.upgrade() {
            shared.shutdown();
        }
        if let Some(ended) = self.ended.take() {
            let _ = ended.send(());
        }
    }
}

/// Foreground operations only replace desired paths or signal the single worker.
pub(in crate::server) struct Control {
    shared: SharedQueue,
    transport_end: Mutex<Option<oneshot::Sender<()>>>,
    transport_ended: TransportEnded,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Default for Control {
    fn default() -> Self {
        let (ended, receiver) = oneshot::channel();
        Self {
            shared: SharedQueue::default(),
            transport_end: Mutex::new(Some(ended)),
            transport_ended: async move {
                if receiver.await.is_err() {
                    // No transport lease was acquired. Root drop does not prove
                    // that a direct embedder's ClientSocket stopped accepting ACKs.
                    futures::future::pending::<()>().await;
                }
            }
            .boxed()
            .shared(),
            worker: Mutex::new(None),
        }
    }
}

impl Control {
    pub(in crate::server) fn transport_lease(&self) -> Option<TransportLease> {
        self.transport_end
            .lock()
            .take()
            .map(|sender| TransportLease {
                ended: Some(sender),
                shared: SharedQueue::downgrade(&self.shared),
            })
    }

    pub(in crate::server) fn record_capabilities(&self, capabilities: &ClientCapabilities) {
        let watched = capabilities
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.did_change_watched_files.as_ref());
        self.shared.queue.lock().supported = watched.is_some_and(|watch| {
            watch.dynamic_registration == Some(true) && watch.relative_pattern_support == Some(true)
        });
    }

    pub(in crate::server) fn start(&self, client: Client) {
        {
            let mut queue = self.shared.queue.lock();
            if queue.closed || queue.started || !queue.supported {
                return;
            }
            queue.started = true;
        }
        let shared = SharedQueue::downgrade(&self.shared);
        let ended = self.transport_ended.clone();
        match std::thread::Builder::new()
            .name("vize-config-watch".into())
            .spawn(move || Worker::new(client, ended, shared).run())
        {
            Ok(worker) => *self.worker.lock() = Some(worker),
            Err(error) => {
                self.shared.queue.lock().started = false;
                tracing::warn!("failed to spawn configuration watch worker: {error}");
            }
        }
    }

    pub(in crate::server) fn update(&self, mut paths: Vec<PathBuf>) {
        paths.sort();
        paths.dedup();
        let previous = {
            let mut queue = self.shared.queue.lock();
            if queue.closed || !queue.supported {
                return;
            }
            queue.latest.replace(paths)
        };
        drop(previous);
        self.shared.ready.notify_one();
    }

    pub(in crate::server) fn shutdown(&self) {
        self.shared.shutdown();
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct Generation {
    id: String,
    paths: Vec<PathBuf>,
}

struct Worker {
    client: Client,
    transport_ended: TransportEnded,
    shared: WeakQueue,
    sequence: u64,
    active: Option<Generation>,
    obsolete: Option<String>,
}

impl Worker {
    fn new(client: Client, transport_ended: TransportEnded, shared: WeakQueue) -> Self {
        Self {
            client,
            transport_ended,
            shared,
            sequence: 0,
            active: None,
            obsolete: None,
        }
    }

    fn run(mut self) {
        loop {
            let paths = {
                let Some(shared) = self.shared.upgrade() else {
                    break;
                };
                let mut queue = shared.queue.lock();
                while !queue.closed && queue.latest.is_none() {
                    shared.ready.wait(&mut queue);
                }
                if queue.closed {
                    break;
                }
                queue.latest.take()
            };
            if let Some(paths) = paths
                && !self.replace(paths)
            {
                break;
            }
        }
    }

    /// Failed cleanup waits for another ordinary update, never a retry loop.
    fn cleanup(&mut self) -> Option<bool> {
        let Some(id) = self.obsolete.as_ref() else {
            return Some(true);
        };
        match wait_rpc(
            &self.shared,
            self.transport_ended.clone(),
            self.client.unregister_capability(vec![unregistration(id)]),
        )? {
            Ok(()) => {
                self.obsolete = None;
                Some(true)
            }
            Err(error) => {
                tracing::warn!("failed to retire configuration watch {id}: {error}");
                Some(false)
            }
        }
    }

    /// At most one active ACKed generation and one obsolete ACKed ID are retained.
    fn replace(&mut self, paths: Vec<PathBuf>) -> bool {
        match self.cleanup() {
            None => return false,
            Some(false) => return true,
            Some(true) => {}
        }
        let (paths, watchers) = watchers(&paths);
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.paths == paths)
        {
            return true;
        }
        if paths.is_empty() {
            self.obsolete = self.active.take().map(|active| active.id);
            return self.cleanup().is_some();
        }
        self.sequence += 1;
        let id = cstr!("vize-config-watch-{}", self.sequence);
        let options = DidChangeWatchedFilesRegistrationOptions { watchers };
        let registration = Registration {
            id: id.clone().into(),
            method: METHOD.into(),
            register_options: match serde_json::to_value(options) {
                Ok(options) => Some(options),
                Err(error) => {
                    tracing::warn!("cannot encode configuration watches: {error}");
                    return true;
                }
            },
        };
        match wait_rpc(
            &self.shared,
            self.transport_ended.clone(),
            self.client.register_capability(vec![registration]),
        ) {
            None => return false,
            Some(Err(error)) => {
                tracing::warn!("failed to register configuration watch {id}: {error}");
                return true;
            }
            Some(Ok(())) => {}
        }
        self.obsolete = self
            .active
            .replace(Generation { id, paths })
            .map(|active| active.id);
        self.cleanup().is_some()
    }
}

fn wait_rpc<F>(
    shared: &WeakQueue,
    ended: TransportEnded,
    request: F,
) -> Option<tower_lsp::jsonrpc::Result<()>>
where
    F: Future<Output = tower_lsp::jsonrpc::Result<()>>,
{
    let mut request = Box::pin(request);
    crate::runtime::block_on(async {
        // Linearize the first poll against shutdown, then release the queue.
        // Once initiated, the request receiver must survive every shutdown ACK.
        let mut initiated = false;
        let guarded = futures::future::poll_fn(|cx| {
            if initiated {
                return request.as_mut().poll(cx).map(Some);
            }
            let Some(shared) = shared.upgrade() else {
                return std::task::Poll::Ready(None);
            };
            let queue = shared.queue.lock();
            if queue.closed {
                return std::task::Poll::Ready(None);
            }
            initiated = true;
            request.as_mut().poll(cx).map(Some)
        });
        let result = match select(ended, Box::pin(guarded)).await {
            Either::Left(_) => return None,
            Either::Right((result, _)) => result,
        };
        // An ACK received after shutdown is consumed, never followed by cleanup
        // or a coalesced replacement on the now-closed root.
        let shared = shared.upgrade()?;
        if shared.queue.lock().closed {
            None
        } else {
            result
        }
    })
}

fn unregistration(id: &str) -> Unregistration {
    Unregistration {
        id: id.into(),
        method: METHOD.into(),
    }
}

/// Keep valid targets even when another observation cannot be represented.
/// The ACKed identity is this representable subset, never a claim of full coverage.
fn watchers(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<FileSystemWatcher>) {
    let mut represented = Vec::new();
    let mut watchers = Vec::new();
    for path in paths {
        if let Some(watcher) = watcher(path) {
            represented.push(path.clone());
            watchers.push(watcher);
        } else {
            tracing::warn!(
                "cannot watch configuration target {}: requires an absolute file parent and UTF-8 filename; coverage is incomplete",
                path.display()
            );
        }
    }
    (represented, watchers)
}

fn watcher(path: &Path) -> Option<FileSystemWatcher> {
    let parent = path.parent()?;
    let filename = path.file_name()?.to_str()?;
    let base_uri = Url::from_directory_path(parent).ok()?;
    let mut pattern = String::new("");
    for character in filename.chars() {
        match character {
            '[' => pattern.push_str("[[]"),
            ']' => pattern.push_str("[]]"),
            '{' => pattern.push_str("[{]"),
            '}' => pattern.push_str("[}]"),
            '?' => pattern.push_str("[?]"),
            '*' => pattern.push_str("[*]"),
            // VS Code applies ECMAScript String.trim before parsing the glob.
            character if glob_trim_character(character) => {
                pattern.push('[');
                pattern.push(character);
                pattern.push(']');
            }
            _ => pattern.push(character),
        }
    }
    Some(FileSystemWatcher {
        glob_pattern: GlobPattern::Relative(RelativePattern {
            base_uri: OneOf::Right(base_uri),
            pattern: pattern.into(),
        }),
        kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
    })
}

fn glob_trim_character(character: char) -> bool {
    matches!(character, '\t'..='\r' | ' ' | '\u{00a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
        | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

#[cfg(test)]
#[path = "config_watchers_tests.rs"]
mod tests;
