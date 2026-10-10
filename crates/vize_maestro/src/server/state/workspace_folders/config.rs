//! Reuse one host evaluation for editor flags and per-file linter plans.
use super::{
    LintPlanScope, LinterConfigPlanWithConfigRuleOptions, LinterFeatureFlags, Path, PathBuf,
    ProjectIgnoreSet, ServerState, WorkspaceFolderConfig,
};
use vize_carton::config::LoadedProjectConfig;

impl WorkspaceFolderConfig {
    #[cfg(feature = "native")]
    pub(super) fn deferred(root: PathBuf) -> Self {
        Self {
            root,
            plan: Default::default(),
            scopes: Vec::new(),
            global_ignores: Vec::new(),
            project_ignores: None,
            features: Default::default(),
        }
    }

    pub(super) fn load(root: PathBuf) -> Self {
        let loaded = vize_carton::config::load_project_config_with_source(Some(&root));
        Self::from_project(root, &loaded)
    }

    fn from_project(root: PathBuf, loaded: &LoadedProjectConfig) -> Self {
        let document = &loaded.document;
        let compatibility = document
            .compiler_compatibility_vue_version()
            .or(document.legacy_compatibility_vue_version());
        let compiler_vapor = document.compiler_vapor();
        let (_, flags) = document.clone().into_config_and_features();
        let features =
            LinterFeatureFlags::from_config_features(flags, compatibility, compiler_vapor);
        let plan = if loaded.source_path.is_some() {
            document.linter_plan_with_config_rule_options()
        } else {
            LinterConfigPlanWithConfigRuleOptions::default()
        };
        let config_dir = loaded
            .source_path
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or(&root);
        let scopes = plan
            .plan
            .entries
            .iter()
            .map(|entry| {
                LintPlanScope::new(
                    entry.base_path.as_deref(),
                    entry.files.as_deref(),
                    &entry.ignores,
                    config_dir,
                    &root,
                )
            })
            .collect();
        let project_ignores = loaded
            .project_root
            .as_ref()
            .and_then(|_| ProjectIgnoreSet::new(&plan.plan.global_ignores, config_dir));
        let global_ignores = plan
            .plan
            .global_ignores
            .iter()
            .filter(|_| loaded.project_root.is_none())
            .map(|entry| {
                LintPlanScope::new(
                    entry.base_path.as_deref(),
                    None,
                    std::slice::from_ref(&entry.pattern),
                    config_dir,
                    &root,
                )
            })
            .collect();
        Self {
            root,
            plan,
            scopes,
            global_ignores,
            project_ignores,
            features,
        }
    }
}

impl ServerState {
    pub(in crate::server::state) fn install_project_linter_context(
        &self,
        root: &Path,
        loaded: &LoadedProjectConfig,
    ) {
        let context = WorkspaceFolderConfig::from_project(root.to_path_buf(), loaded);
        let mut contexts = self.workspace_folder_configs.write();
        contexts.retain(|context| context.root != root);
        contexts.push(context);
    }
}
