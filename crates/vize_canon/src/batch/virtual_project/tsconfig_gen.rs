//! Generating the virtual project's `tsconfig.json`: inheriting the user's
//! compiler options through `extends`, stripping path-sensitive options, and
//! re-anchoring path aliases into the virtual mirror.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashSet, String as CompactString, ToCompactString, cstr, profile};

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::write_if_changed;

use super::tsconfig_paths::{
    normalize_path_lexically, normalize_tsconfig_path_target, parse_jsonc_value,
    resolve_extended_tsconfig_path,
};
use super::{AUTO_IMPORT_STUBS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject};

const PATH_SENSITIVE_COMPILER_OPTIONS: &[&str] = &[
    "baseUrl",
    "paths",
    "rootDir",
    "rootDirs",
    "outDir",
    "declarationDir",
    "typeRoots",
    "tsBuildInfoFile",
];

impl VirtualProject {
    pub(super) fn write_tsconfig_file(
        &self,
        path: &Path,
        out_dir: Option<&Path>,
        declaration_map: bool,
    ) -> CorsaResult<()> {
        let tsconfig = self.generate_tsconfig_value(out_dir, declaration_map)?;
        let content = serde_json::to_string_pretty(&tsconfig)?;
        write_if_changed(path, content.as_bytes())?;
        Ok(())
    }

    fn generate_tsconfig_value(
        &self,
        out_dir: Option<&Path>,
        declaration_map: bool,
    ) -> CorsaResult<Value> {
        let mut config = Map::new();
        let original_tsconfig = self.resolved_tsconfig_path();
        if out_dir.is_none()
            && let Some(ref tsconfig_path) = original_tsconfig
        {
            config.insert(
                "extends".into(),
                Value::String(tsconfig_path.to_string_lossy().into_owned()),
            );
        }

        let mut compiler_options = self.load_compiler_options(original_tsconfig.as_deref())?;

        // Capture the original path-alias map before stripping path-sensitive
        // options, so it can be re-anchored into the virtual mirror below.
        let original_paths = compiler_options
            .get("paths")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();

        for option in PATH_SENSITIVE_COMPILER_OPTIONS {
            compiler_options.remove(*option);
        }
        compiler_options.insert("allowImportingTsExtensions".into(), Value::Bool(true));

        // Re-anchor tsconfig `paths` into the virtual mirror. Without this the
        // aliases inherited via `extends` resolve against the real source tree,
        // where `.vue` files only match the ambient `*.vue` stub (default export
        // only) and named re-exports surface as false `TS2614`. Each alias
        // target gets a mirror candidate first (so the generated `.vue.ts`
        // modules win) and the real-tree path as a fallback (so aliases to files
        // outside the checked set keep resolving).
        if !original_paths.is_empty() {
            compiler_options.insert(
                "paths".into(),
                Value::Object(self.remap_paths(&original_paths)),
            );
        }

        if let Some(out_dir) = out_dir {
            compiler_options.insert("noEmit".into(), Value::Bool(false));
            compiler_options.insert("declaration".into(), Value::Bool(true));
            compiler_options.insert("emitDeclarationOnly".into(), Value::Bool(true));
            compiler_options.insert("declarationMap".into(), Value::Bool(declaration_map));
            compiler_options.insert(
                "rootDir".into(),
                Value::String(
                    self.common_virtual_source_dir()
                        .to_string_lossy()
                        .into_owned(),
                ),
            );
            compiler_options.insert(
                "outDir".into(),
                Value::String(out_dir.to_string_lossy().into_owned()),
            );
        } else {
            compiler_options.remove("declaration");
            compiler_options.remove("emitDeclarationOnly");
            compiler_options.remove("declarationMap");
            compiler_options.remove("outDir");
            compiler_options.insert("noEmit".into(), Value::Bool(true));
        }

        config.insert("compilerOptions".into(), Value::Object(compiler_options));
        config.insert(
            "include".into(),
            Value::Array(
                self.include_paths()
                    .into_iter()
                    .map(|path| Value::String(path.into()))
                    .collect(),
            ),
        );
        config.insert("exclude".into(), Value::Array(Vec::new()));

        Ok(Value::Object(config))
    }

    fn include_paths(&self) -> Vec<CompactString> {
        let mut includes: Vec<_> = self
            .virtual_files
            .keys()
            .filter_map(|path| path.strip_prefix(&self.virtual_root).ok())
            .map(|path| path.to_string_lossy().to_compact_string())
            .collect();
        if !self.virtual_ts_options.auto_import_stubs.is_empty() {
            includes.push(AUTO_IMPORT_STUBS_FILE.into());
        }
        includes.push(VUE_MODULE_STUBS_FILE.into());
        includes.sort();
        includes
    }
    #[allow(clippy::disallowed_types)]
    fn load_compiler_options(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(Map::new());
        };

        let mut seen = FxHashSet::default();
        self.load_compiler_options_inner(tsconfig_path, &mut seen)
    }

    #[allow(clippy::disallowed_types)]
    fn load_compiler_options_inner(
        &self,
        tsconfig_path: &Path,
        seen: &mut FxHashSet<PathBuf>,
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
        let base_dir = normalized.parent().unwrap_or(self.project_root.as_path());
        self.normalize_paths_for_project_root(&mut compiler_options, base_dir);

        let Some(parent_path) = config
            .get("extends")
            .and_then(Value::as_str)
            .and_then(|extends| resolve_extended_tsconfig_path(&normalized, extends))
        else {
            return Ok(compiler_options);
        };

        let mut inherited = self.load_compiler_options_inner(&parent_path, seen)?;
        inherited.extend(compiler_options);
        Ok(inherited)
    }

    #[allow(clippy::disallowed_types)]
    fn normalize_paths_for_project_root(
        &self,
        compiler_options: &mut Map<std::string::String, Value>,
        base_dir: &Path,
    ) {
        let Some(paths) = compiler_options
            .get_mut("paths")
            .and_then(Value::as_object_mut)
        else {
            return;
        };

        for targets in paths.values_mut() {
            let Some(targets) = targets.as_array_mut() else {
                continue;
            };
            for target in targets {
                let Some(raw_target) = target.as_str() else {
                    continue;
                };
                if Path::new(raw_target).is_absolute() {
                    continue;
                }
                *target = Value::String(
                    normalize_tsconfig_path_target(base_dir, &self.project_root, raw_target).into(),
                );
            }
        }
    }

    /// Re-anchor tsconfig `paths` targets into the virtual mirror. Each relative
    /// target yields two candidates: the mirror copy (resolved relative to the
    /// virtual tsconfig, which lives in the mirror root) followed by the real
    /// source-tree path as a fallback. Absolute and non-string targets pass
    /// through unchanged.
    #[allow(clippy::disallowed_types)]
    fn remap_paths(
        &self,
        paths: &Map<std::string::String, Value>,
    ) -> Map<std::string::String, Value> {
        let up = self.virtual_root_to_project_prefix();
        let mut remapped = Map::new();
        for (alias, targets) in paths {
            let Some(targets) = targets.as_array() else {
                remapped.insert(alias.clone(), targets.clone());
                continue;
            };
            let mut candidates = Vec::with_capacity(targets.len() * 2);
            for target in targets {
                let Some(target) = target.as_str() else {
                    candidates.push(target.clone());
                    continue;
                };
                if Path::new(target).is_absolute() {
                    candidates.push(Value::String(target.to_owned()));
                    continue;
                }
                let core = target.strip_prefix("./").unwrap_or(target);
                candidates.push(Value::String(cstr!("./{core}").into()));
                candidates.push(Value::String(cstr!("{up}{core}").into()));
            }
            remapped.insert(alias.clone(), Value::Array(candidates));
        }
        remapped
    }

    /// Relative prefix (e.g. `../../../`) from the virtual root back to the
    /// project root, used to aim alias fallbacks at the real source tree.
    fn virtual_root_to_project_prefix(&self) -> CompactString {
        let depth = self
            .virtual_root
            .strip_prefix(&self.project_root)
            .map(|relative| relative.components().count())
            .unwrap_or(0);
        let mut prefix = CompactString::with_capacity(depth * 3);
        for _ in 0..depth {
            prefix.push_str("../");
        }
        prefix
    }
}
