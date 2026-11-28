//! Host-owned project selection shared by the CLI and native SFC adapters.
#![expect(
    clippy::disallowed_types,
    reason = "immutable project snapshots are shared across native and CLI workers"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use vize_atelier_sfc::{SfcCompileResult, SfcDescriptor, SfcError, script::SfcTypeProject};
use vize_canon::TsconfigSourceKind;
use vize_l0::hash::hash_str;
use vize_l0::{FxHashMap, String};

mod captured;
use captured::CapturedProject;

mod host_path;
pub use host_path::{plain_windows_path, plain_windows_path_buf};

mod source_facts;
pub(crate) use source_facts::{Facts, SourceFacts};

type Snapshot = Arc<CapturedProject>;
type SessionCell = Arc<OnceLock<Result<Snapshot, String>>>;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

/// Contextual results include the complete importer and immutable authority.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeProjectCacheIdentity {
    importer_hash: u64,
    importer_len: usize,
    session: u64,
    selection: usize,
    options: [u8; 32],
}

#[cfg(test)]
mod tests;

/// One API request or CLI build owns a session. Each root is validated once
/// in that stability window, and parallel workers share its immutable specs.
/// A new single-file/HMR request validates the observations again.
pub struct ImportedTypeProjects {
    fallback: Option<PathBuf>,
    selected: Mutex<FxHashMap<PathBuf, SessionCell>>,
    source_facts: SourceFacts,
    session: u64,
}

impl Default for ImportedTypeProjects {
    fn default() -> Self {
        Self {
            fallback: None,
            selected: Mutex::default(),
            source_facts: SourceFacts::default(),
            session: NEXT_SESSION.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// A host-selected authority and the configuration inputs that selected it.
#[derive(Default)]
pub struct SelectedTypeProject {
    project: Option<Arc<SfcTypeProject>>,
    capture: Option<Snapshot>,
    importer_sensitive: bool,
    session: u64,
    selection: usize,
}

impl SelectedTypeProject {
    pub fn info(&self) -> Option<(&Path, u64)> {
        self.capture
            .as_ref()
            .map(|capture| (capture.root.as_path(), capture.revision))
    }
    pub fn cache_identity(&self, importer: &Path) -> Option<TypeProjectCacheIdentity> {
        self.importer_sensitive.then(|| {
            let importer = importer.to_string_lossy();
            TypeProjectCacheIdentity {
                importer_hash: hash_str(&importer),
                importer_len: importer.len(),
                session: self.session,
                selection: self.selection,
                options: self
                    .project
                    .as_ref()
                    .map_or([0; 32], |project| project.cache_revision()),
            }
        })
    }
    pub fn as_deref(&self) -> Option<&SfcTypeProject> {
        self.project.as_deref()
    }

    pub fn track(&self, mut result: SfcCompileResult) -> SfcCompileResult {
        if let Some(capture) = self.capture.as_ref() {
            result
                .dependencies
                .push(capture.root.to_string_lossy().as_ref().into());
            result.dependencies.sort();
            result.dependencies.dedup();
        }
        result
    }
}

impl ImportedTypeProjects {
    pub(crate) fn for_facts(
        &self,
        facts: Facts,
        filename: &Path,
    ) -> Result<SelectedTypeProject, SfcError> {
        if facts.needs_project {
            self.for_file(filename).map_err(|message| SfcError {
                message,
                code: Some("V_SFC_TYPE_RESOLUTION".into()),
                loc: None,
            })
        } else {
            Ok(SelectedTypeProject {
                importer_sensitive: facts.importer_sensitive,
                session: self.session,
                ..Default::default()
            })
        }
    }

    pub fn new(fallback: Option<&str>) -> Self {
        Self {
            fallback: fallback.map(PathBuf::from),
            ..Self::default()
        }
    }

    pub fn for_descriptor(
        &self,
        descriptor: &SfcDescriptor<'_>,
        filename: &Path,
    ) -> Result<SelectedTypeProject, SfcError> {
        // JavaScript and normal-script-only modules need neither project IO
        // nor a source-fact cache allocation on the single-file API path.
        if descriptor.script_setup.is_none()
            || ![descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
                .into_iter()
                .flatten()
                .any(|script| matches!(script.lang.as_deref(), Some("ts" | "tsx")))
        {
            return Ok(SelectedTypeProject::default());
        }
        let facts = self
            .source_facts
            .record(&descriptor.source, || Facts::for_descriptor(descriptor));
        self.for_facts(facts, filename).map_err(|mut error| {
            error.loc = descriptor
                .script_setup
                .as_ref()
                .map(|script| script.loc.clone());
            error
        })
    }

    fn for_file(&self, filename: &Path) -> Result<SelectedTypeProject, String> {
        let Some(root) =
            SfcTypeProject::discover_tsconfig(filename)?.or_else(|| self.fallback.clone())
        else {
            return Ok(SelectedTypeProject {
                importer_sensitive: true,
                session: self.session,
                ..Default::default()
            });
        };
        let snapshot = self.snapshot(&root)?;
        let selected = snapshot.select(filename, TsconfigSourceKind::Typed);
        Ok(SelectedTypeProject {
            project: Some(snapshot.project(&selected)?),
            importer_sensitive: true,
            session: self.session,
            selection: Arc::as_ptr(&snapshot) as usize,
            capture: Some(snapshot),
        })
    }

    fn snapshot(&self, root: &Path) -> Result<Snapshot, String> {
        let entry = self
            .selected
            .lock()
            .map_err(|_| String::from("TypeScript project session unavailable"))?
            .entry(root.to_path_buf())
            .or_default()
            .clone();
        entry.get_or_init(|| CapturedProject::get(root)).clone()
    }
}

/// Project roots affected by an observed configuration or absent candidate.
pub fn type_project_owners(path: &Path) -> Vec<PathBuf> {
    captured::owners(&host_path(path))
}

/// Configuration watch inputs for an exact captured revision, copied once by
/// the host instead of being repeated in each compiled SFC's dependencies.
pub fn type_project_inputs(
    root: &Path,
    revision: u64,
    known_revision: Option<u64>,
) -> Option<Vec<PathBuf>> {
    captured::inputs(&host_path(root), revision, known_revision)
}

/// Captures index plain spellings; accept a verbatim one from older hosts.
fn host_path(path: &Path) -> PathBuf {
    vize_l0::typescript_project::normalize_path_lexically(&plain_windows_path_buf(
        path.to_path_buf(),
    ))
}
