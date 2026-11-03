//! Differential-corpus root resolution and closure-evidence gating.
//!
//! Every corpus-runnable Davinci lane reads
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` and sweeps the `.vue` files under
//! `<dir>`. Only this checkout's own `tests/_fixtures/_git` root can count as
//! Phase 2 closure evidence, and only when its indexed mode-160000 gitlinks
//! reconcile exactly with `git submodule status`: a missing, drifted,
//! conflicted, invalid, empty, or mismatched inventory fails closed before
//! any Vue file is collected. Every other root — including an external
//! checkout that merely ends in `tests/_fixtures/_git` — is swept in
//! smoke/shard scope with `closure_evidence=false`.

mod inventory;
mod shard;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod io_tests;

#[cfg(all(test, unix))]
mod symlink_tests;

pub use inventory::{
    IndexedGitlinks, InventoryError, SubmoduleState, parse_indexed_gitlinks,
    parse_submodule_status, reconcile,
};
pub use shard::{CorpusShard, SHARD_ENV, SHARD_VECTOR_ENV, logical_vector, resolve_env_shard};

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

use vize_l0::CompactString;

/// The canonical corpus root, relative to the workspace root.
pub const CANONICAL_CORPUS_RELATIVE: &str = "tests/_fixtures/_git";

/// The environment variable naming the corpus root to sweep.
pub const CORPUS_ENV: &str = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS";

/// How much a sweep over a corpus root is allowed to prove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusScope {
    /// This checkout's own fixture tree with a fully reconciled submodule
    /// inventory; the only scope that is closure evidence.
    Canonical { submodules: usize },
    /// Any other root: a smoke/shard sweep, never closure evidence.
    Smoke,
}

/// A validated corpus root and the Vue files under it.
#[derive(Debug)]
pub struct CorpusSweep {
    /// The resolved corpus root directory.
    pub root: PathBuf,
    /// What this sweep is allowed to prove.
    pub scope: CorpusScope,
    /// Every `.vue` file under the root, sorted, `node_modules` excluded.
    pub files: Vec<PathBuf>,
}

impl CorpusSweep {
    /// Whether this sweep may be reported as differential-corpus closure
    /// evidence.
    #[must_use]
    pub fn closure_evidence(&self) -> bool {
        matches!(self.scope, CorpusScope::Canonical { .. })
    }

    /// Stable scope label for report lines.
    #[must_use]
    pub fn scope_label(&self) -> &'static str {
        match self.scope {
            CorpusScope::Canonical { .. } => "canonical",
            CorpusScope::Smoke => "smoke",
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Whether `root` is this checkout's own canonical fixture root.
///
/// Compares fully canonicalized paths, so an external checkout with the same
/// `tests/_fixtures/_git` suffix never classifies as canonical.
#[must_use]
pub fn is_canonical_root(root: &Path) -> bool {
    let canonical_root = workspace_root().join(CANONICAL_CORPUS_RELATIVE);
    match (fs::canonicalize(root), fs::canonicalize(&canonical_root)) {
        (Ok(resolved), Ok(canonical)) => resolved == canonical,
        _ => false,
    }
}

fn git_stdout(workspace: &Path, args: &[&str]) -> CompactString {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to spawn git {args:?}: {error}"));
    assert!(
        output.status.success(),
        "git {:?} failed under {}: {}",
        args,
        workspace.display(),
        core::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>"),
    );
    core::str::from_utf8(&output.stdout)
        .unwrap_or_else(|error| panic!("git {args:?} produced non-UTF-8 output: {error}"))
        .into()
}

/// Reconcile the canonical fixture inventory of the checkout at `workspace`.
///
/// Panics with a deterministic diagnostic unless every indexed gitlink under
/// [`CANONICAL_CORPUS_RELATIVE`] is hydrated at its indexed commit.
pub fn require_reconciled_canonical_inventory(workspace: &Path) -> usize {
    let stage = git_stdout(
        workspace,
        &["ls-files", "--stage", "-z", "--", CANONICAL_CORPUS_RELATIVE],
    );
    let status = git_stdout(
        workspace,
        &["submodule", "status", "--", CANONICAL_CORPUS_RELATIVE],
    );
    let indexed = parse_indexed_gitlinks(&stage)
        .unwrap_or_else(|error| panic!("canonical corpus inventory rejected: {error}"));
    let states = parse_submodule_status(&status)
        .unwrap_or_else(|error| panic!("canonical corpus inventory rejected: {error}"));
    reconcile(&indexed, &states)
        .unwrap_or_else(|error| panic!("canonical corpus inventory rejected: {error}"))
}

fn fail_corpus_io(operation: &str, path: &Path, error: &io::Error) -> ! {
    panic!(
        "corpus IO failed: {operation} {}: {error} (kind={:?}, raw_os_error={:?})",
        path.display(),
        error.kind(),
        error.raw_os_error(),
    );
}

fn corpus_entry_path(root: &Path, entry: io::Result<fs::DirEntry>) -> PathBuf {
    entry
        .unwrap_or_else(|error| fail_corpus_io("read directory entry under", root, &error))
        .path()
}

/// Read the complete UTF-8 source of a selected corpus file.
///
/// # Panics
///
/// Fails the calling test with the path and original IO error when a selected
/// file cannot be read; unreadable files must never become omitted evidence.
pub fn read_corpus_source(file: &Path) -> CompactString {
    // Retain every logical alias in the collected vector, but do not send the
    // whole alias chain through one kernel pathname lookup. Resolve each
    // component before adding the next; canonicalize still rejects a genuine
    // self-referential link. There is one source read and no error retry.
    let mut physical = PathBuf::new();
    for component in file.components() {
        physical.push(component);
        if matches!(component, std::path::Component::Prefix(_)) {
            continue;
        }
        physical = fs::canonicalize(&physical)
            .unwrap_or_else(|error| fail_corpus_io("read source", file, &error));
    }
    fs::read_to_string(&physical)
        .unwrap_or_else(|error| fail_corpus_io("read source", file, &error))
        .into()
}

/// Collect every `.vue` file under `root`, sorted, skipping `node_modules`
/// trees (they repeat the same shipped sources; sweeps target project code).
///
/// # Panics
///
/// Fails the calling test with the directory and original IO error when
/// enumeration fails, rather than reporting a partial corpus as complete.
pub fn collect_vue_files(root: &Path, out: &mut Vec<PathBuf>) {
    let physical = fs::canonicalize(root)
        .unwrap_or_else(|error| fail_corpus_io("read directory", root, &error));
    collect_vue_edges(root, &physical, &physical, &mut Vec::new(), out);
}

fn collect_vue_edges(
    logical: &Path,
    physical: &Path,
    boundary: &Path,
    edges: &mut Vec<(PathBuf, PathBuf)>,
    out: &mut Vec<PathBuf>,
) {
    let entries = fs::read_dir(physical)
        .unwrap_or_else(|error| fail_corpus_io("read directory", logical, &error));
    let mut children: Vec<PathBuf> = entries
        .map(|entry| corpus_entry_path(logical, entry))
        .collect();
    children.sort();
    for child in children {
        let Some(name) = child.file_name() else {
            panic!("directory entry has no file name: {}", child.display());
        };
        let selected = logical.join(name);
        if child.is_dir() {
            if name == "node_modules" || name == "_git-worktrees" {
                continue;
            }
            let resolved = fs::canonicalize(&child)
                .unwrap_or_else(|error| fail_corpus_io("read directory", &selected, &error));
            if !resolved.starts_with(boundary) {
                continue;
            }
            // Keep a logical alias once, but never repeat the same physical
            // directory edge along a route. Other routes remain independent.
            let edge = (physical.to_path_buf(), resolved);
            if edges.contains(&edge) {
                continue;
            }
            edges.push(edge.clone());
            collect_vue_edges(&selected, &edge.1, boundary, edges, out);
            edges.pop();
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            let resolved = fs::canonicalize(&child)
                .unwrap_or_else(|error| fail_corpus_io("read source", &selected, &error));
            if resolved.starts_with(boundary) {
                out.push(selected);
            }
        }
    }
}

/// Resolve [`CORPUS_ENV`] into a validated, scope-labeled sweep.
///
/// Returns `None` when the variable is unset. A relative root that does not
/// resolve from the current directory is retried against the workspace root,
/// so `tests/_fixtures/_git` works from any crate directory. A canonical root
/// fails closed — before any Vue file is collected — unless its submodule
/// inventory reconciles exactly; every other root is labeled smoke scope.
pub fn resolve_env_sweep() -> Option<CorpusSweep> {
    let root = PathBuf::from(std::env::var_os(CORPUS_ENV)?);
    let root = if root.is_relative() && !root.is_dir() {
        workspace_root().join(&root)
    } else {
        root
    };
    assert!(
        root.is_dir(),
        "{CORPUS_ENV} must name a directory: {}",
        root.display()
    );
    let scope = if is_canonical_root(&root) {
        let submodules = require_reconciled_canonical_inventory(&workspace_root());
        CorpusScope::Canonical { submodules }
    } else {
        CorpusScope::Smoke
    };
    let mut files = Vec::new();
    collect_vue_files(&root, &mut files);
    let sweep = CorpusSweep { root, scope, files };
    match sweep.scope {
        CorpusScope::Canonical { submodules } => eprintln!(
            "davinci-differential corpus scope: root={} scope=canonical closure_evidence=true submodules={submodules}",
            sweep.root.display(),
        ),
        CorpusScope::Smoke => eprintln!(
            "davinci-differential corpus scope: root={} scope=smoke closure_evidence=false",
            sweep.root.display(),
        ),
    }
    Some(sweep)
}
