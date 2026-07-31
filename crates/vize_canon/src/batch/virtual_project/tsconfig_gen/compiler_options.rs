//! Loading and flattening effective `compilerOptions` from a tsconfig chain.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashSet, profile};

use crate::batch::error::CorsaResult;

use super::super::VirtualProject;
use super::super::tsconfig_paths::{
    normalize_path_lexically, parse_jsonc_value, resolve_extended_tsconfig_path,
};
use super::path_rebase;

#[derive(Clone, Copy)]
enum PathOptions {
    Rebase,
    Verbatim,
}

impl VirtualProject {
    #[allow(clippy::disallowed_types)]
    pub(in super::super) fn load_compiler_options(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        self.load_compiler_options_with_paths(tsconfig_path, PathOptions::Rebase)
    }

    /// Flatten the effective options without changing path spellings. The
    /// input-less option probe validates syntax only, so its paths stay inert.
    #[allow(clippy::disallowed_types)]
    pub(in super::super) fn load_compiler_options_verbatim(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        self.load_compiler_options_with_paths(tsconfig_path, PathOptions::Verbatim)
    }

    #[allow(clippy::disallowed_types)]
    fn load_compiler_options_with_paths(
        &self,
        tsconfig_path: Option<&Path>,
        path_options: PathOptions,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(Map::new());
        };

        let mut seen = FxHashSet::default();
        self.load_compiler_options_inner(tsconfig_path, &mut seen, path_options)
    }

    #[allow(clippy::disallowed_types)]
    fn load_compiler_options_inner(
        &self,
        tsconfig_path: &Path,
        seen: &mut FxHashSet<PathBuf>,
        path_options: PathOptions,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        if !tsconfig_path.exists() {
            return Ok(Map::new());
        }
        let normalized = normalize_path_lexically(tsconfig_path);
        if !seen.insert(normalized.clone()) {
            return Ok(Map::new());
        }

        let content = profile!("canon.tsconfig.read", std::fs::read_to_string(&normalized))?;
        let config = profile!("canon.tsconfig.parse", parse_jsonc_value(&content))?;
        let mut compiler_options = config
            .get("compilerOptions")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if matches!(path_options, PathOptions::Rebase) {
            let base_dir = normalized.parent().unwrap_or(self.project_root.as_path());
            path_rebase::onto_project_root(&mut compiler_options, base_dir, &self.project_root);
        }

        // `extends` may be a single specifier or an array; array entries are
        // applied in order, with later entries overriding earlier ones, and
        // the extending file overriding them all.
        let mut inherited = Map::new();
        match config.get("extends") {
            Some(Value::String(extends)) => {
                if let Some(parent_path) = resolve_extended_tsconfig_path(&normalized, extends) {
                    inherited =
                        self.load_compiler_options_inner(&parent_path, seen, path_options)?;
                }
            }
            Some(Value::Array(entries)) => {
                for extends in entries.iter().filter_map(Value::as_str) {
                    if let Some(parent_path) = resolve_extended_tsconfig_path(&normalized, extends)
                    {
                        inherited.extend(self.load_compiler_options_inner(
                            &parent_path,
                            seen,
                            path_options,
                        )?);
                    }
                }
            }
            _ => {}
        }
        if inherited.is_empty() {
            return Ok(compiler_options);
        }

        inherited.extend(compiler_options);
        Ok(inherited)
    }
}
