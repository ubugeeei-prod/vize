//! Project-reference ownership for discovered projects (#13090).
//!
//! The nearest `tsconfig.json` of a create-vue app is a solution shell
//! (`files: []` plus `references`) that declares no `paths`; the alias lives in
//! the referenced `tsconfig.app.json`. TypeScript, `vue-tsc` and Vue's own
//! compiler give a source the options of the referenced project that owns it,
//! so discovery does the same instead of capturing the shell.
#![expect(
    clippy::disallowed_types,
    reason = "captured reference graphs are shared across parallel SFC workers"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, LazyLock, RwLock};

use vize_carton::typescript_ownership::{ProjectReferenceGraph, TsconfigSourceKind};
use vize_carton::{FxHashMap, String};

use super::super::super::super::batch_epoch::current_batch_epoch;
use super::host::FileHost;
use super::{Observations, fresh};

struct Capture {
    graph: ProjectReferenceGraph,
    observations: Observations,
    epoch: AtomicU64,
}

static GRAPHS: LazyLock<RwLock<FxHashMap<PathBuf, Arc<Capture>>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

fn capture(shell: &Path) -> Result<Arc<Capture>, String> {
    let epoch = current_batch_epoch();
    let cached = GRAPHS
        .read()
        .map_err(|_| String::from("TypeScript project reference cache unavailable"))?
        .get(shell)
        .cloned();
    if let Some(entry) = cached.filter(|entry| fresh(&entry.observations, &entry.epoch, epoch)) {
        return Ok(entry);
    }
    let host = FileHost::default();
    let graph = ProjectReferenceGraph::load(shell, &host);
    let entry = Arc::new(Capture {
        graph,
        observations: host.observations(),
        epoch: AtomicU64::new(epoch),
    });
    GRAPHS
        .write()
        .map_err(|_| String::from("TypeScript project reference cache unavailable"))?
        .insert(shell.to_path_buf(), Arc::clone(&entry));
    Ok(entry)
}

/// The project of `shell`'s reference graph that owns `file`: the unique
/// referenced owner, otherwise `shell` itself. A config without references
/// selects itself after one read of its own bytes.
pub(super) fn owner(shell: &Path, file: &Path) -> Result<PathBuf, String> {
    let capture = capture(shell)?;
    if !capture.graph.has_references() {
        return Ok(shell.to_path_buf());
    }
    Ok(capture
        .graph
        .select(file, TsconfigSourceKind::for_path(file))
        .filter(|selected| capture.graph.projects().first() != Some(selected))
        .unwrap_or_else(|| shell.to_path_buf()))
}

/// Every config read or probed while selecting an owner under a solution
/// `shell`. A config without references is already a discovery input.
pub(super) fn inputs(shell: &Path) -> Result<Vec<PathBuf>, String> {
    let capture = capture(shell)?;
    if !capture.graph.has_references() {
        return Ok(Vec::new());
    }
    Ok(capture
        .observations
        .iter()
        .map(|(path, _)| path.clone())
        .collect())
}
