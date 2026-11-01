//! Terminal input-root observations, before any optional semantic transform.

use super::{
    CorsaResult, FxHashSet, PathBuf, RuntimeDependencyLink, RuntimeDependencyRoots, VirtualProject,
};

/// These are observations from the two existing materialization producers,
/// not a complete native input set. The terminal witness must still seal all
/// source/config/library and reference closure before admitting any edit.
pub(in crate::batch) struct CliMaterializedInputs {
    pub(in crate::batch) package_links: vize_carton::FxHashMap<PathBuf, PathBuf>,
    pub(in crate::batch) runtime_links: Option<Vec<RuntimeDependencyLink>>,
}

impl VirtualProject {
    /// The caller holds the ordinary MaterializeLock throughout preparation,
    /// witness validation and native checking. Default callers use materialize.
    pub(in crate::batch) fn materialize_cli_inputs(&self) -> CorsaResult<CliMaterializedInputs> {
        let mut roots = RuntimeDependencyRoots::observed();
        let package_links = self.materialize_snapshot(
            &FxHashSet::default(),
            &Default::default(),
            None,
            false,
            Some(&mut roots),
        )?;
        Ok(CliMaterializedInputs {
            package_links,
            runtime_links: roots.finish(),
        })
    }
}
