//! One immutable ownership capture and lazily derived compiler-option views.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock, Weak};
use vize_atelier_sfc::script::SfcTypeProject;
use vize_canon::{TsconfigOwnershipSnapshot, TsconfigSourceKind};
use vize_l0::{FxHashMap, String, cstr};

type ProjectCell = Arc<OnceLock<Result<Arc<SfcTypeProject>, String>>>;
type CaptureCell = Arc<Mutex<Option<Arc<CapturedProject>>>>;
static CAPTURES: LazyLock<Mutex<FxHashMap<PathBuf, CaptureCell>>> = LazyLock::new(Mutex::default);
static NEXT_CAPTURE: AtomicU64 = AtomicU64::new(1);
static OWNERS: LazyLock<Mutex<FxHashMap<PathBuf, Vec<Weak<CapturedProject>>>>> =
    LazyLock::new(Mutex::default);
static CURRENT: LazyLock<Mutex<FxHashMap<PathBuf, Weak<CapturedProject>>>> =
    LazyLock::new(Mutex::default);

pub(super) struct CapturedProject {
    pub root: PathBuf,
    pub revision: u64,
    snapshot: TsconfigOwnershipSnapshot,
    inputs: Vec<PathBuf>,
    projects: Mutex<FxHashMap<PathBuf, ProjectCell>>,
}

impl CapturedProject {
    pub(super) fn get(root: &Path) -> Result<Arc<Self>, String> {
        let cell = CAPTURES
            .lock()
            .map_err(|_| String::from("TypeScript project capture cache unavailable"))?
            .entry(root.to_path_buf())
            .or_default()
            .clone();
        let mut current = cell
            .lock()
            .map_err(|_| String::from("TypeScript project capture unavailable"))?;
        if let Some(capture) = current
            .as_ref()
            .filter(|capture| !capture.snapshot.observations().needs_confirmation())
        {
            return Ok(Arc::clone(capture));
        }
        let snapshot = TsconfigOwnershipSnapshot::capture(root).map_err(|error| {
            cstr!(
                "cannot select TypeScript project {}: {error}",
                root.display()
            )
        })?;
        let capture = Arc::new(Self {
            root: snapshot
                .project_paths()
                .first()
                .cloned()
                .unwrap_or_else(|| root.to_path_buf()),
            revision: NEXT_CAPTURE.fetch_add(1, Ordering::Relaxed),
            inputs: snapshot.observations().paths(),
            snapshot,
            projects: Mutex::default(),
        });
        capture.register()?;
        *current = Some(Arc::clone(&capture));
        Ok(capture)
    }

    pub(super) fn select(&self, source: &Path, kind: TsconfigSourceKind) -> PathBuf {
        self.snapshot.select(source, kind)
    }

    pub(super) fn project(&self, selected: &Path) -> Result<Arc<SfcTypeProject>, String> {
        let cell = self
            .projects
            .lock()
            .map_err(|_| String::from("TypeScript project options cache unavailable"))?
            .entry(selected.to_path_buf())
            .or_default()
            .clone();
        cell.get_or_init(|| {
            let options = self.snapshot.compiler_options(selected).map_err(|error| {
                cstr!(
                    "cannot capture TypeScript project {}: {error}",
                    selected.display()
                )
            })?;
            SfcTypeProject::from_flattened(selected, options)
        })
        .clone()
    }

    fn register(self: &Arc<Self>) -> Result<(), String> {
        {
            let mut owners = OWNERS
                .lock()
                .map_err(|_| String::from("TypeScript project owners unavailable"))?;
            for input in &self.inputs {
                owners
                    .entry(input.clone())
                    .or_default()
                    .push(Arc::downgrade(self));
            }
        }
        CURRENT
            .lock()
            .map_err(|_| String::from("TypeScript project revisions unavailable"))?
            .insert(self.root.clone(), Arc::downgrade(self));
        Ok(())
    }
}

impl Drop for CapturedProject {
    fn drop(&mut self) {
        if let Ok(mut owners) = OWNERS.lock() {
            for input in &self.inputs {
                if let Some(entries) = owners.get_mut(input) {
                    entries.retain(|entry| !std::ptr::eq(entry.as_ptr(), self));
                    if entries.is_empty() {
                        owners.remove(input);
                    }
                }
            }
        }
    }
}

/// Old in-flight captures retain their edges until their final reader exits.
/// This query reads the reverse index; it never scans configuration files.
pub(super) fn owners(path: &Path) -> Vec<PathBuf> {
    let captures: Vec<_> = OWNERS
        .lock()
        .ok()
        .and_then(|owners| {
            owners
                .get(path)
                .map(|entries| entries.iter().filter_map(Weak::upgrade).collect())
        })
        .unwrap_or_default();
    let mut roots: Vec<_> = captures
        .iter()
        .map(|capture| capture.root.clone())
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

/// Inputs are copied once when a host begins watching a new project revision.
pub(super) fn inputs(
    root: &Path,
    revision: u64,
    known_revision: Option<u64>,
) -> Option<Vec<PathBuf>> {
    let capture = CURRENT.lock().ok()?.get(root)?.upgrade()?;
    if capture.revision != revision {
        return None;
    }
    if known_revision == Some(revision) {
        return Some(Vec::new());
    }
    // Initial watch admission validates once for the shared project, closing
    // edits between compilation and registering an external config watch.
    if capture.snapshot.observations().needs_confirmation() {
        return None;
    }
    Some(capture.inputs.clone())
}
