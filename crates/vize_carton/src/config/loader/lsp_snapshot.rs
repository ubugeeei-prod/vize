//! One evaluation of the workspace config for an editor session.

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::config::{
    ConfigFeatureFlags, ConfigLintRuleOptions, LanguageServerUnstableFlags, LinterConfig,
    VizeConfig,
};

use super::{
    BeforeConfigRead, ConfigLoadFailureIdentity, LoadedProjectConfig, LoadedRawConfig,
    checked::load_raw_editor_config_attempt_with_compile_cache, project_config_snapshot,
};

/// Optional compiled-code storage owned by one editor server, never evaluated JS state.
/// Each load still uses a fresh Node process and all original input observations.
#[derive(Default)]
pub struct ConfigEvaluationSession {
    compile_cache: OnceLock<Option<tempfile::TempDir>>,
}

fn create_compile_cache() -> Option<tempfile::TempDir> {
    let mut builder = tempfile::Builder::new();
    builder.prefix("vize-config-compile-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Set privacy at creation, before a child can write compiled code.
        builder.permissions(std::fs::Permissions::from_mode(0o700));
    }
    builder.tempdir().ok()
}

impl ConfigEvaluationSession {
    /// Observe one fresh evaluation, sharing only Node's compiled code with this session.
    pub fn load_snapshot_observed(
        &self,
        path: Option<&Path>,
        before_read: &mut dyn FnMut(&[PathBuf]),
    ) -> LoadedLspConfig {
        // Preserve user cache/coverage policy. A setup failure is only a cache miss.
        let externally_configured = [
            "NODE_COMPILE_CACHE",
            "NODE_DISABLE_COMPILE_CACHE",
            "NODE_COMPILE_CACHE_PORTABLE",
            "NODE_V8_COVERAGE",
        ]
        .iter()
        .any(|name| std::env::var_os(name).is_some())
            || std::env::var_os("NODE_OPTIONS")
                .is_some_and(|options| options.to_string_lossy().contains("coverage"));
        let directory = (!externally_configured)
            .then(|| {
                self.compile_cache
                    .get_or_init(create_compile_cache)
                    .as_ref()
                    .map(|directory| directory.path())
            })
            .flatten();
        load_snapshot(path, Some(before_read), directory)
    }
}

/// All LSP config values derived from one raw config evaluation.
#[derive(Debug, Clone)]
pub struct LoadedLspConfig {
    /// Configuration was absent or successfully parsed in this one evaluation.
    pub valid: bool,
    /// Evaluation failure retained for an editor-visible reload error.
    pub error: Option<std::string::String>,
    /// Host-only failed evaluation identity. Never configuration or watch authority;
    /// absent metadata preserves strict equality of the original error message.
    pub failure_identity: Option<ConfigLoadFailureIdentity>,
    /// Last attempt's actual inputs and missing candidates, for repair watching
    /// only. These paths never make a failed configuration authoritative.
    pub watch_dependencies: Vec<PathBuf>,
    /// False when the loader cannot expose all failed resolution candidates.
    /// Unchanged observations alone must not suppress the next repair attempt.
    pub watch_dependencies_complete: bool,
    /// The same raw evaluation retains ordered lint scopes and selected root.
    pub project: LoadedProjectConfig,
    pub config: VizeConfig,
    pub source_path: Option<PathBuf>,
    pub features: ConfigFeatureFlags,
    pub linter: LinterConfig,
    pub lint_rule_options: ConfigLintRuleOptions,
    pub language_server_unstable_flags: LanguageServerUnstableFlags,
    pub request_timeout_ms: u64,
    /// Authored compiler whitespace from this same checked evaluation.
    pub compiler_whitespace: Option<&'static str>,
}

/// Evaluate JS/TS/PKL only once so the editor timeout and type checker
/// settings always come from the same config snapshot.
pub fn load_lsp_config_snapshot(path: Option<&Path>) -> LoadedLspConfig {
    load_snapshot(path, None, None)
}

/// Observe actual discovery and loader inputs before their selection or load.
///
/// Runs synchronously on the caller's configuration worker. Newly discovered
/// imported helpers remain post-load dependencies; this is not an atomic file
/// snapshot or a replacement for the selected engine's resolution.
pub fn load_lsp_config_snapshot_observed(
    path: Option<&Path>,
    before_read: &mut dyn FnMut(&[PathBuf]),
) -> LoadedLspConfig {
    load_snapshot(path, Some(before_read), None)
}

fn load_snapshot(
    path: Option<&Path>,
    before_read: BeforeConfigRead<'_>,
    compile_cache: Option<&Path>,
) -> LoadedLspConfig {
    let attempt =
        load_raw_editor_config_attempt_with_compile_cache(path, before_read, compile_cache);
    let (loaded, error) = match attempt.outcome {
        Ok(loaded) => (loaded, None),
        Err(error) => {
            eprintln!("Warning: Failed to load editor project configuration: {error}");
            (
                LoadedRawConfig {
                    config: Default::default(),
                    source_path: None,
                },
                Some(error),
            )
        }
    };
    let mut snapshot = LoadedLspConfig::from_project(project_config_snapshot(loaded));
    snapshot.valid = error.is_none();
    snapshot.error = error;
    snapshot.failure_identity = attempt.failure_identity;
    snapshot.watch_dependencies = attempt.dependencies;
    snapshot.watch_dependencies_complete = attempt.dependencies_complete;
    snapshot
}

impl LoadedLspConfig {
    fn from_project(project: LoadedProjectConfig) -> Self {
        let document = &project.document;
        let compiler_whitespace = document.compiler_whitespace();
        let linter = document.linter();
        let lint_rule_options = document.lint_rule_options().clone();
        let language_server_unstable_flags = document.language_server_unstable_flags();
        let request_timeout_ms = document.lsp_request_timeout_ms();
        let (config, features) = document.clone().into_config_and_features();
        let source_path = project.source_path.clone();
        Self {
            valid: true,
            error: None,
            failure_identity: None,
            watch_dependencies: Vec::new(),
            watch_dependencies_complete: true,
            project,
            config,
            source_path,
            features,
            linter,
            lint_rule_options,
            language_server_unstable_flags,
            request_timeout_ms,
            compiler_whitespace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{load_lsp_config_snapshot, load_lsp_config_snapshot_observed};
    use std::path::{Path, PathBuf};

    fn discovery_paths(root: &Path) -> Vec<PathBuf> {
        [
            "vite.config.ts",
            "vite.config.js",
            "vite.config.mjs",
            "vite.config.cjs",
            "vite.config.mts",
            "vite.config.cts",
        ]
        .map(|name| root.join(name))
        .to_vec()
    }

    #[cfg(unix)]
    #[test]
    fn compiled_code_directory_is_private_and_removed_with_its_session() {
        use super::{ConfigEvaluationSession, create_compile_cache};
        use std::os::unix::fs::PermissionsExt;
        let session = ConfigEvaluationSession::default();
        let directory = session
            .compile_cache
            .get_or_init(create_compile_cache)
            .as_ref()
            .unwrap();
        let path = directory.path().to_path_buf();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o077,
            0
        );
        drop(session);
        assert!(!path.exists());
    }

    #[test]
    fn observes_explicit_policy_before_reading_without_observing_directory_mtimes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("serialized-config.json");
        std::fs::write(&path, r#"{"vue":{"formatter":{"printWidth":91}}}"#).unwrap();
        let mut observed = Vec::new();
        let snapshot = load_lsp_config_snapshot_observed(Some(&path), &mut |paths| {
            observed.extend_from_slice(paths);
            std::fs::write(&path, r#"{"vue":{"formatter":{"printWidth":93}}}"#).unwrap();
        });
        assert!(snapshot.valid);
        assert_eq!(snapshot.config.formatter.print_width, 93);
        assert!(observed.iter().all(|input| input == &path));

        observed.clear();
        let absent = load_lsp_config_snapshot_observed(Some(root.path()), &mut |paths| {
            observed.extend_from_slice(paths);
        });
        assert!(absent.valid);
        let mut expected = discovery_paths(root.path());
        expected.extend(
            ["pkl", "ts", "mts", "cts", "js", "mjs", "cjs", "json"]
                .map(|extension| root.path().join(format!("vize.config.{extension}"))),
        );
        assert_eq!(observed, expected);
    }

    #[test]
    fn observed_config_exit_before_finally_preserves_authored_diagnostic() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("vite.config.mjs");
        std::fs::write(
            &path,
            "console.error('authored-specificreason'); process.exit(17);",
        )
        .unwrap();
        let snapshot = load_lsp_config_snapshot_observed(Some(root.path()), &mut |_| {});
        assert!(!snapshot.valid);
        let error = snapshot.error.unwrap();
        let status = if cfg!(windows) {
            "exit code: 17"
        } else {
            "exit status: 17"
        };
        assert_eq!(
            error,
            format!(
                "failed to parse {}: node failed to load config: authored-specificreason ({status}; observation transport: failed to fill whole buffer)",
                path.display()
            )
        );
        assert!(!snapshot.watch_dependencies_complete);
    }

    #[test]
    fn missing_explicit_vite_file_remains_a_watch_input_until_created() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("vite.config.mjs");
        let missing = load_lsp_config_snapshot(Some(&path));
        assert!(!missing.valid);
        assert!(missing.watch_dependencies_complete);
        assert_eq!(missing.watch_dependencies, vec![path.clone()]);
        std::fs::write(&path, "export default { vue: {} };").unwrap();
        let created = load_lsp_config_snapshot(Some(&path));
        assert!(created.valid, "{:?}", created.error);
        assert!(created.watch_dependencies_complete);
        let canonical = path.canonicalize().unwrap();
        let mut expected = vec![path.clone(), canonical.clone()];
        expected.extend(
            canonical
                .parent()
                .unwrap()
                .ancestors()
                .flat_map(|directory| {
                    ["vite", "vite-plus"].map(|engine| directory.join("node_modules").join(engine))
                }),
        );
        expected.sort();
        expected.dedup();
        assert_eq!(created.watch_dependencies, expected);
        assert_eq!(created.source_path, Some(path));
    }

    #[test]
    fn every_encountered_retired_file_remains_observable_until_deletion_repairs_discovery() {
        let root = tempfile::tempdir().unwrap();
        let paths = [
            root.path().join("vize.config.json"),
            root.path().join("vize.config.ts"),
        ];
        for path in &paths {
            std::fs::write(path, "historical bytes are never evaluated").unwrap();
        }
        let rejected = load_lsp_config_snapshot(Some(root.path()));
        assert!(!rejected.valid);
        assert!(rejected.watch_dependencies_complete);
        let mut expected = discovery_paths(root.path());
        expected.extend(paths.iter().cloned());
        expected.sort();
        assert_eq!(rejected.watch_dependencies, expected);
        std::fs::remove_file(&paths[0]).unwrap();
        let retained = load_lsp_config_snapshot(Some(root.path()));
        assert!(!retained.valid);
        let mut expected = discovery_paths(root.path());
        expected.push(paths[1].clone());
        expected.sort();
        assert_eq!(retained.watch_dependencies, expected);
        std::fs::remove_file(&paths[1]).unwrap();
        let repaired = load_lsp_config_snapshot(Some(root.path()));
        assert!(repaired.valid, "{:?}", repaired.error);
        assert!(repaired.source_path.is_none());
        let mut expected = discovery_paths(root.path());
        expected.sort();
        assert_eq!(repaired.watch_dependencies, expected);
    }
}
