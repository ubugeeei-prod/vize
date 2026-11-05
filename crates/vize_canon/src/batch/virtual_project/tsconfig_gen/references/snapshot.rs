//! Immutable membership authority for one selected TypeScript project graph.

use std::path::{Path, PathBuf};
#[expect(
    clippy::disallowed_types,
    reason = "cloned project snapshots share captured JSON without copying its values"
)]
use std::sync::Arc;

use vize_carton::FxHashMap;

use super::super::compiler_inputs::{
    CompilerInputs, ConfigurationObservations, FrozenCompilerInputs,
};
use super::{
    TsconfigOwnershipOptions, TsconfigSourceKind, graph,
    ownership::unique_owner,
    spec::{OwnershipSpec, SpecCache},
};
use crate::batch::error::{CorsaError, CorsaResult};

/// Captured ownership specs without a virtual project or mutable worker cache.
/// Hosts revalidate `observations()` before reuse outside their current batch.
#[derive(Clone)]
pub struct TsconfigOwnershipSnapshot {
    root: PathBuf,
    project_paths: Vec<PathBuf>,
    specs: FxHashMap<PathBuf, OwnershipSpec>,
    options: TsconfigOwnershipOptions,
    observations: ConfigurationObservations,
    #[expect(
        clippy::disallowed_types,
        reason = "captured inputs outlive a request and are shared by snapshot clones"
    )]
    inputs: Arc<FrozenCompilerInputs>,
}

impl TsconfigOwnershipSnapshot {
    /// Load strict options lazily from the same bytes used for membership.
    /// Hosts may memoize this result once for each selected project per capture.
    pub fn compiler_options(
        &self,
        selected: &Path,
    ) -> CorsaResult<vize_carton::typescript_project::FlattenedCompilerOptions> {
        let selected = vize_carton::typescript_project::normalize_path_lexically(selected);
        if !self.project_paths.contains(&selected) {
            return Err(CorsaError::Configuration(vize_carton::cstr!(
                "uncaptured tsconfig project {}",
                selected.display()
            )));
        }
        self.inputs.compiler_options(&selected)
    }

    /// Read one already-selected graph through Canon's observed input authority.
    /// Unreadable or malformed membership specs retain the legacy root fallback;
    /// the host must still load its selected compiler configuration strictly.
    pub fn capture(root: &Path) -> CorsaResult<Self> {
        Self::capture_with_options(root, TsconfigOwnershipOptions::default())
    }

    pub fn capture_with_options(
        root: &Path,
        options: TsconfigOwnershipOptions,
    ) -> CorsaResult<Self> {
        if !root.is_absolute() {
            return Err(CorsaError::Configuration(vize_carton::cstr!(
                "project ownership capture requires an absolute selected config path"
            )));
        }
        let inputs = CompilerInputs::default();
        let project_paths = graph::collect_project_paths_with_inputs(root, Some(&inputs));
        let mut loader = SpecCache::default();
        let specs = project_paths
            .iter()
            .filter_map(|path| {
                loader
                    .load_with_inputs(path, Some(&inputs))
                    .map(|spec| (path.clone(), spec))
            })
            .collect();
        let root = project_paths
            .first()
            .cloned()
            .unwrap_or_else(|| root.to_path_buf());
        let (inputs, observations) = inputs.freeze()?;
        #[expect(
            clippy::disallowed_types,
            reason = "captured inputs outlive a request and are shared by snapshot clones"
        )]
        let inputs = Arc::new(inputs);
        Ok(Self {
            root,
            project_paths,
            specs,
            options,
            observations,
            inputs,
        })
    }

    /// Only a unique owner wins. Unowned or ambiguous sources retain the root.
    /// Selection reads no configuration and does not validate capture freshness.
    pub fn select(&self, source: &Path, kind: TsconfigSourceKind) -> PathBuf {
        let source = graph::normalize_path(source);
        unique_owner(&self.root, &self.project_paths, |path| {
            self.specs.get(path).is_some_and(|spec| {
                spec.includes_normalized(&source, self.options.case_sensitive(), kind)
            })
        })
    }

    pub fn project_paths(&self) -> &[PathBuf] {
        &self.project_paths
    }

    /// Revalidate once before a project batch, or before each unbatched request.
    pub fn observations(&self) -> &ConfigurationObservations {
        &self.observations
    }
}
