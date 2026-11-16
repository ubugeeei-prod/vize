use std::path::{Path, PathBuf};

use crate::batch::error::CorsaResult;

use super::super::{UnresolvedExtends, VirtualProject};

impl VirtualProject {
    pub(in crate::batch::virtual_project) fn resolved_tsconfig_path(&self) -> Option<PathBuf> {
        if let Some(snapshot) = self.compiler_configuration.as_ref() {
            return snapshot.tsconfig_path().map(Path::to_path_buf);
        }
        if let Some(ref tsconfig_path) = self.tsconfig_path {
            return Some(tsconfig_path.clone());
        }
        let tsconfig = self.project_root.join("tsconfig.json");
        tsconfig.exists().then_some(tsconfig)
    }

    /// The configs whose contents govern this project's alias map: the anchored
    /// tsconfig and every solution-style reference (#3923 cache invalidation).
    pub fn governing_config_paths(&self) -> Vec<PathBuf> {
        if let Some(snapshot) = self.compiler_configuration.as_ref() {
            return snapshot.dependencies().to_vec();
        }
        self.governing_config_paths_with_inputs(None)
    }

    pub(in crate::batch::virtual_project) fn governing_config_paths_with_inputs(
        &self,
        inputs: Option<&super::super::tsconfig_gen::compiler_inputs::CompilerInputs>,
    ) -> Vec<PathBuf> {
        let Some(anchored) = self.resolved_tsconfig_path() else {
            return Vec::new();
        };
        let mut config_roots = vec![anchored.clone()];
        config_roots.extend(
            super::super::tsconfig_gen::references::referenced_project_configs_with_inputs(
                &anchored, inputs,
            ),
        );
        let mut paths = config_roots.clone();
        for config in config_roots {
            if let Ok(flattened) =
                self.load_compiler_options_flattened_with_inputs(Some(&config), inputs)
            {
                paths.extend(flattened.input_paths);
            }
            paths.extend(self.vue_target_manifest_path_with_inputs(&config, inputs));
        }
        paths.sort();
        paths.dedup();
        paths
    }

    /// Validate the selected editor owner's `extends` chain through the same
    /// resolver used for compiler options.
    pub fn checked_config_chain_paths(&self) -> CorsaResult<Vec<PathBuf>> {
        let Some(anchored) = self.resolved_tsconfig_path() else {
            return Ok(Vec::new());
        };
        self.checked_compiler_config_paths(&anchored)
    }

    /// `extends` entries of the effective chain that TypeScript reports (TS6053)
    /// and skips. Checking continues with the rest of the chain.
    pub fn unresolved_tsconfig_extends(&self) -> Vec<UnresolvedExtends> {
        let Some(anchored) = self.resolved_tsconfig_path() else {
            return Vec::new();
        };
        self.load_compiler_options_flattened(Some(&anchored))
            .map(|flattened| flattened.unresolved_extends)
            .unwrap_or_default()
    }

    /// File that project-level diagnostics are attributed to.
    pub(crate) fn project_diagnostics_anchor(&self) -> PathBuf {
        self.resolved_tsconfig_path()
            .unwrap_or_else(|| self.project_root.clone())
    }
}
