mod native_options;

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashSet, String as CompactString, ToCompactString, cstr, profile};

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::write_if_changed;

use super::tsconfig_paths::{
    normalize_path_lexically, normalize_tsconfig_path_target, parse_jsonc_value,
    resolve_extended_tsconfig_path,
};
use super::{AUTO_IMPORT_STUBS_FILE, SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject};
use native_options::normalize_native_removed_options;

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
    pub(crate) fn tsconfig_preserves_unused_diagnostics(&self) -> bool {
        self.preserve_unused_diagnostics
    }

    /// Alias prefixes declared in the effective tsconfig `paths` map, with
    /// wildcard suffixes stripped: `@/*` → `@/`, `@scope/*` → `@scope/`,
    /// `#imports` → `#imports`. Used as a cost model for shard planning:
    /// files importing through the same project alias are coupled. Aliases
    /// whose every target lives under `node_modules` (e.g. a pinned `vue`
    /// mapping) are dependency cost every program pays anyway and are skipped.
    pub(crate) fn path_alias_prefixes(&self) -> Vec<CompactString> {
        let Ok(compiler_options) =
            self.load_compiler_options(self.resolved_tsconfig_path().as_deref())
        else {
            return Vec::new();
        };
        let Some(paths) = compiler_options.get("paths").and_then(Value::as_object) else {
            return Vec::new();
        };
        paths
            .iter()
            .filter(|(_, targets)| {
                !targets.as_array().is_some_and(|targets| {
                    !targets.is_empty()
                        && targets.iter().all(|target| {
                            target
                                .as_str()
                                .is_some_and(|target| target.contains("node_modules"))
                        })
                })
            })
            .map(|(alias, _)| alias.trim_end_matches('*').to_compact_string())
            .collect()
    }

    pub(super) fn resolve_tsconfig_preserves_unused_diagnostics(&self) -> bool {
        let Some(tsconfig_path) = self.resolved_tsconfig_path() else {
            return false;
        };
        let Ok(compiler_options) = self.load_compiler_options(Some(tsconfig_path.as_path())) else {
            return false;
        };

        compiler_option_enabled(&compiler_options, "noUnusedLocals")
            || compiler_option_enabled(&compiler_options, "noUnusedParameters")
    }

    pub(super) fn write_tsconfig_file(
        &self,
        path: &Path,
        out_dir: Option<&Path>,
        declaration_map: bool,
    ) -> CorsaResult<()> {
        self.write_tsconfig_file_with_includes(path, out_dir, declaration_map, None)
    }

    /// Write a tsconfig whose `include` lists only the given virtual paths
    /// (plus the shared stub files). Used for shard configs that partition the
    /// project across parallel Corsa CLI runs.
    pub(crate) fn write_shard_tsconfig(
        &self,
        shard_index: usize,
        include_virtual_paths: &[&Path],
    ) -> CorsaResult<PathBuf> {
        let config_path = self
            .virtual_root
            .join(cstr!("tsconfig.shard{shard_index}.json").as_str());
        self.write_tsconfig_file_with_includes(
            &config_path,
            None,
            false,
            Some(include_virtual_paths),
        )?;
        Ok(config_path)
    }

    fn write_tsconfig_file_with_includes(
        &self,
        path: &Path,
        out_dir: Option<&Path>,
        declaration_map: bool,
        include_virtual_paths: Option<&[&Path]>,
    ) -> CorsaResult<()> {
        let tsconfig =
            self.generate_tsconfig_value(out_dir, declaration_map, include_virtual_paths)?;
        let content = serde_json::to_string_pretty(&tsconfig)?;
        write_if_changed(path, content.as_bytes())?;
        Ok(())
    }

    fn generate_tsconfig_value(
        &self,
        out_dir: Option<&Path>,
        declaration_map: bool,
        include_virtual_paths: Option<&[&Path]>,
    ) -> CorsaResult<Value> {
        let mut config = Map::new();
        let original_tsconfig = self.resolved_tsconfig_path();

        // The effective compiler options are flattened into the generated
        // config instead of `extends`-ing the user's tsconfig. Corsa would
        // otherwise re-parse the whole original chain and fail the entire CLI
        // run on config-file diagnostics vize already compensates for (e.g.
        // TS5102 for the removed `baseUrl` the mirror strips and re-anchors),
        // and the real tree's `files`/`include` lists must not leak into the
        // virtual program anyway.
        let mut compiler_options = self.load_compiler_options(original_tsconfig.as_deref())?;

        // Capture the original path-alias map and type roots before stripping
        // path-sensitive options, so they can be re-anchored into the virtual
        // mirror below.
        let original_paths = compiler_options
            .get("paths")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let original_type_roots = compiler_options
            .get("typeRoots")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for option in PATH_SENSITIVE_COMPILER_OPTIONS {
            compiler_options.remove(*option);
        }
        normalize_native_removed_options(&mut compiler_options);
        compiler_options.insert("allowImportingTsExtensions".into(), Value::Bool(true));
        if self.needs_vue_jsx_compiler_options() {
            compiler_options
                .entry("jsx")
                .or_insert_with(|| Value::String("preserve".into()));
            compiler_options
                .entry("jsxImportSource")
                .or_insert_with(|| Value::String("vue".into()));
        }

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

        // Re-anchor custom `typeRoots` the same way: list the mirror copy and
        // the real-tree directory. TypeScript scans every listed root and
        // skips missing directories, so `types: [...]` entries served by a
        // custom root keep resolving instead of raising a false TS2688 that
        // only exists inside the mirror.
        if !original_type_roots.is_empty() {
            compiler_options.insert(
                "typeRoots".into(),
                Value::Array(self.remap_dir_entries(&original_type_roots)),
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
                self.include_paths(include_virtual_paths)
                    .into_iter()
                    .map(|path| Value::String(path.into()))
                    .collect(),
            ),
        );
        config.insert("exclude".into(), Value::Array(Vec::new()));

        Ok(Value::Object(config))
    }

    pub(super) fn needs_vue_jsx_compiler_options(&self) -> bool {
        self.virtual_files.values().any(|file| {
            file.virtual_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".vue.tsx"))
        })
    }

    fn include_paths(&self, include_virtual_paths: Option<&[&Path]>) -> Vec<CompactString> {
        let relative = |path: &Path| {
            path.strip_prefix(&self.virtual_root)
                .ok()
                .map(|path| path.to_string_lossy().to_compact_string())
        };
        let mut includes: Vec<_> = match include_virtual_paths {
            Some(paths) => paths.iter().filter_map(|path| relative(path)).collect(),
            None => self
                .virtual_files
                .keys()
                .filter_map(|path| relative(path))
                .collect(),
        };
        if !self.virtual_ts_options.auto_import_stubs.is_empty() {
            includes.push(AUTO_IMPORT_STUBS_FILE.into());
        }
        includes.push(VUE_MODULE_STUBS_FILE.into());
        if self.uses_shared_helpers() {
            includes.push(SHARED_HELPERS_FILE.into());
        }
        includes.sort();
        includes
    }
    #[allow(clippy::disallowed_types)]
    pub(super) fn load_compiler_options(
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

        // `extends` may be a single specifier or an array; array entries are
        // applied in order, with later entries overriding earlier ones, and
        // the extending file overriding them all.
        let mut inherited = Map::new();
        match config.get("extends") {
            Some(Value::String(extends)) => {
                if let Some(parent_path) = resolve_extended_tsconfig_path(&normalized, extends) {
                    inherited = self.load_compiler_options_inner(&parent_path, seen)?;
                }
            }
            Some(Value::Array(entries)) => {
                for extends in entries.iter().filter_map(Value::as_str) {
                    if let Some(parent_path) = resolve_extended_tsconfig_path(&normalized, extends)
                    {
                        inherited.extend(self.load_compiler_options_inner(&parent_path, seen)?);
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

    #[allow(clippy::disallowed_types)]
    fn normalize_paths_for_project_root(
        &self,
        compiler_options: &mut Map<std::string::String, Value>,
        base_dir: &Path,
    ) {
        // Relative path-ish options resolve against the tsconfig that declares
        // them; rebase them onto the project root so the flattened option set
        // keeps the declaring config's meaning.
        if let Some(type_roots) = compiler_options
            .get_mut("typeRoots")
            .and_then(Value::as_array_mut)
        {
            for entry in type_roots {
                let Some(raw_entry) = entry.as_str() else {
                    continue;
                };
                if Path::new(raw_entry).is_absolute() {
                    continue;
                }
                *entry = Value::String(
                    normalize_tsconfig_path_target(base_dir, &self.project_root, raw_entry).into(),
                );
            }
        }

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

    /// Re-anchor a list of project-root-relative directories (e.g. `typeRoots`)
    /// into the virtual mirror: each relative entry yields the mirror copy
    /// followed by the real source-tree directory. Absolute and non-string
    /// entries pass through unchanged.
    fn remap_dir_entries(&self, entries: &[Value]) -> Vec<Value> {
        let up = self.virtual_root_to_project_prefix();
        let mut remapped = Vec::with_capacity(entries.len() * 2);
        for entry in entries {
            let Some(entry_str) = entry.as_str() else {
                remapped.push(entry.clone());
                continue;
            };
            if Path::new(entry_str).is_absolute() {
                remapped.push(Value::String(entry_str.to_owned()));
                continue;
            }
            let core = entry_str.strip_prefix("./").unwrap_or(entry_str);
            remapped.push(Value::String(cstr!("./{core}").into()));
            remapped.push(Value::String(cstr!("{up}{core}").into()));
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

#[allow(clippy::disallowed_types)]
fn compiler_option_enabled(options: &Map<std::string::String, Value>, name: &str) -> bool {
    options.get(name).and_then(Value::as_bool).unwrap_or(false)
}
