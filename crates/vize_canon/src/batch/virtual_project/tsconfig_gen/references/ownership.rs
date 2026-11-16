use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

pub use vize_l0::typescript_ownership::TsconfigSourceKind;
pub(super) use vize_l0::typescript_ownership::unique_owner;

use super::{graph, spec::SpecCache};

/// Host path semantics used while matching tsconfig ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TsconfigOwnershipOptions {
    case_sensitive: bool,
}

impl TsconfigOwnershipOptions {
    pub(super) const fn case_sensitive(self) -> bool {
        self.case_sensitive
    }

    /// Build an authority for a host whose file names have the given case
    /// semantics. This is public for remote/virtual hosts and deterministic
    /// cross-platform regression tests.
    pub const fn with_case_sensitive(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }
}

impl Default for TsconfigOwnershipOptions {
    fn default() -> Self {
        Self {
            case_sensitive: vize_l0::typescript_ownership::host_uses_case_sensitive_file_names(),
        }
    }
}

/// Run-scoped parsed ownership graph. There is no process-global state: HMR
/// and CLI runs can replace this cache when configuration inputs change.
#[derive(Default)]
pub struct TsconfigOwnershipCache {
    options: TsconfigOwnershipOptions,
    specs: SpecCache,
    project_paths: FxHashMap<PathBuf, Vec<PathBuf>>,
}

impl TsconfigOwnershipCache {
    pub fn with_options(options: TsconfigOwnershipOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    /// The root project followed by every transitive reference, in declaration
    /// order and with cycles/deduplicated diamonds visited once.
    pub fn project_paths(&mut self, tsconfig_path: &Path) -> Vec<PathBuf> {
        let resolved = self.specs.normalize(tsconfig_path);
        self.project_paths
            .entry(resolved)
            .or_insert_with_key(|root| graph::collect_project_paths(root))
            .clone()
    }

    /// Whether a single effective project includes `source_path`.
    pub fn project_owns_source(
        &mut self,
        tsconfig_path: &Path,
        source_path: &Path,
        source_kind: TsconfigSourceKind,
    ) -> bool {
        self.project_owns_normalized_source(
            tsconfig_path,
            &graph::normalize_path(source_path),
            source_kind,
        )
    }

    fn project_owns_normalized_source(
        &mut self,
        tsconfig_path: &Path,
        source_path: &Path,
        source_kind: TsconfigSourceKind,
    ) -> bool {
        let case_sensitive = self.options.case_sensitive;
        let Some(spec) = self.specs.load(tsconfig_path) else {
            return false;
        };
        spec.includes_normalized(source_path, case_sensitive, source_kind)
    }

    /// The inherited `compilerOptions.allowJs` for one effective project.
    pub fn project_allows_js(&mut self, tsconfig_path: &Path) -> bool {
        self.specs
            .load(tsconfig_path)
            .and_then(|spec| spec.allow_js())
            .unwrap_or(false)
    }

    /// Resolve one source through the complete project-reference graph.
    /// Unowned and multiply-owned inputs fail closed to the root project.
    pub fn effective_config_for_source(
        &mut self,
        tsconfig_path: &Path,
        source_path: &Path,
        source_kind: TsconfigSourceKind,
    ) -> PathBuf {
        let source_path = graph::normalize_path(source_path);
        self.effective_config_for_normalized_source(tsconfig_path, &source_path, source_kind)
    }

    /// [`Self::effective_config_for_source`] for a source the caller has
    /// already canonicalized, as `canonicalize` would spell it.
    pub fn effective_config_for_normalized_source(
        &mut self,
        tsconfig_path: &Path,
        source_path: &Path,
        source_kind: TsconfigSourceKind,
    ) -> PathBuf {
        let projects = self.project_paths(tsconfig_path);
        let root = projects
            .first()
            .cloned()
            .unwrap_or_else(|| self.specs.normalize(tsconfig_path));
        unique_owner(&root, &projects, |project| {
            self.project_owns_normalized_source(project, source_path, source_kind)
        })
    }
}
