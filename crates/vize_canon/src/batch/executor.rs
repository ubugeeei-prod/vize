//! Corsa-backed batch executor.
//!
//! This module materializes the virtual project, asks the Corsa project-session
//! API for diagnostics across every generated file, and maps those diagnostics
//! back to the original source positions.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::error::{CorsaError, CorsaNotFoundError, CorsaResult};
use super::import_rewriter::ImportRewriter;
use super::type_checker::{
    DeclarationEmitOptions, DeclarationEmitResult, DeclarationOutput, TypeCheckResult,
};
use super::virtual_project::{
    AUTO_IMPORT_STUBS_FILE, SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject,
};
use crate::{corsa_client::CorsaProjectClient, file_uri::path_to_file_uri};
use oxc_span::SourceType;
use vize_carton::{
    String,
    corsa_resolver::{CorsaResolveError, CorsaResolveRequest, resolve_corsa_executable},
    cstr, profile,
};

/// Helper-type declarations shipped alongside emitted declaration outputs.
/// Shares the mirror helpers file name so the artifact is recognizable, but
/// carries only the type aliases (no global augmentation, no macro values).
const DECLARATION_HELPERS_FILE: &str = crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME;

mod cli;
mod diagnostics;

use cli::{auto_server_count, check_with_cli, check_with_cli_sharded};
use diagnostics::map_batch_diagnostics;

/// Batch executor backed by `corsa`'s project-session diagnostics API.
pub struct CorsaExecutor {
    /// Path to the resolved Corsa executable.
    corsa_path: PathBuf,
}

impl CorsaExecutor {
    /// Create a new executor by finding a local or global Corsa executable.
    pub fn new(project_root: &Path) -> Result<Self, CorsaNotFoundError> {
        Self::with_corsa_path(project_root, None)
    }

    /// Create a new executor with an optional explicit Corsa executable path.
    pub fn with_corsa_path(
        project_root: &Path,
        corsa_path: Option<&Path>,
    ) -> Result<Self, CorsaNotFoundError> {
        let request = CorsaResolveRequest {
            explicit_path: corsa_path,
            project_root: Some(project_root),
        };

        match resolve_corsa_executable(request) {
            Ok(corsa_path) => Ok(Self { corsa_path }),
            Err(CorsaResolveError::ExplicitNotFound { path, .. }) => {
                Err(CorsaNotFoundError::new_explicit(project_root, &path))
            }
            Err(CorsaResolveError::NotFound) => Err(CorsaNotFoundError::new(project_root)),
        }
    }

    /// Get the resolved executable path.
    pub fn corsa_path(&self) -> &Path {
        &self.corsa_path
    }

    /// Run type checking on the virtual project with an auto-tuned number of
    /// parallel Corsa CLI processes.
    pub fn check(&self, project: &VirtualProject) -> CorsaResult<TypeCheckResult> {
        self.check_with_servers(project, None)
    }

    /// Run type checking on the virtual project. `servers` is the number of
    /// parallel Corsa CLI processes the project is partitioned across
    /// (`None` auto-tunes from the machine width and project size).
    pub fn check_with_servers(
        &self,
        project: &VirtualProject,
        servers: Option<usize>,
    ) -> CorsaResult<TypeCheckResult> {
        profile!("canon.executor.materialize", project.materialize())?;

        let servers = servers.unwrap_or_else(|| auto_server_count(project));
        if servers > 1 {
            match profile!(
                "canon.corsa.cli",
                check_with_cli_sharded(&self.corsa_path, project, servers)
            ) {
                Ok(result) => return Ok(result),
                Err(_cli_error) => {
                    // Degrade to a single CLI program before the session API:
                    // a shard-specific failure must not cost CLI support.
                }
            }
        }

        match profile!("canon.corsa.cli", check_with_cli(&self.corsa_path, project)) {
            Ok(result) => return Ok(result),
            Err(_cli_error) => {
                // Fall through to the project-session API. This keeps the batch
                // runner usable with runtimes whose CLI diagnostics are not
                // available or not parseable.
            }
        }

        self.check_with_project_session(project)
    }

    fn check_with_project_session(&self, project: &VirtualProject) -> CorsaResult<TypeCheckResult> {
        let corsa_path = self.corsa_path.to_string_lossy();
        let mut client = match profile!(
            "canon.corsa.session",
            CorsaProjectClient::new_for_workspace(
                Some(corsa_path.as_ref()),
                project.virtual_root()
            )
        ) {
            Ok(client) => client,
            Err(error) if should_fallback_to_cli(&error) => {
                return profile!(
                    "canon.corsa.cli_fallback",
                    check_with_cli(&self.corsa_path, project)
                );
            }
            Err(error) => return Err(map_corsa_error(error)),
        };
        let uris = profile!(
            "canon.corsa.collect_uris",
            collect_virtual_file_uris(project.virtual_root())
        )?;
        let raw_diagnostics = profile!(
            "canon.corsa.diagnostics",
            client
                .request_diagnostics_batch(&uris)
                .map_err(map_corsa_error)
        )?;
        let diagnostics = profile!(
            "canon.corsa.map_diagnostics",
            map_batch_diagnostics(raw_diagnostics, project)
        );
        let success = diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != 1);

        Ok(TypeCheckResult {
            exit_code: if success { 0 } else { 1 },
            success,
            diagnostics,
        })
    }

    /// Emit declaration files from the materialized virtual project.
    pub fn emit_declarations(
        &self,
        project: &VirtualProject,
        options: &DeclarationEmitOptions,
    ) -> CorsaResult<DeclarationEmitResult> {
        profile!("canon.executor.materialize_dts", project.materialize())?;
        let config_path = profile!(
            "canon.project.dts_tsconfig",
            project.write_declaration_tsconfig(options.out_dir.as_path(), options.declaration_map)
        )?;
        let output = profile!(
            "canon.corsa.emit_dts",
            Command::new(&self.corsa_path)
                .current_dir(project.virtual_root())
                .arg("--pretty")
                .arg("false")
                .arg("--project")
                .arg(&config_path)
                .output()
        )?;

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            #[allow(clippy::disallowed_types)]
            let stderr = std::string::String::from_utf8_lossy(&output.stderr);
            #[allow(clippy::disallowed_types)]
            let stdout = std::string::String::from_utf8_lossy(&output.stdout);
            let message = if stderr.trim().is_empty() {
                stdout.trim().to_owned().into()
            } else if stdout.trim().is_empty() {
                stderr.trim().to_owned().into()
            } else {
                cstr!("{}\n{}", stderr.trim(), stdout.trim())
            };
            return Err(CorsaError::CorsaExecution { exit_code, message });
        }

        profile!(
            "canon.dts.rewrite_outputs",
            rewrite_declaration_outputs(options.out_dir.as_path())
        )?;

        Ok(DeclarationEmitResult {
            files: profile!(
                "canon.dts.collect_outputs",
                collect_declaration_outputs(options.out_dir.as_path())
            )?,
        })
    }
}

fn collect_virtual_file_uris(virtual_root: &Path) -> CorsaResult<Vec<String>> {
    let mut uris = Vec::new();

    for entry in walkdir::WalkDir::new(virtual_root) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if is_internal_virtual_project_file(virtual_root, path) {
            continue;
        }
        if let Some("ts" | "tsx") = path.extension().and_then(|extension| extension.to_str()) {
            uris.push(path_to_file_uri(path));
        }
    }

    uris.sort();
    Ok(uris)
}

fn is_internal_virtual_project_file(virtual_root: &Path, path: &Path) -> bool {
    is_internal_virtual_project_stub(path) || is_under_virtual_node_modules(virtual_root, path)
}

fn is_internal_virtual_project_stub(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                AUTO_IMPORT_STUBS_FILE | VUE_MODULE_STUBS_FILE | SHARED_HELPERS_FILE
            )
        })
}

fn is_under_virtual_node_modules(virtual_root: &Path, path: &Path) -> bool {
    path.strip_prefix(virtual_root)
        .ok()
        .and_then(|path| path.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .is_some_and(|name| name == "node_modules")
}

fn collect_declaration_outputs(out_dir: &Path) -> CorsaResult<Vec<DeclarationOutput>> {
    let mut files = Vec::new();
    let rewriter = ImportRewriter::new();
    if !out_dir.exists() {
        return Ok(files);
    }

    for entry in walkdir::WalkDir::new(out_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".d.ts"))
        {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        files.push(DeclarationOutput {
            path: path.to_path_buf(),
            content: rewriter
                .rewrite_declaration_specifiers(&content, SourceType::ts())
                .code,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn rewrite_declaration_outputs(out_dir: &Path) -> CorsaResult<()> {
    let rewriter = ImportRewriter::new();
    if !out_dir.exists() {
        return Ok(());
    }

    let mut wrote_vue_declaration = false;
    for entry in walkdir::WalkDir::new(out_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".d.ts") {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        let mut rewritten = rewriter
            .rewrite_declaration_specifiers(&content, SourceType::ts())
            .code;
        // Generated `.vue.d.ts` outputs reference the hoisted helper type
        // aliases (`__EmitFn`, `__RuntimePropShape`, ...). Wire each one to
        // the helpers declaration shipped alongside the outputs so consumer
        // programs resolve them without including the virtual mirror.
        if name.ends_with(".vue.d.ts") {
            wrote_vue_declaration = true;
            let depth = path
                .strip_prefix(out_dir)
                .ok()
                .and_then(|relative| relative.parent())
                .map(|parent| parent.components().count())
                .unwrap_or(0);
            let mut reference = String::from("/// <reference path=\"");
            for _ in 0..depth {
                reference.push_str("../");
            }
            reference.push_str(DECLARATION_HELPERS_FILE);
            reference.push_str("\" />\n");
            reference.push_str(&rewritten);
            rewritten = reference.as_str().into();
        }
        if rewritten.as_str() != content {
            std::fs::write(path, rewritten.as_str())?;
        }
    }

    if wrote_vue_declaration {
        std::fs::write(
            out_dir.join(DECLARATION_HELPERS_FILE),
            crate::virtual_ts::DECLARATION_HELPERS_DTS,
        )?;
    }

    Ok(())
}

fn map_corsa_error(message: String) -> CorsaError {
    CorsaError::CorsaExecution {
        exit_code: -1,
        message,
    }
}

fn should_fallback_to_cli(error: &str) -> bool {
    error.contains("expected tuple marker")
        || error.contains("expected uint8 marker")
        || error.contains("expected bin marker")
        || error.contains("process is closed: jsonrpc reader")
        || error.contains("Broken pipe")
        || error.contains("broken pipe")
}

#[cfg(test)]
mod tests {
    use super::{CorsaExecutor, collect_declaration_outputs, collect_virtual_file_uris};
    use crate::file_uri::path_to_file_uri;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use vize_carton::cstr;

    use tempfile::TempDir;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

        let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(&*cstr!(
                "corsa-executor-{name}-{}-{case_id}",
                std::process::id()
            ))
    }

    #[test]
    fn collects_virtual_type_script_files_only() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("index.ts"), "").unwrap();
        fs::write(root.join("component.vue.ts"), "").unwrap();
        fs::write(root.join("__vize_vue_modules.d.ts"), "").unwrap();
        fs::write(root.join("__vize_auto_imports.d.ts"), "").unwrap();
        fs::create_dir_all(root.join("node_modules/vue")).unwrap();
        fs::write(root.join("node_modules/vue/index.d.ts"), "").unwrap();
        fs::create_dir_all(root.join("node_modules/vite")).unwrap();
        fs::write(root.join("node_modules/vite/client.d.ts"), "").unwrap();
        fs::write(root.join("tsconfig.json"), "{}").unwrap();
        fs::write(root.join("ignored.js"), "").unwrap();

        let uris = collect_virtual_file_uris(root).unwrap();

        assert_eq!(
            uris,
            vec![
                path_to_file_uri(root.join("component.vue.ts").as_path()),
                path_to_file_uri(root.join("index.ts").as_path()),
            ]
        );
    }

    #[test]
    fn encodes_reserved_characters_in_virtual_file_uris() {
        let root = unique_case_dir("reserved-uri");
        let _ = fs::remove_dir_all(&root);
        let route_dir = root.join("pages").join("[[org]]").join("[packageName]");
        fs::create_dir_all(&route_dir).unwrap();
        fs::write(route_dir.join("[versionRange].vue.ts"), "").unwrap();

        let uris = collect_virtual_file_uris(root.as_path()).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            uris,
            vec![path_to_file_uri(
                route_dir.join("[versionRange].vue.ts").as_path()
            )]
        );
        assert!(uris[0].contains("%5B%5Borg%5D%5D"));
        assert!(uris[0].contains("%5BpackageName%5D"));
        assert!(uris[0].contains("%5BversionRange%5D.vue.ts"));
    }

    #[test]
    fn normalizes_explicit_node_modules_bin_wrapper_to_native_preview_binary() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let wrapper = root.join("node_modules/.bin/tsgo");
        let native = root
            .join("node_modules")
            .join("@typescript")
            .join("native-preview")
            .join("lib")
            .join("tsgo");

        fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
        fs::create_dir_all(native.parent().unwrap()).unwrap();
        fs::write(&wrapper, "").unwrap();
        fs::write(&native, "").unwrap();

        let executor = CorsaExecutor::with_corsa_path(root, Some(&wrapper)).unwrap();

        assert_eq!(executor.corsa_path(), native.canonicalize().unwrap());
    }

    #[test]
    fn uses_explicit_corsa_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("project");
        let explicit = temp_dir.path().join("bin").join("tsgo");

        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(explicit.parent().unwrap()).unwrap();
        fs::write(&explicit, "").unwrap();

        let executor = CorsaExecutor::with_corsa_path(&project_root, Some(&explicit)).unwrap();

        assert_eq!(executor.corsa_path(), explicit.canonicalize().unwrap());
    }

    #[test]
    fn resolves_relative_explicit_corsa_path_against_project_root() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("project");
        let explicit = project_root.join("bin").join("tsgo");

        fs::create_dir_all(explicit.parent().unwrap()).unwrap();
        fs::write(&explicit, "").unwrap();

        let executor = CorsaExecutor::with_corsa_path(
            &project_root,
            Some(PathBuf::from("bin/tsgo").as_path()),
        )
        .unwrap();

        assert_eq!(executor.corsa_path(), explicit.canonicalize().unwrap());
    }

    #[test]
    fn collects_emitted_declaration_outputs() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("dist/types");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(out_dir.join("App.vue.d.ts"), "export {};\n").unwrap();
        fs::write(out_dir.join("skip.js"), "").unwrap();

        let files = collect_declaration_outputs(&out_dir).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, out_dir.join("App.vue.d.ts"));
        assert_eq!(files[0].content, "export {};\n");
    }

    #[cfg(unix)]
    #[test]
    fn cli_global_diagnostics_do_not_trigger_session_fallback() {
        use crate::batch::VirtualProject;
        use std::os::unix::fs::PermissionsExt;

        let case_dir = unique_case_dir("global-diagnostics");
        let _ = fs::remove_dir_all(&case_dir);
        let cache_dir = case_dir.join(".cache");
        let source = case_dir.join("src").join("main.ts");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "const value: number = 1;\n").unwrap();

        // A runtime whose project check exits non-zero with only file-less
        // config diagnostics (e.g. TS2688) ran fine; treating that as a CLI
        // failure would fall back to the far slower project-session API
        // (`--api` here would hang the test forever). The project-level
        // diagnostic surfaces attributed to the project's tsconfig anchor.
        let tsgo = cache_dir.join("tsgo");
        fs::write(
            &tsgo,
            "#!/bin/sh\nif [ \"$1\" = \"--api\" ]; then exec sleep 600; fi\necho \"error TS2688: Cannot find type definition file for 'vite/client'.\"\nexit 2\n",
        )
        .unwrap();
        fs::set_permissions(&tsgo, fs::Permissions::from_mode(0o755)).unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&source).unwrap();
        let executor = CorsaExecutor::new(&case_dir).unwrap();
        let result = executor.check(&project).unwrap();

        assert!(!result.success);
        assert_eq!(result.exit_code, 2);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, Some(2688));
        assert_eq!(result.diagnostics[0].severity, 1);
        assert_eq!(result.diagnostics[0].file, case_dir);

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[cfg(unix)]
    #[test]
    fn checks_with_cli_when_project_session_api_is_unavailable() {
        use crate::batch::VirtualProject;
        use std::os::unix::fs::PermissionsExt;

        let case_dir = unique_case_dir("cli-fallback");
        let _ = fs::remove_dir_all(&case_dir);
        let cache_dir = case_dir.join(".cache");
        let source = case_dir.join("src").join("main.ts");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "const value: number = 1;\n").unwrap();

        let tsgo = cache_dir.join("tsgo");
        fs::write(
            &tsgo,
            "#!/bin/sh\nif [ \"$1\" = \"--api\" ]; then printf 'api unavailable'; exit 0; fi\nexit 0\n",
        )
        .unwrap();
        fs::set_permissions(&tsgo, fs::Permissions::from_mode(0o755)).unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&source).unwrap();
        let executor = CorsaExecutor::new(&case_dir).unwrap();
        let result = executor.check(&project).unwrap();

        assert!(result.success);
        assert!(result.diagnostics.is_empty());

        let _ = fs::remove_dir_all(&case_dir);
    }
}
