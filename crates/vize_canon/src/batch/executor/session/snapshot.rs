//! Materialized file revisions and exact diagnostic-input membership.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String, hash::hash_bytes};

use super::{VirtualProject, is_internal_virtual_project_stub, is_under_virtual_node_modules};
use crate::batch::error::CorsaResult;
use crate::batch::executor::session::diagnostic_paths::is_authored_diagnostic_input;
use crate::batch::source_policy::SourceFilePolicy;
use crate::batch::virtual_project::MaterializedFileDelta;
use crate::file_uri::path_to_file_uri;

#[derive(Default)]
pub(super) struct MaterializedSnapshot {
    pub(super) revisions: FxHashMap<PathBuf, u64>,
    pub(super) uris: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct MaterializedDelta {
    pub(super) changed: Vec<PathBuf>,
    pub(super) created: Vec<PathBuf>,
    pub(super) deleted: Vec<PathBuf>,
}

impl MaterializedSnapshot {
    pub(super) fn capture(
        virtual_root: &Path,
        source_policy: SourceFilePolicy,
    ) -> CorsaResult<Self> {
        let mut snapshot = Self::default();
        for entry in walkdir::WalkDir::new(virtual_root) {
            let entry = entry?;
            let path = entry.path();
            let Some(captured) = capture_path_revision(path)? else {
                continue;
            };
            snapshot.revisions.insert(path.to_path_buf(), captured.hash);
            if captured.resolves_to_file
                && source_policy.accepts_diagnostic_input(path)
                && !is_under_virtual_node_modules(virtual_root, path)
                && !is_internal_virtual_project_stub(path)
            {
                snapshot.uris.push(path_to_file_uri(path));
            }
        }
        snapshot.uris.sort();
        Ok(snapshot)
    }

    pub(super) fn diff(&self, previous: &Self) -> MaterializedDelta {
        let mut delta = MaterializedDelta::default();
        for (path, revision) in &self.revisions {
            match previous.revisions.get(path) {
                None => delta.created.push(path.clone()),
                Some(previous) if previous != revision => delta.changed.push(path.clone()),
                Some(_) => {}
            }
        }
        for path in previous.revisions.keys() {
            if !self.revisions.contains_key(path) {
                delta.deleted.push(path.clone());
            }
        }
        delta.sort();
        delta
    }

    pub(super) fn extend_diagnostic_paths(&mut self, project: &VirtualProject) -> CorsaResult<()> {
        for path in project.diagnostic_paths_sorted() {
            if !path.is_file() || !is_authored_diagnostic_input(&path) {
                continue;
            }
            let content = std::fs::read(&path)?;
            self.revisions.insert(path.clone(), hash_bytes(&content));
            self.uris.push(path_to_file_uri(&path));
        }
        self.uris.sort();
        self.uris.dedup();
        Ok(())
    }

    pub(super) fn apply_delta(
        &mut self,
        project: &VirtualProject,
        delta: &MaterializedFileDelta,
    ) -> CorsaResult<()> {
        for path in &delta.deleted {
            self.revisions.remove(path);
            let uri = path_to_file_uri(path);
            self.uris.retain(|candidate| candidate != uri);
        }
        for path in delta.changed.iter().chain(&delta.created) {
            let Some(captured) = capture_path_revision(path)? else {
                self.revisions.remove(path);
                let uri = path_to_file_uri(path);
                self.uris.retain(|candidate| candidate != uri);
                continue;
            };
            self.revisions.insert(path.clone(), captured.hash);
            if captured.resolves_to_file
                && project.source_file_policy().accepts_diagnostic_input(path)
                && !is_under_virtual_node_modules(project.virtual_root(), path)
                && !is_internal_virtual_project_stub(path)
            {
                self.uris.push(path_to_file_uri(path));
            }
        }
        self.uris.sort();
        self.uris.dedup();
        Ok(())
    }
}

struct CapturedPathRevision {
    hash: u64,
    resolves_to_file: bool,
}

fn capture_path_revision(path: &Path) -> CorsaResult<Option<CapturedPathRevision>> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(path)?;
        let resolves_to_file = path.is_file();
        let hash = if resolves_to_file {
            hash_bytes(&std::fs::read(path)?)
        } else {
            hash_path(&target)
        };
        return Ok(Some(CapturedPathRevision {
            hash,
            resolves_to_file,
        }));
    }
    if metadata.is_file() {
        return Ok(Some(CapturedPathRevision {
            hash: hash_bytes(&std::fs::read(path)?),
            resolves_to_file: true,
        }));
    }
    Ok(None)
}

impl MaterializedDelta {
    pub(super) fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.created.is_empty() && self.deleted.is_empty()
    }

    fn sort(&mut self) {
        self.changed.sort();
        self.created.sort();
        self.deleted.sort();
    }
}

fn hash_path(path: &Path) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}
