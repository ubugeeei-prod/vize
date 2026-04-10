//! Corsa-backed batch executor.
//!
//! This module materializes the virtual project, asks the Corsa project-session
//! API for diagnostics across every generated file, and maps those diagnostics
//! back to the original source positions.

use std::path::{Path, PathBuf};

use super::error::{CorsaError, CorsaNotFoundError, CorsaResult};
use super::type_checker::TypeCheckResult;
use super::virtual_project::VirtualProject;
use crate::{
    corsa_client::CorsaProjectClient,
    lsp_client::paths::{corsa_search_roots, find_corsa_in_search_roots},
};
use vize_carton::{cstr, String};

mod diagnostics;

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
        let mut client = CorsaProjectClient::new_for_workspace(
            Some(corsa_path.as_ref()),
            project.virtual_root(),
        )
        .map_err(map_corsa_error)?;
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

fn map_corsa_error(message: String) -> CorsaError {
    CorsaError::CorsaExecution {
        exit_code: -1,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_virtual_file_uris, normalize_corsa_path};
    use std::fs;
    use tempfile::TempDir;
    use vize_carton::cstr;

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
}
