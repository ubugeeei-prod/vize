//! The referenced project that owns a source (#13090).
//!
//! A solution-style `tsconfig.json` gives each source the options of the
//! referenced project whose effective `files` / `include` / `exclude` admits
//! it, exactly as the compiler, linter and checker select it through the
//! shared `vize_l0::typescript_ownership` authority.
//!
//! `project_paths` runs for every non-relative import of a document, so the
//! reference graph is captured once per solution and each importer's owner
//! once per capture. A capture is reused while every config it read or probed
//! keeps its modification time and length.
#![expect(
    clippy::disallowed_types,
    reason = "captured reference graphs are shared across concurrent editor requests"
)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::SystemTime;

use serde_json::Value;
use vize_l0::FxHashMap;
use vize_l0::typescript_ownership::{
    ProjectReferenceGraph, TsconfigSourceKind, normalize_path, source_spellings,
};
use vize_l0::typescript_project::{CompilerConfigHost, parse_jsonc_value};

type Stamp = Option<(Option<SystemTime>, u64)>;

fn stamp(path: &Path) -> Stamp {
    std::fs::metadata(path)
        .ok()
        .map(|metadata| (metadata.modified().ok(), metadata.len()))
}

/// Direct reads of the user's configs that remember every path consulted;
/// an unreadable config owns nothing.
#[derive(Default)]
struct FileHost {
    observed: RefCell<Vec<PathBuf>>,
}

impl CompilerConfigHost for FileHost {
    type Error = ();

    fn read_jsonc(&self, path: &Path) -> Result<Value, ()> {
        let source = std::fs::read_to_string(path).map_err(|_| ())?;
        parse_jsonc_value(&source).map_err(|_| ())
    }

    fn configuration_error(&self, _message: vize_l0::String) {}

    fn observe(&self, path: &Path) {
        self.observed.borrow_mut().push(path.to_path_buf());
    }
}

struct Capture {
    graph: ProjectReferenceGraph,
    stamps: Vec<(PathBuf, Stamp)>,
    /// Owners keyed by every spelling of the importer, so retargeting a
    /// symlinked directory on its path selects again.
    owners: Mutex<FxHashMap<Vec<PathBuf>, Option<PathBuf>>>,
}

impl Capture {
    fn fresh(&self) -> bool {
        self.stamps.iter().all(|(path, seen)| stamp(path) == *seen)
    }
}

static CAPTURES: LazyLock<Mutex<FxHashMap<PathBuf, Arc<Capture>>>> = LazyLock::new(Mutex::default);

fn capture(shell: &Path) -> Option<Arc<Capture>> {
    let cached = CAPTURES.lock().ok()?.get(shell).cloned();
    if let Some(capture) = cached.filter(|capture| capture.fresh()) {
        return Some(capture);
    }
    let host = FileHost::default();
    let graph = ProjectReferenceGraph::load(shell, &host);
    let mut observed = host.observed.into_inner();
    observed.push(shell.to_path_buf());
    observed.sort();
    observed.dedup();
    let capture = Arc::new(Capture {
        graph,
        stamps: observed
            .into_iter()
            .map(|path| {
                let seen = stamp(&path);
                (path, seen)
            })
            .collect(),
        owners: Mutex::default(),
    });
    CAPTURES
        .lock()
        .ok()?
        .insert(shell.to_path_buf(), Arc::clone(&capture));
    Some(capture)
}

/// The unique referenced project of `shell` that owns `source`, spelled under
/// the shell's directory as the editor spells it. `None` when `shell` has no
/// references, or when no project or several projects admit the source.
pub(super) fn owning_reference(shell: &Path, source: &Path) -> Option<PathBuf> {
    let capture = capture(shell)?;
    if !capture.graph.has_references() {
        return None;
    }
    let spellings = source_spellings(source);
    if let Some(owner) = capture.owners.lock().ok()?.get(&spellings) {
        return owner.clone();
    }
    let owner = select(&capture.graph, shell, &spellings, source);
    capture.owners.lock().ok()?.insert(spellings, owner.clone());
    owner
}

fn select(
    graph: &ProjectReferenceGraph,
    shell: &Path,
    spellings: &[PathBuf],
    source: &Path,
) -> Option<PathBuf> {
    let owner = graph.select_spellings(spellings, TsconfigSourceKind::for_path(source))?;
    if graph.projects().first() == Some(&owner) {
        return None;
    }
    let shell_dir = shell.parent()?;
    Some(
        owner
            .strip_prefix(normalize_path(shell_dir))
            .map_or_else(|_| owner.clone(), |relative| shell_dir.join(relative)),
    )
}
