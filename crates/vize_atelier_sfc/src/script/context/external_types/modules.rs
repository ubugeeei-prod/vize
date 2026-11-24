//! Imported type modules shared by the components of one stability window.
//!
//! Every component of a batch that imports a shared type walks the same
//! modules. A graph keeps what those walks have in common: each module is read
//! and digested once, each import edge is resolved once, and the walk from one
//! module for one set of requested exports is computed once. A component then
//! resolves only its own imports and merges the modules of the finished walks.
//!
//! A graph serves one batch generation and one resolution context: the
//! selected project and the revision of its options. Nothing in it depends on
//! the importing component. A compile outside a batch walks a graph of its
//! own, so it observes the files as they are at that moment.
//!
//! A graph keeps what it observed, not what went wrong: a module that could
//! not be read is read again by the next walk that reaches it, and a walk
//! that met such a module is not kept.

#![expect(
    clippy::disallowed_types,
    reason = "one graph is shared by the parallel workers of a batch"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock, PoisonError};

use vize_carton::{FxHashMap, String};

use super::super::batch_epoch::NO_EPOCH;
use super::attempt::Attempted;
use super::closure::TypeClosure;
use super::resolution::{path_key, project::Project, resolve_import_path_with_project};
use super::walk::{RequestedExports, TypeDependency};
use super::work::{Step, count};
use super::{FileTypeSummary, summary_cache};

/// A module's position in its graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ModuleId(u32);

type ClosureCell = Arc<Attempted<Arc<TypeClosure>>>;
type EntryCell = Arc<OnceLock<Option<ModuleId>>>;

/// Module ids the kept walks of one graph may hold: 32 MB. Each walk holds
/// two ids per module it reached, so many different requests into a wide
/// barrel add up. Past this the graph still serves every read and every
/// resolution from memory; only the walks over them are repeated.
const KEPT_WALK_IDS: usize = (32 << 20) / size_of::<ModuleId>();

pub(super) struct Module {
    path: PathBuf,
    /// The name of this module in dependency lists.
    pub(super) key: String,
    facts: Attempted<ModuleFacts>,
    /// Finished walks from this module, by requested exports.
    closures: Mutex<FxHashMap<RequestedExports, ClosureCell>>,
}

impl Module {
    /// Read this module once: later walks of its graph reuse the summary.
    pub(super) fn facts(&self) -> Option<&ModuleFacts> {
        let read = || {
            let summary = summary_cache::load(&self.path).ok_or(())?;
            let targets = summary
                .dependencies
                .iter()
                .map(|_| OnceLock::new())
                .collect();
            Ok::<_, ()>(ModuleFacts { summary, targets })
        };
        self.facts.get_or_attempt(read).ok()
    }
}

/// What one read of a module's file established.
pub(super) struct ModuleFacts {
    pub(super) summary: Arc<FileTypeSummary>,
    /// The module each dependency of `summary` resolves to, found on first use.
    pub(super) targets: Box<[OnceLock<Option<ModuleId>>]>,
}

/// Slots of the first chunk; each later chunk doubles the one before it.
const FIRST_CHUNK: usize = 16;
const CHUNKS: usize = 32;

/// Modules by id. A slot never moves once filled, so a walk reads modules
/// without a lock while other workers add more.
#[derive(Default)]
struct ModuleTable {
    chunks: [OnceLock<Box<[OnceLock<Module>]>>; CHUNKS],
}

impl ModuleTable {
    fn slot(&self, id: ModuleId, fill: bool) -> Option<&OnceLock<Module>> {
        // Chunk `c` holds `FIRST_CHUNK << c` slots and starts after the
        // `FIRST_CHUNK * (2^c - 1)` slots of the chunks before it.
        let index = usize::try_from(id.0).ok()?;
        let chunk = (index / FIRST_CHUNK + 1).checked_ilog2()?;
        let start = FIRST_CHUNK * ((1_usize << chunk) - 1);
        let cell = self.chunks.get(usize::try_from(chunk).ok()?)?;
        let slots = if fill {
            cell.get_or_init(|| (0..FIRST_CHUNK << chunk).map(|_| OnceLock::new()).collect())
        } else {
            cell.get()?
        };
        slots.get(index - start)
    }

    fn get(&self, id: ModuleId) -> Option<&Module> {
        self.slot(id, false)?.get()
    }

    fn insert(&self, id: ModuleId, module: Module) -> Option<()> {
        self.slot(id, true)?.set(module).ok()
    }
}

#[derive(Default)]
pub(super) struct ModuleGraph {
    /// The batch generation whose files this graph observed.
    epoch: u64,
    ids: Mutex<FxHashMap<PathBuf, ModuleId>>,
    table: ModuleTable,
    /// The module a component's own import resolves to, by importing
    /// directory and then specifier.
    entries: Mutex<FxHashMap<PathBuf, FxHashMap<String, EntryCell>>>,
    /// Module ids the kept walks hold, and how many they may hold.
    kept_ids: AtomicUsize,
    kept_ids_limit: usize,
}

impl ModuleGraph {
    pub(super) fn new(epoch: u64) -> Self {
        Self::keeping(epoch, KEPT_WALK_IDS)
    }

    /// A graph whose kept walks hold at most `kept_ids_limit` module ids.
    pub(super) fn keeping(epoch: u64, kept_ids_limit: usize) -> Self {
        Self {
            epoch,
            kept_ids_limit,
            ..Self::default()
        }
    }

    pub(super) fn module(&self, id: ModuleId) -> Option<&Module> {
        self.table.get(id)
    }

    /// The summary a walk of this graph already read for `id`.
    pub(super) fn summary(&self, id: ModuleId) -> Option<&FileTypeSummary> {
        Some(&self.module(id)?.facts.get()?.summary)
    }

    fn intern(&self, path: PathBuf) -> Option<ModuleId> {
        count(Step::SharedLock, 1);
        let mut ids = self.ids.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(id) = ids.get(&path) {
            return Some(*id);
        }
        let id = ModuleId(u32::try_from(ids.len()).ok()?);
        self.table.insert(
            id,
            Module {
                key: path_key(&path),
                path: path.clone(),
                facts: Attempted::default(),
                closures: Mutex::default(),
            },
        )?;
        ids.insert(path, id);
        Some(id)
    }

    /// Resolve `specifier` as written in `importer`.
    pub(super) fn resolve(
        &self,
        importer: &Path,
        specifier: &str,
        project: Option<&Project>,
    ) -> Option<ModuleId> {
        self.intern(resolve_import_path_with_project(
            importer, specifier, project, self.epoch,
        )?)
    }

    /// The module a component in `importer`'s directory imports as
    /// `specifier`, resolved once for every component of that directory.
    pub(super) fn entry(
        &self,
        importer: &Path,
        specifier: &str,
        project: Option<&Project>,
    ) -> Option<ModuleId> {
        let directory = importer.parent()?;
        let cell = {
            count(Step::SharedLock, 1);
            let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
            if !entries.contains_key(directory) {
                entries.insert(directory.to_path_buf(), FxHashMap::default());
            }
            let specifiers = entries.get_mut(directory)?;
            match specifiers.get(specifier) {
                Some(cell) => Arc::clone(cell),
                None => Arc::clone(specifiers.entry(specifier.into()).or_default()),
            }
        };
        *cell.get_or_init(|| self.resolve(importer, specifier, project))
    }

    /// The module `dependency` of `module` resolves to, probed once.
    pub(super) fn target(
        &self,
        module: &Module,
        dependency: &TypeDependency,
        target: &OnceLock<Option<ModuleId>>,
        project: Option<&Project>,
    ) -> Option<ModuleId> {
        *target.get_or_init(|| self.resolve(&module.path, dependency.specifier(), project))
    }

    /// The walk from `module` for `requested`, computed by the first worker
    /// that asks for it and kept for the others. A walk waits on no other
    /// walk, so import cycles cannot make two workers wait on each other.
    ///
    /// A walk that met a module it could not read is used by its caller and
    /// not kept, and neither is a walk the graph has no room for.
    pub(super) fn closure(
        &self,
        module: ModuleId,
        requested: &RequestedExports,
        project: Option<&Project>,
    ) -> Option<Arc<TypeClosure>> {
        let key = requested.canonical();
        let cell = {
            count(Step::SharedLock, 1);
            let mut closures = self
                .module(module)?
                .closures
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            match closures.get(&key) {
                Some(cell) => Arc::clone(cell),
                None => Arc::clone(closures.entry(key).or_default()),
            }
        };
        let walk = || {
            let walked = Arc::new(TypeClosure::walk(self, module, requested, project));
            if walked.complete && self.keeps(&walked) {
                Ok(walked)
            } else {
                Err(walked)
            }
        };
        Some(match cell.get_or_attempt(walk) {
            Ok(kept) => Arc::clone(kept),
            Err(unkept) => unkept,
        })
    }

    /// Whether the graph has room to keep `closure`, which it then counts.
    fn keeps(&self, closure: &TypeClosure) -> bool {
        let ids = closure.modules.len() + closure.files.len();
        let kept = self.kept_ids.fetch_add(ids, Ordering::Relaxed);
        if kept.saturating_add(ids) <= self.kept_ids_limit {
            return true;
        }
        self.kept_ids.fetch_sub(ids, Ordering::Relaxed);
        false
    }
}

/// The graph of one resolution context in one batch generation.
struct Window {
    epoch: u64,
    revision: Option<[u8; 32]>,
    graph: Arc<ModuleGraph>,
}

#[derive(Default)]
struct Windows {
    /// Resolution without a project.
    unconfigured: Option<Window>,
    /// Resolution through a project, by its config path.
    projects: FxHashMap<PathBuf, Option<Window>>,
}

static WINDOWS: LazyLock<Mutex<Windows>> = LazyLock::new(Mutex::default);

/// The graph every worker of batch generation `epoch` shares for `project`,
/// or `None` when this compile must walk a graph of its own: outside a batch,
/// or when the slot already belongs to a newer generation or to another
/// revision of the same project.
pub(super) fn shared(project: Option<&Project>, epoch: u64) -> Option<Arc<ModuleGraph>> {
    if epoch == NO_EPOCH {
        return None;
    }
    let revision = project.map(|project| project.revision);
    count(Step::SharedLock, 1);
    let mut windows = WINDOWS.lock().unwrap_or_else(PoisonError::into_inner);
    let slot = match project {
        None => &mut windows.unconfigured,
        Some(project) => {
            if !windows.projects.contains_key(&project.config) {
                windows.projects.insert(project.config.clone(), None);
            }
            windows.projects.get_mut(&project.config)?
        }
    };
    if let Some(window) = slot.as_ref() {
        if window.epoch == epoch && window.revision == revision {
            return Some(Arc::clone(&window.graph));
        }
        if window.epoch >= epoch {
            return None;
        }
    }
    let graph = Arc::new(ModuleGraph::new(epoch));
    let retired = slot.replace(Window {
        epoch,
        revision,
        graph: Arc::clone(&graph),
    });
    // Free the previous generation's modules after releasing the lock.
    drop(windows);
    drop(retired);
    Some(graph)
}

/// Drop every shared graph once no batch is open: the files they observed are
/// no longer held stable.
pub(in crate::script::context) fn retire() {
    let retired = std::mem::take(&mut *WINDOWS.lock().unwrap_or_else(PoisonError::into_inner));
    drop(retired);
}
