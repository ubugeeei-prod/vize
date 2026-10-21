//! TypeScript project authority complements the shared Vite snapshot.
use std::path::Path;
#[expect(
    clippy::disallowed_types,
    reason = "immutable snapshots are shared by editor owners"
)]
use std::sync::Arc;

use super::{ServerState, host::LoadedLspConfig};

impl ServerState {
    /// Keep the observed child transport on the existing bounded config workers.
    #[cfg(feature = "native")]
    #[expect(
        clippy::disallowed_types,
        reason = "initial admission retains the shared editor owner on its worker"
    )]
    pub(crate) async fn load_workspace_config_async(
        self: &std::sync::Arc<Self>,
        dir: &Path,
        folders: Option<&[tower_lsp::lsp_types::WorkspaceFolder]>,
    ) -> bool {
        let state = self.clone();
        let directory = dir.to_path_buf();
        let folders = folders.map(<[_]>::to_vec);
        Self::project_background(move || {
            state.load_workspace_config(&directory);
            state.apply_initialize_workspace_folders(folders.as_deref(), Some(&directory));
        })
        .await
        .is_some()
    }

    pub(super) fn apply_tsconfig_config_snapshot(
        &self,
        directory: &Path,
        loaded: &mut LoadedLspConfig,
        before: Option<&vize_canon::ConfigurationObservations>,
        needs_confirmation: bool,
    ) -> bool {
        let key = self.project_contexts.key.read().clone();
        let authored = loaded.clone();
        let mut observations = before.cloned().unwrap_or_default();
        let result = Self::prepare_tsconfig_config_snapshot(
            directory,
            key.as_ref(),
            loaded,
            &mut observations,
        );
        *self.project_contexts.configuration_observation.write() = Some(
            super::super::project_contexts::freshness::ConfigurationObservation::initial(
                &authored,
                loaded,
                observations,
                result.is_ok(),
                before,
                needs_confirmation,
            ),
        );
        match result {
            Ok(compiler) => {
                *self.project_contexts.config_dependencies.write() =
                    loaded.project.dependencies.clone();
                *self.project_contexts.compiler_configuration.write() = Some(compiler);
                true
            }
            Err(error) => {
                tracing::warn!("Failed to resolve project configuration: {error}");
                false
            }
        }
    }

    pub(in crate::server::state) fn prepare_tsconfig_config_snapshot(
        directory: &Path,
        key: Option<&super::super::project_contexts::ProjectContextKey>,
        loaded: &mut LoadedLspConfig,
        observations: &mut vize_canon::ConfigurationObservations,
    ) -> Result<Arc<vize_canon::CompilerConfigurationSnapshot>, vize_l0::String> {
        let root = loaded.project.project_root.as_deref().unwrap_or(directory);
        let base = loaded
            .source_path
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or(directory);
        let inferred = key.is_some_and(|key| key.inferred);
        let has_explicit_tsconfig = key.is_some_and(|key| key.tsconfig.is_some())
            || loaded.config.type_checker.tsconfig.is_some();
        let conventional = [
            root.join("tsconfig.json"),
            root.join("jsconfig.json"),
            base.join("tsconfig.json"),
            base.join("jsconfig.json"),
        ];
        let configured = key
            .and_then(|key| key.tsconfig.clone())
            .or_else(|| {
                loaded
                    .config
                    .type_checker
                    .tsconfig
                    .as_deref()
                    .map(|path| base.join(path))
            })
            .unwrap_or_else(|| {
                conventional
                    .iter()
                    .find(|path| path.is_file())
                    .cloned()
                    .unwrap_or_else(|| root.join("tsconfig.json"))
            });
        if inferred
            || (key.is_none_or(|key| key.is_root())
                && loaded.config.type_checker.tsconfig.is_none())
        {
            let mut observed = conventional.to_vec();
            if !observed.contains(&configured) {
                observed.push(configured.clone());
            }
            observations.extend(&vize_canon::ConfigurationObservations::capture(&observed));
        }
        let selected = (!inferred || has_explicit_tsconfig).then_some(configured);
        if let Some(selected) = selected
            .as_ref()
            .filter(|selected| !selected.is_file() && has_explicit_tsconfig)
        {
            loaded.watch_dependencies.push(selected.clone());
            observations.extend(&vize_canon::ConfigurationObservations::capture(
                std::slice::from_ref(selected),
            ));
            return Err(vize_l0::cstr!(
                "selected TypeScript configuration does not exist: {}",
                selected.display()
            ));
        }
        let mut dependencies = loaded.project.dependencies.clone();
        let compiler;
        if let Some(selected) = selected.as_ref().filter(|selected| selected.is_file()) {
            // Keep the checker on the same TS project when Vite moves the source root.
            loaded.config.type_checker.tsconfig =
                Some(selected.to_string_lossy().into_owned().into());
            let attempt =
                vize_canon::CompilerConfigurationSnapshot::capture_attempt(root, Some(selected));
            loaded.watch_dependencies.extend(attempt.dependencies);
            observations.extend(&attempt.observations);
            let snapshot = attempt.outcome.map_err(|error| vize_l0::cstr!("{error}"))?;
            dependencies.extend(snapshot.dependencies().iter().cloned());
            compiler = Arc::new(snapshot);
            // Authored Vite dialects, including Vue 3, take precedence.
            let source_owner = key.is_some_and(|key| key.tsconfig.is_some());
            let solution_shell = !source_owner && compiler.is_solution_shell();
            if loaded.features.vue_version.is_none() && !solution_shell {
                match compiler.vue_target() {
                    Ok(Some(version)) => {
                        loaded.features.vue_version = Some(version);
                        loaded.features.type_checker_legacy_vue2 |= version.is_legacy();
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(vize_l0::cstr!("{error}"));
                    }
                }
            }
        } else {
            loaded.config.type_checker.tsconfig = None;
            let attempt = if inferred {
                vize_canon::CompilerConfigurationSnapshot::capture_inferred_attempt(root)
            } else {
                vize_canon::CompilerConfigurationSnapshot::capture_attempt(root, None)
            };
            loaded.watch_dependencies.extend(attempt.dependencies);
            observations.extend(&attempt.observations);
            let snapshot = attempt.outcome.map_err(|error| vize_l0::cstr!("{error}"))?;
            dependencies.extend(snapshot.dependencies().iter().cloned());
            compiler = Arc::new(snapshot);
        }
        dependencies.extend(selected);
        dependencies.sort();
        dependencies.dedup();
        loaded.project.dependencies = dependencies;
        if !observations.changed_paths().is_empty() {
            return Err(vize_l0::cstr!(
                "TypeScript configuration changed during admission; keeping the last good configuration"
            ));
        }
        Ok(compiler)
    }
}
