//! Final owned install leaves are distinct from uncertain cache ancestors.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::VirtualProject;
use crate::batch::error::CorsaResult;

impl VirtualProject {
    pub(in super::super) fn workspace_cold_owned_files(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> FxHashSet<PathBuf> {
        let aliases = self.workspace_alias_links();
        let mut owned = expected_files.clone();
        // Cold GC preserves current alias destinations without visiting their
        // retired descendants. Expected/union files are never excluded.
        owned.extend(
            self.retired_package_shadow_paths
                .iter()
                .filter(|path| {
                    !aliases
                        .iter()
                        .any(|alias| path.starts_with(&alias.virtual_dir))
                })
                .cloned(),
        );
        owned
    }

    pub(in super::super) fn workspace_owned_parent_dirs(
        &self,
        extra_owned_files: &FxHashSet<PathBuf>,
    ) -> FxHashSet<PathBuf> {
        // Include retired artifacts and editor-union files before any cleanup,
        // independently of whether their last route owner still exists.
        self.package_shadow_files
            .keys()
            .chain(self.package_shadow_manifests.keys())
            .chain(extra_owned_files)
            .flat_map(|path| path.parent().into_iter().flat_map(Path::ancestors))
            .chain(
                self.materialized_package_links
                    .iter()
                    .filter(|(_, target)| target.starts_with(&self.virtual_root))
                    .flat_map(|(alias, _)| alias.parent().into_iter().flat_map(Path::ancestors)),
            )
            .filter(|path| path.starts_with(&self.virtual_root))
            .map(Path::to_path_buf)
            .collect()
    }

    pub(in super::super) fn workspace_install_roots(&self) -> CorsaResult<FxHashSet<PathBuf>> {
        let mut roots = FxHashMap::default();
        for topology in self.package_shadow_artifacts.values() {
            for (root, manifest) in &topology.install_roots {
                // These roots are derived from an actual selected in-owner
                // installation, and this owner independently pins its manifest.
                // No vanished or canonicalized disk endpoint is authority.
                if root == &self.virtual_root
                    || !root.starts_with(&self.virtual_root)
                    || topology.manifests.get(&root.join("package.json")) != Some(manifest)
                    || self
                        .package_shadow_manifests
                        .get(&root.join("package.json"))
                        != Some(manifest)
                    || roots
                        .insert(root.clone(), manifest)
                        .is_some_and(|old| old != manifest)
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Workspace installation has no unambiguous owned manifest",
                    )
                    .into());
                }
            }
        }
        Ok(roots.into_keys().collect())
    }
}
