//! Discovery and caching of declarations that augment Vue global components.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use futures::{channel::oneshot, lock::Mutex as AsyncMutex};
use ignore::{DirEntry, WalkBuilder};
use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;

#[cfg(test)]
use parking_lot::Mutex;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;

use super::ServerState;

struct CachedPaths {
    generation: u64,
    paths: Vec<PathBuf>,
}

pub(super) struct GlobalComponentReferences {
    paths: RwLock<Option<CachedPaths>>,
    scan_lock: AsyncMutex<()>,
    generation: AtomicU64,
    watcher_supported: AtomicBool,
    #[cfg(test)]
    scan_count: AtomicUsize,
    #[cfg(test)]
    last_scan_thread: Mutex<Option<std::thread::ThreadId>>,
}

impl GlobalComponentReferences {
    pub(super) fn new() -> Self {
        Self {
            paths: RwLock::new(None),
            scan_lock: AsyncMutex::new(()),
            generation: AtomicU64::new(0),
            watcher_supported: AtomicBool::new(false),
            #[cfg(test)]
            scan_count: AtomicUsize::new(0),
            #[cfg(test)]
            last_scan_thread: Mutex::new(None),
        }
    }

    pub(super) fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        *self.paths.write() = None;
    }

    fn cached_paths(&self) -> Option<Vec<PathBuf>> {
        let generation = self.generation.load(Ordering::Acquire);
        self.paths
            .read()
            .as_ref()
            .filter(|cached| cached.generation == generation)
            .map(|cached| cached.paths.clone())
    }

    async fn paths<F>(&self, mut workspace_root: F) -> Vec<PathBuf>
    where
        F: FnMut() -> Option<PathBuf>,
    {
        loop {
            if let Some(paths) = self.cached_paths() {
                return paths;
            }

            let _scan = self.scan_lock.lock().await;
            if let Some(paths) = self.cached_paths() {
                return paths;
            }
            let generation = self.generation.load(Ordering::Acquire);
            let paths = match workspace_root() {
                Some(root) => {
                    #[cfg(test)]
                    self.scan_count.fetch_add(1, Ordering::AcqRel);
                    let discovery = discover_in_background(root).await;
                    #[cfg(test)]
                    if let Some(thread_id) = discovery.thread_id {
                        *self.last_scan_thread.lock() = Some(thread_id);
                    }
                    discovery.paths
                }
                None => Vec::new(),
            };

            if self.generation.load(Ordering::Acquire) != generation {
                continue;
            }
            *self.paths.write() = Some(CachedPaths {
                generation,
                paths: paths.clone(),
            });
            return paths;
        }
    }
}

impl ServerState {
    pub(crate) async fn global_component_reference_paths(&self) -> Vec<PathBuf> {
        self.global_component_references
            .paths(|| self.get_workspace_root())
            .await
    }

    pub(crate) fn invalidate_global_component_references<I, U>(&self, uris: I) -> bool
    where
        I: IntoIterator<Item = U>,
        U: AsRef<str>,
    {
        if !uris.into_iter().any(|uri| is_declaration_uri(uri.as_ref())) {
            return false;
        }
        self.global_component_references.invalidate();
        self.batch_cache.invalidate();
        true
    }

    pub(crate) fn set_global_component_watcher_supported(&self, supported: bool) {
        self.global_component_references
            .watcher_supported
            .store(supported, Ordering::Release);
    }

    pub(crate) fn global_component_watcher_supported(&self) -> bool {
        self.global_component_references
            .watcher_supported
            .load(Ordering::Acquire)
    }
}

struct DiscoveryResult {
    paths: Vec<PathBuf>,
    #[cfg(test)]
    thread_id: Option<std::thread::ThreadId>,
}

async fn discover_in_background(root: PathBuf) -> DiscoveryResult {
    let (sender, receiver) = oneshot::channel();
    let spawned = std::thread::Builder::new()
        .name("vize-global-components".to_string())
        .spawn(move || {
            let paths = collect_global_component_declarations(&root);
            let _ = sender.send(DiscoveryResult {
                paths,
                #[cfg(test)]
                thread_id: Some(std::thread::current().id()),
            });
        });
    if let Err(error) = spawned {
        tracing::warn!("failed to spawn global component discovery: {error}");
        return DiscoveryResult {
            paths: Vec::new(),
            #[cfg(test)]
            thread_id: None,
        };
    }
    receiver.await.unwrap_or_else(|_| DiscoveryResult {
        paths: Vec::new(),
        #[cfg(test)]
        thread_id: None,
    })
}

fn collect_global_component_declarations(root: &Path) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .ignore(false)
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .parents(false)
        .follow_links(false)
        .filter_entry(should_visit);

    let mut paths = Vec::new();
    for entry in builder.build().flatten() {
        let path = entry.path();
        if !is_declaration_file(path) {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.len() > 4 * 1024 * 1024 {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if content.contains("GlobalComponents") && content.contains("declare module") {
            paths.push(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()));
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn should_visit(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_some_and(|kind| kind.is_dir()) {
        return true;
    }
    !matches!(
        entry.file_name().to_str(),
        Some(".git" | "node_modules" | "target" | "coverage" | "dist")
    )
}

fn is_declaration_uri(uri: &str) -> bool {
    Url::parse(uri)
        .ok()
        .and_then(|url| url.to_file_path().ok())
        .is_some_and(|path| is_declaration_file(&path))
        || is_declaration_file(Path::new(uri))
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Barrier};

    use tower_lsp::lsp_types::Url;

    use super::{ServerState, collect_global_component_declarations};

    const DECLARATION: &str =
        "declare module 'vue' { interface GlobalComponents { NuxtCard: unknown } }";

    #[test]
    fn finds_hidden_project_augmentations_without_scanning_dependencies() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".nuxt")).unwrap();
        std::fs::create_dir_all(root.path().join("node_modules/pkg")).unwrap();
        std::fs::write(root.path().join("components.d.ts"), DECLARATION).unwrap();
        std::fs::write(root.path().join(".nuxt/components.d.ts"), DECLARATION).unwrap();
        std::fs::write(
            root.path().join("node_modules/pkg/global.d.ts"),
            DECLARATION,
        )
        .unwrap();
        std::fs::write(root.path().join("unrelated.d.ts"), "interface Plain {}\n").unwrap();

        let paths = collect_global_component_declarations(root.path());
        assert_eq!(paths.len(), 2, "{paths:?}");
        assert!(paths.iter().any(|path| path.ends_with("components.d.ts")));
        assert!(
            paths
                .iter()
                .any(|path| path.ends_with(".nuxt/components.d.ts"))
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.starts_with(root.path().join("node_modules")))
        );
    }

    #[test]
    fn concurrent_cold_lookups_share_one_background_scan() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("components.d.ts"), DECLARATION).unwrap();
        let state = Arc::new(ServerState::new());
        state.set_workspace_root(root.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(3));
        let callers = (0..2)
            .map(|_| {
                let state = state.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    crate::runtime::block_on(state.global_component_reference_paths())
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();

        for caller in callers {
            assert_eq!(caller.join().unwrap().len(), 1);
        }
        assert_eq!(
            state
                .global_component_references
                .scan_count
                .load(Ordering::Acquire),
            1
        );
        assert_ne!(
            *state.global_component_references.last_scan_thread.lock(),
            Some(std::thread::current().id())
        );
    }

    #[test]
    fn declaration_file_events_refresh_created_renamed_and_deleted_paths() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".nuxt")).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        assert!(crate::runtime::block_on(state.global_component_reference_paths()).is_empty());

        let created = root.path().join(".nuxt/components.d.ts");
        std::fs::write(&created, DECLARATION).unwrap();
        let created_uri = Url::from_file_path(&created).unwrap();
        assert!(state.invalidate_global_component_references([created_uri.as_str()]));
        assert_eq!(
            crate::runtime::block_on(state.global_component_reference_paths()),
            [std::fs::canonicalize(&created).unwrap()]
        );

        let renamed = root.path().join(".nuxt/components.d.mts");
        std::fs::rename(&created, &renamed).unwrap();
        let renamed_uri = Url::from_file_path(&renamed).unwrap();
        assert!(
            state.invalidate_global_component_references([
                created_uri.as_str(),
                renamed_uri.as_str(),
            ])
        );
        assert_eq!(
            crate::runtime::block_on(state.global_component_reference_paths()),
            [std::fs::canonicalize(&renamed).unwrap()]
        );

        std::fs::remove_file(&renamed).unwrap();
        assert!(state.invalidate_global_component_references([renamed_uri.as_str()]));
        assert!(crate::runtime::block_on(state.global_component_reference_paths()).is_empty());
        assert!(!state.invalidate_global_component_references(["file:///workspace/App.vue"]));
        assert_eq!(
            state
                .global_component_references
                .scan_count
                .load(Ordering::Acquire),
            4
        );
    }
}
