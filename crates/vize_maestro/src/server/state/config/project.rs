//! TypeScript project authority complements the shared Vite snapshot.
use std::path::Path;

use super::ServerState;

impl ServerState {
    pub(super) fn apply_tsconfig_config_snapshot(
        &self,
        directory: &Path,
        loaded: &mut vize_carton::config::LoadedLspConfig,
    ) -> bool {
        let root = loaded.project.project_root.as_deref().unwrap_or(directory);
        let base = loaded
            .source_path
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or(directory);
        let selected = loaded
            .config
            .type_checker
            .tsconfig
            .as_deref()
            .map(|path| base.join(path))
            .unwrap_or_else(|| {
                [
                    root.join("tsconfig.json"),
                    root.join("jsconfig.json"),
                    base.join("tsconfig.json"),
                    base.join("jsconfig.json"),
                ]
                .into_iter()
                .find(|path| path.is_file())
                .unwrap_or_else(|| root.join("tsconfig.json"))
            });
        let mut dependencies = loaded.project.dependencies.clone();
        if selected.is_file() {
            // Keep the checker on the same TS project when Vite moves the source root.
            loaded.config.type_checker.tsconfig =
                Some(selected.to_string_lossy().into_owned().into());
            match vize_canon::VirtualProject::new(root) {
                Ok(mut project) => {
                    project.set_tsconfig_path(Some(selected.clone()));
                    dependencies.extend(project.governing_config_paths());
                }
                Err(error) => tracing::warn!("Failed to resolve project configuration: {error}"),
            }
            // Authored Vite dialects, including Vue 3, take precedence.
            if loaded.features.vue_version.is_none() {
                match vize_canon::snapshot_tsconfig_vue_target(root, &selected) {
                    Ok(Some(version)) => {
                        loaded.features.vue_version = Some(version);
                        loaded.features.type_checker_legacy_vue2 |= version.is_legacy();
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!("Failed to load vueCompilerOptions: {error}");
                        return false;
                    }
                }
            }
        }
        dependencies.push(selected);
        dependencies.sort();
        dependencies.dedup();
        *self.project_contexts.config_dependencies.write() = dependencies;
        true
    }
}
