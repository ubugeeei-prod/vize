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
use super::virtual_project::VirtualProject;
use crate::{
    corsa_client::CorsaProjectClient,
    lsp_client::paths::{corsa_search_roots, find_corsa_in_search_roots},
};
use oxc_span::SourceType;
use vize_carton::{cstr, String};

mod cli;
mod diagnostics;

use cli::check_with_cli;
use diagnostics::map_batch_diagnostics;

/// Batch executor backed by `corsa`'s project-session diagnostics API.
pub struct CorsaExecutor {
    /// Path to the resolved Corsa executable.
    corsa_path: PathBuf,
}

impl CorsaExecutor {
    /// Create a new executor by finding a local or global Corsa executable.
    pub fn new(project_root: &Path) -> Result<Self, CorsaNotFoundError> {
        let search_roots = corsa_search_roots(Some(project_root));
        if let Some(local_corsa) = find_corsa_in_search_roots(&search_roots) {
            if let Some(corsa_path) = normalize_corsa_path(PathBuf::from(local_corsa.as_str())) {
                return Ok(Self { corsa_path });
            }
        }

        for executable in ["corsa", "tsgo"] {
            if let Ok(global_corsa) = which::which(executable) {
                if let Some(corsa_path) = normalize_corsa_path(global_corsa) {
                    return Ok(Self { corsa_path });
                }
            }
        }

        Err(CorsaNotFoundError::new(project_root))
    }

    /// Get the resolved executable path.
    pub fn corsa_path(&self) -> &Path {
        &self.corsa_path
    }

    /// Run type checking on the virtual project.
    pub fn check(&self, project: &VirtualProject) -> CorsaResult<TypeCheckResult> {
        project.materialize()?;

        let corsa_path = self.corsa_path.to_string_lossy();
        let mut client = match CorsaProjectClient::new_for_workspace(
            Some(corsa_path.as_ref()),
            project.virtual_root(),
        ) {
            Ok(client) => client,
            Err(error) if should_fallback_to_cli(&error) => {
                return check_with_cli(&self.corsa_path, project);
            }
            Err(error) => return Err(map_corsa_error(error)),
        };
        let uris = collect_virtual_file_uris(project.virtual_root())?;
        let diagnostics = map_batch_diagnostics(
            client
                .request_diagnostics_batch(&uris)
                .map_err(map_corsa_error)?,
            project,
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
        project.materialize()?;
        let config_path = project
            .write_declaration_tsconfig(options.out_dir.as_path(), options.declaration_map)?;
        let output = Command::new(&self.corsa_path)
            .current_dir(project.virtual_root())
            .arg("--pretty")
            .arg("false")
            .arg("--project")
            .arg(&config_path)
            .output()?;

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

        rewrite_declaration_outputs(options.out_dir.as_path())?;

        Ok(DeclarationEmitResult {
            files: collect_declaration_outputs(options.out_dir.as_path())?,
        })
    }
}

fn normalize_corsa_path(path: PathBuf) -> Option<PathBuf> {
    let Some(bin_dir) = path.parent() else {
        return Some(path);
    };
    if bin_dir.file_name().and_then(|name| name.to_str()) != Some(".bin") {
        return Some(path);
    }

    let Some(node_modules_dir) = bin_dir.parent() else {
        return Some(path);
    };
    if node_modules_dir.file_name().and_then(|name| name.to_str()) != Some("node_modules") {
        return Some(path);
    }

    let Some(project_root) = node_modules_dir.parent() else {
        return Some(path);
    };
    find_corsa_in_search_roots(&corsa_search_roots(Some(project_root)))
        .map(|resolved| PathBuf::from(resolved.as_str()))
        .filter(|resolved| resolved != &path)
}

fn collect_virtual_file_uris(virtual_root: &Path) -> CorsaResult<Vec<String>> {
    let mut uris = Vec::new();

    for entry in walkdir::WalkDir::new(virtual_root) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some("ts" | "tsx") = path.extension().and_then(|extension| extension.to_str()) {
            uris.push(cstr!("file://{}", path.display()));
        }
    }

    uris.sort();
    Ok(uris)
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
        let rewritten = rewriter
            .rewrite_declaration_specifiers(&content, SourceType::ts())
            .code;
        if rewritten.as_str() != content {
            std::fs::write(path, rewritten.as_str())?;
        }
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
    use super::{collect_declaration_outputs, collect_virtual_file_uris, normalize_corsa_path};
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
            .join("__agent_only")
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
        fs::write(root.join("tsconfig.json"), "{}").unwrap();
        fs::write(root.join("ignored.js"), "").unwrap();

        let uris = collect_virtual_file_uris(root).unwrap();

        assert_eq!(
            uris,
            vec![
                cstr!("file://{}", root.join("component.vue.ts").display()),
                cstr!("file://{}", root.join("index.ts").display()),
            ]
        );
    }

    #[test]
    fn normalizes_node_modules_bin_wrapper_to_native_preview_binary() {
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

        assert_eq!(normalize_corsa_path(wrapper), Some(native));
    }

    #[test]
    fn falls_back_to_project_cache_when_wrapper_lacks_native_binary() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let wrapper = root.join("node_modules/.bin/tsgo");
        let cache = root.join(".cache").join("tsgo");

        fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
        fs::create_dir_all(cache.parent().unwrap()).unwrap();
        fs::write(&wrapper, "").unwrap();
        fs::write(&cache, "").unwrap();

        assert_eq!(normalize_corsa_path(wrapper), Some(cache));
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
    fn checks_with_cli_when_project_session_api_is_unavailable() {
        use super::CorsaExecutor;
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
