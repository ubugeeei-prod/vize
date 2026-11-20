//! Demand-loaded source snapshots for project-wide navigation.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::{channel::oneshot, lock::Mutex};
use ignore::WalkBuilder;
use parking_lot::RwLock;
use tower_lsp::lsp_types::{FileChangeType, FileEvent, Url};
use vize_l0::FxHashMap;

use super::{ServerState, global_components::is_excluded_directory};

mod paths;
mod symbols;
mod worker;

#[derive(Default)]
pub(super) struct Inventory {
    generation: AtomicU64,
    content_generation: AtomicU64,
    paths: RwLock<Option<CachedPaths>>,
    sources: RwLock<Option<CachedSources>>,
    retired: RwLock<Vec<PathBuf>>,
    scan: Mutex<()>,
    worker: worker::Worker,
    #[cfg(test)]
    symbol_worker_failure: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    read_pause: RwLock<Option<(oneshot::Sender<()>, oneshot::Receiver<()>)>>,
}

struct CachedPaths {
    generation: u64,
    roots: Vec<PathBuf>,
    uris: Arc<[Url]>,
}

struct CachedSources {
    generation: u64,
    content_generation: u64,
    roots: Vec<PathBuf>,
    sources: Arc<[(Url, Arc<String>)]>,
}

impl Inventory {
    fn cached_paths(&self, roots: &[PathBuf], generation: u64) -> Option<Arc<[Url]>> {
        self.paths
            .read()
            .as_ref()
            .filter(|cached| cached.generation == generation && cached.roots == roots)
            .map(|cached| Arc::clone(&cached.uris))
    }

    fn cached_sources(
        &self,
        roots: &[PathBuf],
        generation: u64,
        content_generation: u64,
    ) -> Option<Vec<(Url, Arc<String>)>> {
        self.sources
            .read()
            .as_ref()
            .filter(|cached| {
                cached.generation == generation
                    && cached.content_generation == content_generation
                    && cached.roots == roots
            })
            .map(|cached| cached.sources.iter().cloned().collect())
    }
}

fn read_sources(uris: &[Url], retired: &[PathBuf]) -> (Vec<(Url, Arc<String>)>, bool) {
    let mut complete = true;
    let sources = uris
        .iter()
        .filter_map(|uri| {
            let path = uri.to_file_path().ok()?;
            if retired.iter().any(|prefix| path.starts_with(prefix)) {
                return None;
            }
            match std::fs::read_to_string(path) {
                Ok(source) => Some((uri.clone(), Arc::new(source))),
                Err(_) => {
                    complete = false;
                    None
                }
            }
        })
        .collect();
    (sources, complete)
}

impl ServerState {
    pub(crate) fn observe_workspace_project_file_events(&self, events: &[FileEvent]) {
        for event in events {
            if event.typ == FileChangeType::CHANGED {
                self.invalidate_workspace_project_source_contents();
            } else if let Ok(path) = event.uri.to_file_path() {
                self.observe_workspace_project_membership(
                    &path,
                    event.typ == FileChangeType::CREATED,
                );
            }
        }
    }

    /// File-operation notifications retire the old namespace even before the
    /// filesystem view catches up. A later explicit creation restores it.
    pub(crate) fn observe_workspace_project_membership(&self, path: &Path, created: bool) {
        let mut retired = self.workspace_project_files.retired.write();
        if created {
            retired.retain(|prefix| !path.starts_with(prefix) && !prefix.starts_with(path));
        } else if !retired.iter().any(|prefix| path.starts_with(prefix)) {
            retired.retain(|prefix| !prefix.starts_with(path));
            retired.push(path.to_owned());
        }
        self.invalidate_workspace_project_files();
    }

    pub(crate) fn invalidate_workspace_project_files(&self) {
        self.workspace_project_files
            .generation
            .fetch_add(1, Ordering::AcqRel);
        // Membership changes can retire arbitrary directory subtrees. Drop
        // text-free disk indexes eagerly so deleted namespaces cannot
        // accumulate; open buffers rebuild lazily from their resident Arcs.
        self.workspace_symbol_cache.clear();
        self.invalidate_workspace_project_source_contents();
    }

    pub(crate) fn invalidate_workspace_project_source_contents(&self) {
        self.workspace_project_files
            .content_generation
            .fetch_add(1, Ordering::AcqRel);
    }

    /// Only references and symbol requests pay for this inventory. A snapshot
    /// shares immutable closed-file text only after the client has confirmed
    /// the workspace watcher registration. Unsaved buffers always win. Clients
    /// without watcher support retain the fresh-read behavior.
    #[cfg(test)]
    pub(crate) async fn discover_workspace_project_sources(
        &self,
    ) -> Vec<(Url, std::string::String)> {
        self.discover_workspace_project_source_snapshots()
            .await
            .into_iter()
            .map(|(uri, source)| (uri, source.as_ref().to_owned()))
            .collect()
    }

    pub(crate) async fn discover_workspace_project_source_snapshots(
        &self,
    ) -> Vec<(Url, Arc<String>)> {
        let inventory = &self.workspace_project_files;
        let sources = loop {
            let roots = self.project_source_roots();
            let generation = inventory.generation.load(Ordering::Acquire);
            let content_generation = inventory.content_generation.load(Ordering::Acquire);
            let reuse_sources = self.corsa_disk_watcher_registered.load(Ordering::Acquire);
            if reuse_sources
                && let Some(sources) =
                    inventory.cached_sources(&roots, generation, content_generation)
            {
                break sources;
            }
            if let Some(uris) = inventory.cached_paths(&roots, generation) {
                if let Some((sources, complete)) = self
                    .read_current_project_sources(uris, &roots, generation, content_generation)
                    .await
                {
                    if reuse_sources && complete {
                        self.cache_workspace_project_sources(
                            &roots,
                            generation,
                            content_generation,
                            &sources,
                        );
                    }
                    break sources;
                }
                continue;
            }
            let _scan = inventory.scan.lock().await;
            if inventory.generation.load(Ordering::Acquire) != generation {
                continue;
            }
            if let Some(uris) = inventory.cached_paths(&roots, generation) {
                if let Some((sources, complete)) = self
                    .read_current_project_sources(uris, &roots, generation, content_generation)
                    .await
                {
                    if reuse_sources && complete {
                        self.cache_workspace_project_sources(
                            &roots,
                            generation,
                            content_generation,
                            &sources,
                        );
                    }
                    break sources;
                }
                continue;
            }
            let scan_roots = roots.clone();
            let retired = inventory.retired.read().clone();
            let Some((uris, sources, walk_complete, reads_complete)) = background(move || {
                let (uris, walk_complete) = discover_paths(&scan_roots);
                let (sources, reads_complete) = read_sources(&uris, &retired);
                (uris, sources, walk_complete, reads_complete)
            })
            .await
            else {
                break Vec::new();
            };
            if !self.project_source_content_snapshot_current(&roots, generation, content_generation)
            {
                continue;
            }
            if walk_complete {
                *inventory.paths.write() = Some(CachedPaths {
                    generation,
                    roots: roots.clone(),
                    uris: Arc::from(uris),
                });
            }
            if reuse_sources && walk_complete && reads_complete {
                self.cache_workspace_project_sources(
                    &roots,
                    generation,
                    content_generation,
                    &sources,
                );
            }
            break sources;
        };
        let mut sources = sources.into_iter().collect::<FxHashMap<_, _>>();
        for uri in self.documents.uris().iter() {
            if uri
                .to_file_path()
                .is_ok_and(|path| is_project_source(&path))
                && let Some(source) = self.documents.text_snapshot(uri)
            {
                sources.insert(uri.clone(), source);
            }
        }
        let mut sources = sources.into_iter().collect::<Vec<_>>();
        sources.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
        sources
    }

    fn project_source_roots(&self) -> Vec<PathBuf> {
        let mut roots = self.workspace_root_paths();
        roots.sort();
        roots.dedup();
        roots
    }

    fn project_source_snapshot_current(&self, roots: &[PathBuf], generation: u64) -> bool {
        self.workspace_project_files
            .generation
            .load(Ordering::Acquire)
            == generation
            && self.project_source_roots() == roots
    }

    fn project_source_content_snapshot_current(
        &self,
        roots: &[PathBuf],
        generation: u64,
        content_generation: u64,
    ) -> bool {
        self.project_source_snapshot_current(roots, generation)
            && self
                .workspace_project_files
                .content_generation
                .load(Ordering::Acquire)
                == content_generation
    }

    fn cache_workspace_project_sources(
        &self,
        roots: &[PathBuf],
        generation: u64,
        content_generation: u64,
        sources: &[(Url, Arc<String>)],
    ) {
        if !self.project_source_content_snapshot_current(roots, generation, content_generation) {
            return;
        }
        *self.workspace_project_files.sources.write() = Some(CachedSources {
            generation,
            content_generation,
            roots: roots.to_vec(),
            sources: Arc::from(sources),
        });
    }

    async fn read_current_project_sources(
        &self,
        uris: Arc<[Url]>,
        roots: &[PathBuf],
        generation: u64,
        content_generation: u64,
    ) -> Option<(Vec<(Url, Arc<String>)>, bool)> {
        let retired = self.workspace_project_files.retired.read().clone();
        let sources = background(move || read_sources(&uris, &retired))
            .await
            .unwrap_or_default();
        #[cfg(test)]
        {
            let pause = self.workspace_project_files.read_pause.write().take();
            if let Some((reached, resume)) = pause {
                let _ = reached.send(());
                let _ = resume.await;
            }
        }
        self.project_source_content_snapshot_current(roots, generation, content_generation)
            .then_some(sources)
    }
}

fn is_project_source(path: &Path) -> bool {
    !vize_carton::path::is_git_metadata_path(path)
        && matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs")
        )
}

fn discover_paths(roots: &[PathBuf]) -> (Vec<Url>, bool) {
    let mut uris = Vec::new();
    let mut complete = true;
    for root in roots {
        let mut builder = WalkBuilder::new(root);
        builder
            .hidden(false)
            .ignore(false)
            .git_global(false)
            .git_ignore(false)
            .git_exclude(false)
            .parents(false)
            .follow_links(false)
            .filter_entry(|entry| {
                !vize_carton::path::is_git_metadata_path(entry.path())
                    && (!entry.file_type().is_some_and(|kind| kind.is_dir())
                        || !is_excluded_directory(entry.file_name()))
            });
        for entry in builder.build() {
            let Ok(entry) = entry else {
                complete = false;
                continue;
            };
            if entry.file_type().is_some_and(|kind| kind.is_file())
                && is_project_source(entry.path())
                && let Ok(uri) = Url::from_file_path(entry.path())
            {
                uris.push(uri);
            }
        }
    }
    uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    uris.dedup();
    (uris, complete)
}

async fn background<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (sender, receiver) = oneshot::channel();
    if let Err(error) = std::thread::Builder::new()
        .name("vize-project-navigation".into())
        .spawn(move || {
            let _ = sender.send(work());
        })
    {
        tracing::warn!("failed to spawn project navigation discovery: {error}");
        return None;
    }
    receiver.await.ok()
}

#[cfg(test)]
mod membership_tests;
#[cfg(test)]
mod tests;
