//! Demand-loaded source paths for project-wide navigation. Content is never cached.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use futures::{channel::oneshot, lock::Mutex};
use ignore::WalkBuilder;
use parking_lot::RwLock;
use tower_lsp::lsp_types::{FileChangeType, FileEvent, Url};
use vize_l0::FxHashMap;

use super::{ServerState, global_components::is_excluded_directory};

#[derive(Default)]
pub(super) struct Inventory {
    generation: AtomicU64,
    paths: RwLock<Option<CachedPaths>>,
    scan: Mutex<()>,
}

struct CachedPaths {
    generation: u64,
    roots: Vec<PathBuf>,
    uris: Vec<Url>,
}

impl Inventory {
    fn cached_paths(&self, roots: &[PathBuf], generation: u64) -> Option<Vec<Url>> {
        self.paths
            .read()
            .as_ref()
            .filter(|cached| cached.generation == generation && cached.roots == roots)
            .map(|cached| cached.uris.clone())
    }
}

fn read_sources(uris: &[Url]) -> Vec<(Url, std::string::String)> {
    uris.iter()
        .filter_map(|uri| {
            let source = std::fs::read_to_string(uri.to_file_path().ok()?).ok()?;
            Some((uri.clone(), source))
        })
        .collect()
}

impl ServerState {
    pub(crate) fn project_source_watcher_enabled(&self) -> bool {
        self.is_lsp_typecheck_enabled() || self.lsp_features().workspace_symbols
    }

    pub(crate) fn observe_workspace_project_file_events(&self, events: &[FileEvent]) {
        if events
            .iter()
            .any(|event| event.typ != FileChangeType::CHANGED)
        {
            self.invalidate_workspace_project_files();
        }
    }

    pub(crate) fn invalidate_workspace_project_files(&self) {
        self.workspace_project_files
            .generation
            .fetch_add(1, Ordering::AcqRel);
    }

    /// Only references and symbol requests pay for this inventory. A snapshot
    /// contains paths, not text; closed-file edits are read afresh and unsaved
    /// buffers always win. Roots and file events retire the path snapshot.
    pub(crate) async fn discover_workspace_project_sources(
        &self,
    ) -> Vec<(Url, std::string::String)> {
        let inventory = &self.workspace_project_files;
        let sources = loop {
            let mut roots = self.workspace_root_paths();
            roots.sort();
            roots.dedup();
            let generation = inventory.generation.load(Ordering::Acquire);
            if let Some(uris) = inventory.cached_paths(&roots, generation) {
                break background(move || read_sources(&uris))
                    .await
                    .unwrap_or_default();
            }
            let _scan = inventory.scan.lock().await;
            if inventory.generation.load(Ordering::Acquire) != generation {
                continue;
            }
            if let Some(uris) = inventory.cached_paths(&roots, generation) {
                break background(move || read_sources(&uris))
                    .await
                    .unwrap_or_default();
            }
            let scan_roots = roots.clone();
            let Some((uris, sources)) = background(move || {
                let uris = discover_paths(&scan_roots);
                let sources = read_sources(&uris);
                (uris, sources)
            })
            .await
            else {
                break Vec::new();
            };
            if inventory.generation.load(Ordering::Acquire) != generation {
                continue;
            }
            let mut current_roots = self.workspace_root_paths();
            current_roots.sort();
            current_roots.dedup();
            if current_roots != roots {
                continue;
            }
            *inventory.paths.write() = Some(CachedPaths {
                generation,
                roots,
                uris,
            });
            break sources;
        };
        let mut sources = sources.into_iter().collect::<FxHashMap<_, _>>();
        for uri in self.documents.uris() {
            if uri
                .to_file_path()
                .is_ok_and(|path| is_project_source(&path))
                && let Some(source) = self.documents.text(&uri)
            {
                sources.insert(uri, source);
            }
        }
        let mut sources = sources.into_iter().collect::<Vec<_>>();
        sources.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
        sources
    }
}

fn is_project_source(path: &Path) -> bool {
    !vize_l0::path::is_git_metadata_path(path)
        && matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs")
        )
}

fn discover_paths(roots: &[PathBuf]) -> Vec<Url> {
    let mut uris = Vec::new();
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
                !vize_l0::path::is_git_metadata_path(entry.path())
                    && (!entry.file_type().is_some_and(|kind| kind.is_dir())
                        || !is_excluded_directory(entry.file_name()))
            });
        uris.extend(
            builder
                .build()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
                .filter(|entry| is_project_source(entry.path()))
                .filter_map(|entry| Url::from_file_path(entry.path()).ok()),
        );
    }
    uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    uris.dedup();
    uris
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
mod tests;
