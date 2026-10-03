//! Shared effective editor configuration and authored global-declaration scope.

use std::path::{Path, PathBuf};

use crate::batch::{TsconfigOwnershipCache, TsconfigSourceKind};

pub(super) struct EditorProjectConfiguration {
    pub(super) root: PathBuf,
    pub(super) tsconfig: Option<PathBuf>,
    source_path: PathBuf,
}

impl EditorProjectConfiguration {
    pub(super) fn for_source(
        source_path: &Path,
        project_root: Option<&Path>,
        tsconfig_path: Option<&Path>,
    ) -> Self {
        let source_path = vize_carton::path::canonicalize_non_verbatim(source_path);
        let discovered_root = source_path
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("tsconfig.json").is_file())
            .map(Path::to_path_buf);
        // A workspace root governs materialization, while explicit or nearest
        // source configuration governs compiler options and ambient membership.
        let root = project_root
            .map(Path::to_path_buf)
            .or(discovered_root.clone())
            .unwrap_or_else(|| source_path.parent().unwrap_or(&source_path).to_path_buf());
        let root = vize_carton::path::canonicalize_non_verbatim(&root);
        let tsconfig = tsconfig_path
            .map(|path| {
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    root.join(path)
                }
            })
            .or_else(|| discovered_root.map(|dir| dir.join("tsconfig.json")));
        Self {
            root,
            tsconfig,
            source_path,
        }
    }

    pub(super) fn reference_paths(&self, candidates: &[PathBuf]) -> Vec<PathBuf> {
        let mut ownership = TsconfigOwnershipCache::default();
        let effective = self.tsconfig.as_ref().map(|config| {
            ownership.effective_config_for_source(
                config,
                &self.source_path,
                TsconfigSourceKind::Typed,
            )
        });
        let mut selected = candidates
            .iter()
            .filter(|path| super::vue_dependencies_alias::is_declaration(path))
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .filter(|path| {
                path.is_file()
                    && effective.as_ref().map_or_else(
                        // Keep the existing no-tsconfig workspace behavior.
                        // An out-of-root candidate is never a workspace global.
                        || path.starts_with(&self.root),
                        |config| {
                            ownership.project_owns_source(config, path, TsconfigSourceKind::Typed)
                        },
                    )
            })
            .collect::<Vec<_>>();
        selected.sort();
        selected.dedup();
        selected
    }

    /// The diagnostic compatibility route for already parser-classified Vue
    /// component augmentations. Ordinary global values still require exact
    /// membership through `reference_paths`.
    pub(super) fn component_reference_paths(&self, candidates: &[PathBuf]) -> Vec<PathBuf> {
        let mut ownership = TsconfigOwnershipCache::default();
        let effective = self.tsconfig.as_ref().map(|config| {
            ownership.effective_config_for_source(
                config,
                &self.source_path,
                TsconfigSourceKind::Typed,
            )
        });
        if let Some(config) = &self.tsconfig {
            let owners = ownership
                .project_paths(config)
                .iter()
                .filter(|project| {
                    ownership.project_owns_source(
                        project,
                        &self.source_path,
                        TsconfigSourceKind::Typed,
                    )
                })
                .count();
            if owners != 1 {
                return Vec::new();
            }
        }
        let mut selected = candidates
            .iter()
            .filter(|path| super::vue_dependencies_alias::is_declaration(path))
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .filter(|path| {
                if !path.is_file() {
                    return false;
                }
                let Some(config) = effective.as_ref() else {
                    return path.starts_with(&self.root);
                };
                if ownership.project_owns_source(config, path, TsconfigSourceKind::Typed) {
                    return true;
                }
                let Some(root) = config.parent() else {
                    return false;
                };
                if !path.starts_with(root) {
                    return false;
                }
                // A physically nested project is a separate source scope even
                // when its component augmentation is under the workspace root.
                path.ancestors()
                    .skip(1)
                    .find(|directory| directory.join("tsconfig.json").is_file())
                    .is_none_or(|directory| {
                        vize_carton::path::canonicalize_non_verbatim(
                            &directory.join("tsconfig.json"),
                        ) == *config
                    })
            })
            .collect::<Vec<_>>();
        selected.sort();
        selected.dedup();
        selected
    }
}

#[cfg(test)]
#[path = "editor_configuration/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "editor_configuration/reference_tests.rs"]
mod reference_tests;
