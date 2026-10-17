//! A referenced TS project owns its editor state even beside sibling configs.
use super::{
    Arc, ObservedLspConfig, Path, ProjectContextKey, ProjectModel, ServerState, Url,
    paths::physical_path, registry,
};
use vize_canon::batch::TsconfigSourceKind;

impl ServerState {
    pub(super) fn document_reference_context(
        &self,
        uri: &Url,
        state: &Arc<Self>,
    ) -> Option<Arc<registry::ProjectContext>> {
        let snapshot = state.project_contexts.loaded_config.read().clone()?;
        let key = self.document_reference_key(uri, state)?;
        if self.get_virtual_docs(uri).is_some() {
            self.remove_virtual_docs(uri);
        }
        for previous in self.cached_project_states() {
            if previous.project_contexts.key.read().as_ref() != Some(&key)
                && previous.get_virtual_docs(uri).is_some()
            {
                previous.remove_virtual_docs(uri);
            }
        }
        let owner_key = state.project_contexts.key.read().clone();
        let observation = self
            .configuration_records()
            .into_iter()
            .find(|(key, _)| Some(key) == owner_key.as_ref())
            .map(|(_, record)| record)
            .or_else(|| {
                state
                    .project_contexts
                    .configuration_observation
                    .read()
                    .clone()
            });
        let context = {
            let mut contexts = self.project_contexts.contexts.lock();
            if state.project_context_retired() {
                return None;
            }
            let context = contexts
                .entry(key.clone())
                .or_insert_with(|| Arc::new(registry::ProjectContext::new(key.clone())))
                .clone();
            context.provide_snapshot(
                ObservedLspConfig {
                    loaded: snapshot,
                    before: observation
                        .as_ref()
                        .map(|record| record.before_observations())
                        .unwrap_or_default(),
                    needs_confirmation: observation
                        .as_ref()
                        .is_none_or(|record| record.needs_confirmation()),
                },
                state
                    .project_contexts
                    .config_reload_failed
                    .load(super::Ordering::Acquire),
            );
            context
        };
        self.project_contexts.routes.insert(uri.clone(), key);
        Some(context)
    }

    /// Select the standard reference owner without changing routes or buffers.
    pub(super) fn document_reference_key(
        &self,
        uri: &Url,
        state: &Arc<Self>,
    ) -> Option<ProjectContextKey> {
        if state
            .project_contexts
            .key
            .read()
            .as_ref()
            .is_some_and(|key| key.tsconfig.is_some())
        {
            return None;
        }
        let source = physical_path(&uri.to_file_path().ok()?);
        let root = state
            .project_contexts
            .boundary
            .read()
            .clone()
            .or_else(|| state.get_workspace_root())?;
        let (config, _, origin) = state.native_checker_settings();
        let project = ProjectModel::new(Some(&root), origin.as_deref(), &config);
        let shell = project.tsconfig().map(physical_path).or_else(|| {
            [root.join("tsconfig.json"), root.join("jsconfig.json")]
                .into_iter()
                .find(|path| path.is_file())
        })?;
        let kind = match source.extension().and_then(|extension| extension.to_str()) {
            Some("js" | "jsx" | "mjs" | "cjs") => TsconfigSourceKind::JavaScript,
            _ => TsconfigSourceKind::Typed,
        };
        let selected = {
            let mut ownership = self.project_contexts.reference_ownership.lock();
            if ownership.project_paths(&shell).len() < 2 {
                return None;
            }
            ownership.effective_config_for_source(&shell, &source, kind)
        };
        Some(ProjectContextKey {
            root,
            tsconfig: Some(physical_path(&selected)),
        })
    }

    pub(super) fn document_belongs_to_context(
        &self,
        uri: &Url,
        source: &Path,
        key: &ProjectContextKey,
    ) -> bool {
        self.project_contexts.routes.get(uri).map_or_else(
            || {
                key.tsconfig.is_none()
                    && self.document_context_root(source).as_deref() == Some(key.root.as_path())
            },
            |route| route.value() == key,
        )
    }
}
