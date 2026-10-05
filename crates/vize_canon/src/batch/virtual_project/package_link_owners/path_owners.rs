//! Distinct source, shadow and alias roles can own one physical link claim.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{PackageLinkPatch, VirtualProject};
use crate::batch::error::CorsaResult;

pub(super) fn retain_scope_path(files: &mut FxHashMap<PathBuf, usize>, path: &Path) -> bool {
    let count = files.entry(path.to_path_buf()).or_default();
    *count += 1;
    *count == 1
}

pub(super) fn release_scope_path(files: &mut FxHashMap<PathBuf, usize>, path: &Path) -> bool {
    let remove = files.get_mut(path).is_some_and(|count| {
        *count = count.saturating_sub(1);
        *count == 0
    });
    if remove {
        files.remove(path);
    }
    files.is_empty()
}

impl VirtualProject {
    pub(in super::super) fn validate_incremental_workspace_links(
        &self,
        full: Option<&FxHashMap<PathBuf, PathBuf>>,
        patch: Option<&PackageLinkPatch>,
    ) -> CorsaResult<Vec<PathBuf>> {
        // Current target authority is separate from the actual effective plan.
        // Observations here never retire cache links or consume pending paths.
        self.validate_workspace_alias_targets(&self.retired_package_shadow_paths)?;
        let mut links = full
            .cloned()
            .unwrap_or_else(|| self.materialized_package_links.clone());
        if full.is_none()
            && let Some(patch) = patch
        {
            for path in &patch.candidates {
                links.remove(path);
            }
            links.extend(
                patch
                    .desired
                    .iter()
                    .map(|(path, target)| (path.clone(), target.clone())),
            );
        }
        self.validate_workspace_alias_links(&links, &self.retired_package_shadow_paths)
    }
}
