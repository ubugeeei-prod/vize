//! Reusable paths only; source consumers choose their buffer lifetime.

use std::path::PathBuf;
use std::sync::atomic::Ordering;

use tower_lsp::lsp_types::Url;

use super::{CachedPaths, ServerState, background, discover_paths};

pub(super) struct ProjectPaths {
    pub(super) generation: u64,
    pub(super) roots: Vec<PathBuf>,
    pub(super) uris: Vec<Url>,
}

impl ServerState {
    pub(super) async fn current_project_paths(&self) -> ProjectPaths {
        let inventory = &self.workspace_project_files;
        loop {
            let roots = self.project_source_roots();
            let generation = inventory.generation.load(Ordering::Acquire);
            if let Some(uris) = inventory.cached_paths(&roots, generation) {
                return ProjectPaths {
                    generation,
                    roots,
                    uris,
                };
            }
            let _scan = inventory.scan.lock().await;
            if !self.project_source_snapshot_current(&roots, generation) {
                continue;
            }
            if let Some(uris) = inventory.cached_paths(&roots, generation) {
                return ProjectPaths {
                    generation,
                    roots,
                    uris,
                };
            }
            let scan_roots = roots.clone();
            let (uris, complete) = background(move || discover_paths(&scan_roots))
                .await
                .unwrap_or_default();
            if !self.project_source_snapshot_current(&roots, generation) {
                continue;
            }
            // A partial walk still returns all available sources, but must
            // retry discovery next time even without a file event.
            if complete {
                *inventory.paths.write() = Some(CachedPaths {
                    generation,
                    roots: roots.clone(),
                    uris: uris.clone(),
                });
            }
            return ProjectPaths {
                generation,
                roots,
                uris,
            };
        }
    }
}
