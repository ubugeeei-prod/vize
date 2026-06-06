//! Virtual project management for Corsa-backed type checking.
//!
//! This module materializes a mirrored TypeScript project in
//! `node_modules/.vize/canon/` so Corsa can type-check Vue SFCs together with
//! regular TypeScript sources, ambient declarations, and emitted `.d.ts` files.

use std::path::{Path, PathBuf};

use super::error::{CorsaError, CorsaResult};
use super::import_rewriter::ImportRewriter;
use super::materialize_fs::{
    ensure_dir, ensure_materialize_root, prune_unexpected_entries, record_write_batch,
    write_file_untracked, write_if_changed,
};
use super::runtime_deps::materialize_runtime_dependencies;
use super::source_map::{CompositeSourceMap, SfcBlockRange, SfcSourceMap};
use super::{Diagnostic, SfcBlockType};
use crate::script_parse::collect_script_parse_diagnostics;
use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions, extract_interface_fields,
    generate_virtual_ts_with_offsets_and_checks,
};
use oxc_span::SourceType;
use rayon::prelude::*;
use serde_json::{Map, Value};
use vize_atelier_core::parser::parse;
use vize_atelier_sfc::{
    SfcDescriptor, SfcError, SfcParseOptions,
    croquis::{
        SfcCroquisOptions, analyze_sfc_descriptor_with_context,
        analyze_sfc_descriptor_with_context_legacy_vue2,
    },
    parse_sfc,
    script::ScriptCompileContext,
    validate_script_setup_semantics_located,
};
use vize_carton::{
    Bump, FxHashMap, FxHashSet, String as CompactString, ToCompactString, cstr, profile,
};

pub(super) const AUTO_IMPORT_STUBS_FILE: &str = "__vize_auto_imports.d.ts";
pub(super) const VUE_MODULE_STUBS_FILE: &str = "__vize_vue_modules.d.ts";
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

/// A virtual file in the project.
#[derive(Debug)]
pub struct VirtualFile {
    /// Generated or rewritten source code used by Corsa.
    pub content: CompactString,
    /// Source map for position mapping.
    pub source_map: CompositeSourceMap,
    /// Original file path.
    pub original_path: PathBuf,
    /// Materialized file path inside the virtual project.
    pub virtual_path: PathBuf,
}

/// Original position after mapping.
#[derive(Debug, Clone)]
pub struct OriginalPosition {
    /// Original file path.
    pub path: PathBuf,
    /// Line number (0-based).
    pub line: u32,
    /// Column number (0-based).
    pub column: u32,
    /// SFC block type if applicable.
    pub block_type: Option<SfcBlockType>,
}

/// Virtual project for Corsa-backed type checking.
pub struct VirtualProject {
    /// Project root directory.
    project_root: PathBuf,

    /// Virtual project root (`node_modules/.vize/canon`).
    virtual_root: PathBuf,

    /// Explicit tsconfig path, if one was provided by the caller.
    tsconfig_path: Option<PathBuf>,

    /// Global virtual TS options applied to every Vue file.
    virtual_ts_options: VirtualTsOptions,

    /// Internal check generation settings applied to every Vue file.
    virtual_ts_check_options: VirtualTsCheckOptions,

    /// Enable Vue 2.7 / Nuxt 2 Options API compatibility for virtual files.
    legacy_vue2: bool,

    /// Virtual files keyed by materialized path.
    virtual_files: FxHashMap<PathBuf, VirtualFile>,

    /// Non-TS module files that must exist in the virtual mirror for TypeScript
    /// module resolution, keyed by materialized path.
    passthrough_files: FxHashMap<PathBuf, PathBuf>,

    /// Secondary index: original source path -> materialized (virtual) path.
    /// Keeps `find_by_original` / `map_to_virtual` O(1) instead of scanning
    /// every virtual file on each LSP position-mapping request.
    original_index: FxHashMap<PathBuf, PathBuf>,

    /// Original source text as registered, keyed by materialized (virtual)
    /// path. Retained here (rather than on the public `VirtualFile`) so
    /// position mapping can convert offsets to line/column without re-reading
    /// the file from disk on every request, and without changing the public
    /// `VirtualFile` API. This is also the exact text the source map's
    /// original offsets refer to (e.g. an editor's unsaved buffer), so it is
    /// more correct than re-reading live disk state.
    original_contents: FxHashMap<PathBuf, CompactString>,

    /// Parser diagnostics collected before Corsa runs.
    diagnostics: Vec<Diagnostic>,

    /// Import rewriter for `.vue` specifiers inside TypeScript sources.
    rewriter: ImportRewriter,
}

impl VirtualProject {
    /// Create a new virtual project.
    pub fn new(project_root: &Path) -> CorsaResult<Self> {
        let project_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        let virtual_root = project_root
            .join("node_modules")
            .join(".vize")
            .join("canon");

        Ok(Self {
            project_root,
            virtual_root,
            tsconfig_path: None,
            virtual_ts_options: VirtualTsOptions::default(),
            virtual_ts_check_options: VirtualTsCheckOptions::default(),
            legacy_vue2: false,
            virtual_files: FxHashMap::default(),
            passthrough_files: FxHashMap::default(),
            original_index: FxHashMap::default(),
            original_contents: FxHashMap::default(),
            diagnostics: Vec::new(),
            rewriter: ImportRewriter::new(),
        })
    }

    /// Set the tsconfig path to extend.
    pub fn set_tsconfig_path(&mut self, tsconfig_path: Option<PathBuf>) {
        self.tsconfig_path = tsconfig_path;
    }

    /// Set the shared virtual TS options.
    pub fn set_virtual_ts_options(&mut self, options: VirtualTsOptions) {
        self.virtual_ts_options = options;
    }

    pub(crate) fn set_virtual_ts_check_options(&mut self, options: VirtualTsCheckOptions) {
        self.virtual_ts_check_options = options;
    }

    pub(crate) fn set_legacy_vue2(&mut self, enabled: bool) {
        self.legacy_vue2 = enabled;
    }

    /// Get the project root.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the virtual root.
    pub fn virtual_root(&self) -> &Path {
        &self.virtual_root
    }

    /// Register a supported file path.
    pub fn register_path(&mut self, path: &Path) -> CorsaResult<()> {
        let content = profile!("canon.file.read", std::fs::read_to_string(path))?;
        self.register_path_with_content(path, &content)
    }

    /// Register a supported file path with already-loaded content.
    pub fn register_path_with_content(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        let registered = build_registered_file(
            path,
            content,
            VirtualBuildContext {
                project_root: &self.project_root,
                virtual_root: &self.virtual_root,
                virtual_ts_options: &self.virtual_ts_options,
                virtual_ts_check_options: self.virtual_ts_check_options,
                legacy_vue2: self.legacy_vue2,
                rewriter: &self.rewriter,
            },
        )?;
        self.absorb_registered_file(registered);
        Ok(())
    }

    /// Register a batch of file paths, parallelizing per-file parse and Virtual TS
    /// generation across rayon's thread pool. Falls back to sequential work when
    /// the batch is small enough that the fan-out cost would dominate.
    ///
    /// This is deliberately structured as "parallel build, sequential absorb".
    /// `build_registered_file` owns the expensive work (disk read, SFC parse,
    /// template parse, virtual-TS generation, import rewriting) and only needs an
    /// immutable build context, so it scales cleanly across rayon workers. The
    /// mutable project indexes are updated after the join point, which preserves
    /// deterministic maps and avoids locking every insertion in the hot loop.
    pub fn register_paths(&mut self, paths: &[PathBuf]) -> CorsaResult<()> {
        let valid_paths: Vec<&Path> = paths
            .iter()
            .filter(|path| path.is_file())
            .map(PathBuf::as_path)
            .collect();
        if valid_paths.is_empty() {
            return Ok(());
        }

        // Sequential is cheaper for tiny batches than firing up rayon workers.
        if valid_paths.len() <= 1 {
            for path in valid_paths {
                self.register_path(path)?;
            }
            return Ok(());
        }

        let build_context = VirtualBuildContext {
            project_root: self.project_root.as_path(),
            virtual_root: self.virtual_root.as_path(),
            virtual_ts_options: &self.virtual_ts_options,
            virtual_ts_check_options: self.virtual_ts_check_options,
            legacy_vue2: self.legacy_vue2,
            rewriter: &self.rewriter,
        };

        let registered: Result<Vec<RegisteredFile>, CorsaError> = valid_paths
            .par_iter()
            .map(|&path| {
                let content = profile!("canon.file.read", std::fs::read_to_string(path))?;
                build_registered_file(path, &content, build_context)
            })
            .collect();

        self.virtual_files.reserve(valid_paths.len());
        for registered in registered? {
            self.absorb_registered_file(registered);
        }
        Ok(())
    }

    /// Register a `.vue` file.
    pub fn register_vue_file(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        let registered = build_vue_registered_file(
            path,
            content,
            VirtualBuildContext {
                project_root: &self.project_root,
                virtual_root: &self.virtual_root,
                virtual_ts_options: &self.virtual_ts_options,
                virtual_ts_check_options: self.virtual_ts_check_options,
                legacy_vue2: self.legacy_vue2,
                rewriter: &self.rewriter,
            },
        )?;
        self.absorb_registered_file(registered);
        Ok(())
    }

    /// Register a `.ts`/`.tsx`/`.mts`/`.cts` file.
    pub fn register_ts_file(&mut self, path: &Path) -> CorsaResult<()> {
        let content = std::fs::read_to_string(path)?;
        let source_type = source_type_for_path(path).ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
        self.register_script_file(path, &content, source_type)
    }

    /// Register a `.d.ts` file.
    pub fn register_declaration_file(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        self.register_script_file(path, content, SourceType::ts())
    }

    /// Register a non-Vue source file.
    pub fn register_script_file(
        &mut self,
        path: &Path,
        content: &str,
        source_type: SourceType,
    ) -> CorsaResult<()> {
        let registered = build_script_registered_file(
            path,
            content,
            source_type,
            &self.project_root,
            &self.virtual_root,
            &self.rewriter,
        )?;
        self.absorb_registered_file(registered);
        Ok(())
    }

    fn absorb_registered_file(&mut self, registered: RegisteredFile) {
        self.diagnostics.extend(registered.diagnostics);
        self.original_index.insert(
            registered.file.original_path.clone(),
            registered.file.virtual_path.clone(),
        );
        self.original_contents.insert(
            registered.file.virtual_path.clone(),
            registered.original_content,
        );
        for (virtual_path, original_path) in registered.passthrough_files {
            self.passthrough_files.insert(virtual_path, original_path);
        }
        self.virtual_files
            .insert(registered.file.virtual_path.clone(), registered.file);
    }

    /// Materialize the virtual project to disk for diagnostics collection.
    ///
    /// The materialized tree is a cache, but Corsa observes it as a real project.
    /// We therefore prune only entries outside the expected file/dir set and
    /// preserve nested runtime dependencies under `node_modules`. File writes are
    /// batched with directory creation de-duplicated per parent path; tsconfig and
    /// other stable control files still use `write_if_changed` because touching
    /// them can invalidate TypeScript's own filesystem caches.
    pub fn materialize(&self) -> CorsaResult<()> {
        let expected_files = self.expected_materialized_files();
        profile!(
            "canon.project.prepare_dir",
            ensure_materialize_root(&self.virtual_root)
        )?;

        profile!(
            "canon.project.gc",
            prune_unexpected_entries(
                &self.virtual_root,
                &expected_files,
                &[self.virtual_root.join("node_modules")]
            )
        )?;

        profile!(
            "canon.project.runtime_deps",
            materialize_runtime_dependencies(&self.project_root, &self.virtual_root)
        )?;

        profile!(
            "canon.project.write_files",
            (|| -> CorsaResult<()> {
                let mut created_dirs: FxHashSet<&Path> = FxHashSet::default();
                let mut write_calls = 0u64;
                let mut written_bytes = 0u64;
                for file in self.virtual_files.values() {
                    if let Some(parent) = file.virtual_path.parent()
                        && created_dirs.insert(parent)
                    {
                        ensure_dir(parent)?;
                    }
                    write_file_untracked(&file.virtual_path, file.content.as_bytes())?;
                    write_calls += 1;
                    written_bytes += file.content.len() as u64;
                }
                for (virtual_path, original_path) in &self.passthrough_files {
                    if let Some(parent) = virtual_path.parent()
                        && created_dirs.insert(parent)
                    {
                        ensure_dir(parent)?;
                    }
                    let content = std::fs::read(original_path)?;
                    write_file_untracked(virtual_path, &content)?;
                    write_calls += 1;
                    written_bytes += content.len() as u64;
                }
                record_write_batch(write_calls, written_bytes);
                Ok(())
            })()
        )?;

        profile!(
            "canon.project.write_auto_imports",
            self.write_auto_import_stubs()
        )?;

        profile!(
            "canon.project.write_vue_module_stubs",
            self.write_vue_module_stubs()
        )?;

        profile!(
            "canon.project.write_tsconfig",
            self.write_tsconfig_file(&self.virtual_root.join("tsconfig.json"), None, false)
        )?;
        Ok(())
    }

    /// Write a declaration-emitting tsconfig and return its path.
    pub fn write_declaration_tsconfig(
        &self,
        out_dir: &Path,
        declaration_map: bool,
    ) -> CorsaResult<PathBuf> {
        let config_path = self.virtual_root.join("tsconfig.declaration.json");
        profile!(
            "canon.project.write_dts_tsconfig",
            self.write_tsconfig_file(&config_path, Some(out_dir), declaration_map)
        )?;
        Ok(config_path)
    }

    /// Find a virtual file by its original path.
    pub fn find_by_original(&self, original_path: &Path) -> Option<&VirtualFile> {
        let virtual_path = self.original_index.get(original_path)?;
        self.virtual_files.get(virtual_path)
    }

    /// Find a virtual file by its materialized path.
    pub fn find_by_virtual(&self, virtual_path: &Path) -> Option<&VirtualFile> {
        self.virtual_files.get(virtual_path)
    }

    /// Return virtual files sorted by original path for deterministic output.
    pub fn virtual_files_sorted(&self) -> Vec<&VirtualFile> {
        let mut files: Vec<_> = self.virtual_files.values().collect();
        files.sort_by(|left, right| left.original_path.cmp(&right.original_path));
        files
    }

    /// Parser diagnostics collected while registering source files.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Map a virtual position to the original position.
    pub fn map_to_original(
        &self,
        virtual_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        let file = self.virtual_files.get(virtual_path)?;
        let virtual_offset = super::source_map::line_col_to_offset(&file.content, line, column)?;
        let (original_offset, _, block_type) =
            file.source_map.get_original_position(virtual_offset)?;
        let original_content = self.original_contents.get(&file.virtual_path)?;
        let (original_line, original_column) =
            super::source_map::offset_to_line_col(original_content, original_offset)?;

        Some(OriginalPosition {
            path: file.original_path.clone(),
            line: original_line,
            column: original_column,
            block_type,
        })
    }

    /// Map an original position to the virtual position.
    pub fn map_to_virtual(
        &self,
        original_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<(PathBuf, u32, u32)> {
        let file = self.find_by_original(original_path)?;
        let original_content = self.original_contents.get(&file.virtual_path)?;
        let original_offset =
            super::source_map::line_col_to_offset(original_content, line, column)?;
        let virtual_offset = if let Some(ref sfc_map) = file.source_map.sfc_map {
            for block in [
                SfcBlockType::ScriptSetup,
                SfcBlockType::Script,
                SfcBlockType::Template,
            ] {
                if let Some(virtual_offset) = sfc_map.get_virtual_offset(original_offset, block) {
                    let rewritten_offset = file
                        .source_map
                        .import_map
                        .get_virtual_offset(virtual_offset);
                    if let Some((virtual_line, virtual_column)) =
                        super::source_map::offset_to_line_col(&file.content, rewritten_offset)
                    {
                        return Some((file.virtual_path.clone(), virtual_line, virtual_column));
                    }
                }
            }
            return None;
        } else {
            file.source_map
                .import_map
                .get_virtual_offset(original_offset)
        };

        let (virtual_line, virtual_column) =
            super::source_map::offset_to_line_col(&file.content, virtual_offset)?;
        Some((file.virtual_path.clone(), virtual_line, virtual_column))
    }

    /// Get the number of registered files.
    pub fn file_count(&self) -> usize {
        self.virtual_files.len()
    }

    /// Check if the project has any files.
    pub fn is_empty(&self) -> bool {
        self.virtual_files.is_empty()
    }

    fn write_tsconfig_file(
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

    fn write_auto_import_stubs(&self) -> CorsaResult<()> {
        if self.virtual_ts_options.auto_import_stubs.is_empty() {
            return Ok(());
        }

        let capacity = self
            .virtual_ts_options
            .auto_import_stubs
            .iter()
            .fold(64usize, |acc, stub| acc + stub.len() + 1);
        let mut content = CompactString::with_capacity(capacity);
        content.push_str("// @ts-nocheck\n");
        content.push_str("// Framework-provided globals for the virtual project.\n");
        for stub in &self.virtual_ts_options.auto_import_stubs {
            content.push_str(stub);
            content.push('\n');
        }

        write_if_changed(
            &self.virtual_root.join(AUTO_IMPORT_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    fn write_vue_module_stubs(&self) -> CorsaResult<()> {
        let content = r#"declare module "*.vue" {
  const component: import("vue").DefineComponent<any, any, any>;
  export default component;
}

declare module "*.vue.ts" {
  const component: import("vue").DefineComponent<any, any, any>;
  export default component;
}
"#;
        write_if_changed(
            &self.virtual_root.join(VUE_MODULE_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    fn expected_materialized_files(&self) -> FxHashSet<PathBuf> {
        let mut files = FxHashSet::default();
        files.reserve(self.virtual_files.len() + 3);
        files.extend(self.virtual_files.keys().cloned());
        files.extend(self.passthrough_files.keys().cloned());
        if !self.virtual_ts_options.auto_import_stubs.is_empty() {
            files.insert(self.virtual_root.join(AUTO_IMPORT_STUBS_FILE));
        }
        files.insert(self.virtual_root.join(VUE_MODULE_STUBS_FILE));
        files.insert(self.virtual_root.join("tsconfig.json"));
        files
    }

    fn common_virtual_source_dir(&self) -> PathBuf {
        let mut parents = self
            .virtual_files
            .keys()
            .filter_map(|path| path.parent().map(Path::to_path_buf));
        let Some(mut common) = parents.next() else {
            return self.virtual_root.clone();
        };

        for parent in parents {
            while !parent.starts_with(&common) {
                if !common.pop() {
                    return self.virtual_root.clone();
                }
            }
        }

        common
    }

    fn resolved_tsconfig_path(&self) -> Option<PathBuf> {
        if let Some(ref tsconfig_path) = self.tsconfig_path {
            return Some(tsconfig_path.clone());
        }

        let tsconfig = self.project_root.join("tsconfig.json");
        tsconfig.exists().then_some(tsconfig)
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

struct GeneratedVueFile {
    code: CompactString,
    mappings: Vec<crate::virtual_ts::VizeMapping>,
    diagnostics: Vec<Diagnostic>,
}

fn generate_vue_virtual_ts(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    options: &VirtualTsOptions,
    check_options: VirtualTsCheckOptions,
    legacy_vue2: bool,
) -> CorsaResult<GeneratedVueFile> {
    let allocator = Bump::new();
    let mut diagnostics = Vec::new();

    if let Some(ref script) = descriptor.script {
        let script_diagnostics =
            collect_script_parse_diagnostics(&script.content, script.loc.start as u32);
        if !script_diagnostics.is_empty() {
            diagnostics.extend(script_diagnostics.into_iter().map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    SfcBlockType::Script,
                )
            }));
        }
    }

    if let Some(ref script_setup) = descriptor.script_setup {
        let script_diagnostics =
            collect_script_parse_diagnostics(&script_setup.content, script_setup.loc.start as u32);
        if !script_diagnostics.is_empty() {
            diagnostics.extend(script_diagnostics.into_iter().map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    SfcBlockType::ScriptSetup,
                )
            }));
        }
    }

    let template_offset = descriptor
        .template
        .as_ref()
        .map(|template| template.loc.start as u32)
        .unwrap_or(0);
    let template_ast = descriptor.template.as_ref().and_then(|template| {
        profile!("canon.template.parse", {
            let (root, errors) = parse(&allocator, &template.content);
            if errors.is_empty() {
                Some(root)
            } else {
                diagnostics.extend(errors.into_iter().map(|error| {
                    let start = error
                        .loc
                        .as_ref()
                        .map(|loc| template_offset + loc.start.offset)
                        .unwrap_or(template_offset);
                    diagnostic_for_offset(
                        path,
                        source,
                        start,
                        cstr!("Template parse error: {}", error.message),
                        SfcBlockType::Template,
                    )
                }));
                None
            }
        })
    });

    if !diagnostics.is_empty() {
        return Ok(GeneratedVueFile {
            code: invalid_sfc_fallback_virtual_ts(),
            mappings: Vec::new(),
            diagnostics,
        });
    }

    let croquis_options = SfcCroquisOptions::full();

    let analysis = profile!(
        "canon.croquis.analyze_sfc",
        if legacy_vue2 {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                descriptor,
                template_ast.as_ref(),
                croquis_options,
            )
        } else {
            analyze_sfc_descriptor_with_context(descriptor, template_ast.as_ref(), croquis_options)
        }
    );
    let vize_atelier_sfc::croquis::SfcCroquisAnalysis {
        mut croquis,
        script_content,
        script_offset,
    } = analysis;
    profile!(
        "canon.croquis.augment_type_props",
        augment_type_based_props_from_script_context(&mut croquis, descriptor, path)
    );

    let output = profile!(
        "canon.virtual_ts.generate",
        generate_virtual_ts_with_offsets_and_checks(
            &croquis,
            script_content.as_deref(),
            template_ast.as_ref(),
            script_offset,
            template_offset,
            options,
            VirtualTsGenerationOptions {
                check_options,
                legacy_vue2,
            },
        )
    );

    // Surface Vue-specific semantic errors (e.g. DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE)
    // that the SFC compiler catches but TypeScript itself does not. Without this,
    // `vize check` would silently accept SFCs that `vize build` rejects.
    if let Some(diagnostic) = profile!(
        "canon.sfc.compile_validate",
        collect_sfc_compile_diagnostic(path, source, descriptor)
    ) {
        diagnostics.push(diagnostic);
    }

    Ok(GeneratedVueFile {
        code: output.code,
        mappings: output.mappings,
        diagnostics,
    })
}

fn augment_type_based_props_from_script_context(
    croquis: &mut vize_croquis::Croquis,
    descriptor: &SfcDescriptor<'_>,
    path: &Path,
) {
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return;
    };
    if croquis
        .macros
        .define_props()
        .is_none_or(|call| call.type_args.is_none())
    {
        return;
    }

    let mut ctx = ScriptCompileContext::new(&script_setup.content);
    let path_string = path.to_string_lossy();

    if let Some(script) = descriptor.script.as_ref()
        && !script.content.is_empty()
    {
        ctx.collect_types_from(&script.content);
        ctx.collect_imported_types_from_path(&script.content, path_string.as_ref());
    }
    ctx.collect_imported_types_from_path(&script_setup.content, path_string.as_ref());
    ctx.analyze();

    let known_props = known_type_based_prop_names(croquis, &script_setup.content);
    let mut missing_props: Vec<CompactString> = ctx
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            matches!(binding_type, vize_relief::BindingType::Props)
                .then(|| name)
                .filter(|name| !known_props.contains(*name))
                .cloned()
        })
        .collect();
    if missing_props.is_empty() {
        return;
    }
    missing_props.sort();

    for name in missing_props {
        croquis
            .bindings
            .bindings
            .entry(name.clone())
            .or_insert(vize_relief::BindingType::Props);
        croquis
            .macros
            .add_prop(vize_croquis::macros::PropDefinition {
                name,
                prop_type: None,
                required: false,
                default_value: None,
            });
    }
}

fn known_type_based_prop_names(
    croquis: &vize_croquis::Croquis,
    script_setup: &str,
) -> FxHashSet<CompactString> {
    let mut names: FxHashSet<CompactString> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.clone())
        .collect();

    let Some(type_args) = croquis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
    else {
        return names;
    };

    let type_name = strip_outer_angle_brackets(type_args.trim());
    for prop in croquis.types.extract_properties(type_name) {
        names.insert(prop.name);
    }
    for field in extract_interface_fields(script_setup, type_name) {
        names.insert(CompactString::new(field));
    }

    names
}

fn strip_outer_angle_brackets(value: &str) -> &str {
    value
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(value)
}

/// Run only the script-setup semantic validators on this SFC. We deliberately
/// avoid `compile_sfc` here — it would do template codegen and script transform
/// work that doubles the wall time of `vize check` (see the regression on PR
/// #675). The validator covers the diagnostics TypeScript cannot derive on its
/// own; parse-level errors are already collected above.
fn collect_sfc_compile_diagnostic(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
) -> Option<Diagnostic> {
    let script_setup = descriptor.script_setup.as_ref()?;

    // Cheap pre-filter: the only validator we currently run targets
    // `const { ... = ... } = defineProps<...>()`. Skip the OXC parse entirely
    // when none of those tokens appear, which is the common case for app
    // components without destructured typed props.
    if !script_setup_has_validator_candidates(&script_setup.content) {
        return None;
    }

    match validate_script_setup_semantics_located(
        &script_setup.content,
        script_setup.loc.start,
        source,
    ) {
        Ok(()) => None,
        Err(error) => Some(sfc_error_to_diagnostic(path, source, descriptor, &error)),
    }
}

/// Cheap byte-level filter — must be a strict superset of the patterns the
/// underlying validators actually fire on, so we never miss a real diagnostic.
fn script_setup_has_validator_candidates(content: &str) -> bool {
    // Validator needs: typed defineProps (`defineProps<...>`) AND a destructure
    // pattern (`{ ... = ... } = defineProps`). The combined presence of these
    // two substrings is a tight enough filter for typical app code.
    content.contains("defineProps<") && content.contains("= defineProps")
}

fn sfc_error_to_diagnostic(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    error: &SfcError,
) -> Diagnostic {
    let (line, column, block_type) = if let Some(loc) = error.loc.as_ref() {
        // BlockLocation lines/columns are 1-based; Diagnostic stores them 0-based.
        let line = (loc.start_line as u32).saturating_sub(1);
        let column = (loc.start_column as u32).saturating_sub(1);
        (line, column, None)
    } else {
        let (offset, block_type) = default_diagnostic_offset(descriptor);
        let (line, column) = line_column_for_offset(source, offset);
        (line, column, Some(block_type))
    };

    let message = match error.code.as_deref() {
        Some(code) => cstr!("Vue compile error [{}]: {}", code, error.message),
        None => cstr!("Vue compile error: {}", error.message),
    };

    Diagnostic {
        file: path.to_path_buf(),
        line,
        column,
        message,
        code: None,
        severity: 1,
        block_type,
    }
}

/// Best-effort fallback location for SFC compile errors that carry no `loc`.
/// Points at the start of the most relevant block so the diagnostic lands
/// somewhere clickable instead of at file offset 0.
fn default_diagnostic_offset(descriptor: &SfcDescriptor) -> (u32, SfcBlockType) {
    if let Some(setup) = descriptor.script_setup.as_ref() {
        return (setup.loc.start as u32, SfcBlockType::ScriptSetup);
    }
    if let Some(script) = descriptor.script.as_ref() {
        return (script.loc.start as u32, SfcBlockType::Script);
    }
    if let Some(template) = descriptor.template.as_ref() {
        return (template.loc.start as u32, SfcBlockType::Template);
    }
    (0, SfcBlockType::Script)
}

fn invalid_sfc_fallback_virtual_ts() -> CompactString {
    "declare const __vize_component: any;\nexport default __vize_component;\n".into()
}

fn diagnostic_for_offset(
    path: &Path,
    source: &str,
    start: u32,
    message: CompactString,
    block_type: SfcBlockType,
) -> Diagnostic {
    let (line, column) = line_column_for_offset(source, start);
    Diagnostic {
        file: path.to_path_buf(),
        line,
        column,
        message,
        code: None,
        severity: 1,
        block_type: Some(block_type),
    }
}

fn line_column_for_offset(source: &str, offset: u32) -> (u32, u32) {
    let target = (offset as usize).min(source.len());
    let mut line = 0;
    let mut line_start = 0;

    for (index, character) in source.char_indices() {
        if index >= target {
            break;
        }
        if character == '\n' {
            line += 1;
            line_start = index + 1;
        }
    }

    (line, target.saturating_sub(line_start) as u32)
}

fn collect_sfc_block_ranges(descriptor: &SfcDescriptor) -> Vec<SfcBlockRange> {
    let mut blocks = Vec::with_capacity(3);
    if let Some(template) = descriptor.template.as_ref() {
        push_block_range(
            &mut blocks,
            template.loc.start as u32,
            template.content.len() as u32,
            SfcBlockType::Template,
        );
    }
    if let Some(script) = descriptor.script.as_ref() {
        push_block_range(
            &mut blocks,
            script.loc.start as u32,
            script.content.len() as u32,
            SfcBlockType::Script,
        );
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        push_block_range(
            &mut blocks,
            script_setup.loc.start as u32,
            script_setup.content.len() as u32,
            SfcBlockType::ScriptSetup,
        );
    }
    blocks
}

fn push_block_range(
    blocks: &mut Vec<SfcBlockRange>,
    start: u32,
    len: u32,
    block_type: SfcBlockType,
) {
    if len == 0 {
        return;
    }
    blocks.push(SfcBlockRange {
        start,
        end: start + len,
        block_type,
    });
}

/// Result of building a virtual file for a registered path, owned and
/// independent of any `&mut VirtualProject` so it can be produced in parallel.
struct RegisteredFile {
    file: VirtualFile,
    /// Original source text as registered, retained for offset<->line/col
    /// mapping without a disk re-read. Stored on the project, not the public
    /// `VirtualFile`.
    original_content: CompactString,
    passthrough_files: Vec<(PathBuf, PathBuf)>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
struct VirtualBuildContext<'a> {
    project_root: &'a Path,
    virtual_root: &'a Path,
    virtual_ts_options: &'a VirtualTsOptions,
    virtual_ts_check_options: VirtualTsCheckOptions,
    legacy_vue2: bool,
    rewriter: &'a ImportRewriter,
}

fn build_registered_file(
    path: &Path,
    content: &str,
    context: VirtualBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    if path.extension().and_then(|extension| extension.to_str()) == Some("vue") {
        return build_vue_registered_file(path, content, context);
    }

    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return build_script_registered_file(
            path,
            content,
            SourceType::ts(),
            context.project_root,
            context.virtual_root,
            context.rewriter,
        );
    }

    let source_type = source_type_for_path(path).ok_or_else(|| CorsaError::PathError {
        path: path.to_path_buf(),
    })?;
    build_script_registered_file(
        path,
        content,
        source_type,
        context.project_root,
        context.virtual_root,
        context.rewriter,
    )
}

fn build_vue_registered_file(
    path: &Path,
    content: &str,
    context: VirtualBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    let descriptor = profile!(
        "canon.sfc.parse",
        parse_sfc(
            content,
            SfcParseOptions {
                filename: path.to_string_lossy().to_compact_string(),
                ..Default::default()
            },
        )
        .map_err(|error| CorsaError::SfcParse(error.message.to_compact_string()))
    )?;

    let mut effective_options =
        virtual_ts_options_for_descriptor(context.virtual_ts_options, &descriptor);
    effective_options.auto_import_stubs.clear();
    let generated = profile!(
        "canon.vue.virtual_ts",
        generate_vue_virtual_ts(
            path,
            content,
            &descriptor,
            &effective_options,
            context.virtual_ts_check_options,
            context.legacy_vue2,
        )
    )?;
    let GeneratedVueFile {
        code,
        mappings,
        diagnostics,
    } = generated;
    let rewritten = profile!(
        "canon.import.rewrite.vue",
        context.rewriter.rewrite(&code, SourceType::ts())
    );
    let source_map = CompositeSourceMap::new_vue(
        SfcSourceMap::new(mappings, collect_sfc_block_ranges(&descriptor)),
        rewritten.source_map,
    );
    let virtual_path = virtual_vue_path(context.project_root, context.virtual_root, path)?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map,
            original_path: path.to_path_buf(),
            virtual_path,
        },
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_json_modules(
            path,
            content,
            context.project_root,
            context.virtual_root,
        ),
        diagnostics,
    })
}

fn build_script_registered_file(
    path: &Path,
    content: &str,
    source_type: SourceType,
    project_root: &Path,
    virtual_root: &Path,
    rewriter: &ImportRewriter,
) -> CorsaResult<RegisteredFile> {
    let rewritten = profile!(
        "canon.import.rewrite.script",
        rewriter.rewrite(content, source_type)
    );
    let virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map: CompositeSourceMap::new_script(rewritten.source_map),
            original_path: path.to_path_buf(),
            virtual_path,
        },
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_json_modules(
            path,
            content,
            project_root,
            virtual_root,
        ),
        diagnostics: Vec::new(),
    })
}

fn collect_passthrough_json_modules(
    path: &Path,
    content: &str,
    project_root: &Path,
    virtual_root: &Path,
) -> Vec<(PathBuf, PathBuf)> {
    let Some(dir) = path.parent() else {
        return Vec::new();
    };

    let mut seen = FxHashSet::default();
    let mut files = Vec::new();
    for specifier in extract_relative_module_specifiers(content) {
        let Some(original_path) = resolve_relative_json_module(dir, &specifier) else {
            continue;
        };
        let Ok(virtual_path) = mirrored_virtual_path(project_root, virtual_root, &original_path)
        else {
            continue;
        };
        if seen.insert(virtual_path.clone()) {
            files.push((virtual_path, original_path));
        }
    }
    files
}

fn extract_relative_module_specifiers(source: &str) -> Vec<CompactString> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut specifiers = Vec::new();
    let mut i = 0;

    while i < len {
        let keyword_len = if matches_keyword(bytes, i, b"from") {
            4
        } else if matches_keyword(bytes, i, b"import") {
            6
        } else {
            i += 1;
            continue;
        };

        let mut j = i + keyword_len;
        while j < len && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < len && bytes[j] == b'(' {
            j += 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
        }

        if j < len && (bytes[j] == b'"' || bytes[j] == b'\'') {
            let quote = bytes[j];
            let start = j + 1;
            let mut k = start;
            while k < len && bytes[k] != quote {
                k += 1;
            }
            if k < len {
                let specifier = &source[start..k];
                if is_relative_specifier(specifier) {
                    specifiers.push(specifier.to_compact_string());
                }
                i = k + 1;
                continue;
            }
        }

        i += keyword_len;
    }

    specifiers
}

fn matches_keyword(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    if at + keyword.len() > bytes.len() || &bytes[at..at + keyword.len()] != keyword {
        return false;
    }
    let before_ok = at == 0 || !is_identifier_byte(bytes[at - 1]);
    let after = at + keyword.len();
    let after_ok = after >= bytes.len() || !is_identifier_byte(bytes[after]);
    before_ok && after_ok
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
}

fn resolve_relative_json_module(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);

    if specifier.ends_with(".json") && base.is_file() {
        return Some(normalize_existing_path(&base));
    }

    let candidate = append_json_extension(&base);
    if candidate.is_file() {
        return Some(normalize_existing_path(&candidate));
    }

    let candidate = base.join("index.json");
    if candidate.is_file() {
        return Some(normalize_existing_path(&candidate));
    }

    None
}

fn append_json_extension(base: &Path) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}.json")),
        None => base.to_path_buf(),
    }
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn virtual_ts_options_for_descriptor(
    base: &VirtualTsOptions,
    descriptor: &SfcDescriptor,
) -> VirtualTsOptions {
    let css_modules: Vec<CompactString> = descriptor
        .styles
        .iter()
        .filter_map(|style| {
            style
                .module
                .as_ref()
                .map(|module| module.to_compact_string())
        })
        .collect();
    if css_modules.is_empty() {
        return base.clone();
    }

    let mut options = base.clone();
    options.css_modules = css_modules;
    options
}

fn mirrored_virtual_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
) -> CorsaResult<PathBuf> {
    let relative = path.strip_prefix(project_root)?;
    Ok(virtual_root.join(relative))
}

fn virtual_vue_path(project_root: &Path, virtual_root: &Path, path: &Path) -> CorsaResult<PathBuf> {
    let mut virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;
    let file_name = virtual_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| cstr!("{name}.ts"))
        .ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
    virtual_path.set_file_name(file_name.as_str());
    Ok(virtual_path)
}

fn source_type_for_path(path: &Path) -> Option<SourceType> {
    let file_name = path.file_name()?.to_str()?;
    if file_name.ends_with(".tsx") {
        return Some(SourceType::tsx());
    }
    if file_name.ends_with(".ts")
        || file_name.ends_with(".d.ts")
        || file_name.ends_with(".mts")
        || file_name.ends_with(".cts")
    {
        return Some(SourceType::ts());
    }
    None
}

fn resolve_extended_tsconfig_path(tsconfig_path: &Path, extends: &str) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let extends_path = Path::new(extends);
    if !(extends_path.is_absolute()
        || extends.starts_with("./")
        || extends.starts_with("../")
        || extends == "."
        || extends == "..")
    {
        return None;
    }

    let base = if extends_path.is_absolute() {
        extends_path.to_path_buf()
    } else {
        base_dir.join(extends_path)
    };

    tsconfig_path_candidates(base)
        .into_iter()
        .map(|candidate| normalize_path_lexically(&candidate))
        .find(|candidate| candidate.exists())
}

fn tsconfig_path_candidates(base: PathBuf) -> Vec<PathBuf> {
    if base.extension().is_some() {
        return vec![base];
    }

    vec![
        base.clone(),
        base.with_extension("json"),
        base.join("tsconfig.json"),
    ]
}

fn normalize_tsconfig_path_target(
    base_dir: &Path,
    project_root: &Path,
    target: &str,
) -> CompactString {
    let normalized = normalize_path_lexically(&base_dir.join(target));
    if let Ok(relative) = normalized.strip_prefix(project_root) {
        return path_to_tsconfig_target(relative);
    }
    path_to_tsconfig_target(&normalized)
}

fn path_to_tsconfig_target(path: &Path) -> CompactString {
    path.to_string_lossy().replace('\\', "/").into()
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn parse_jsonc_value(content: &str) -> CorsaResult<Value> {
    let stripped = strip_json_comments(content);
    let normalized = strip_trailing_commas(&stripped);
    Ok(serde_json::from_str(&normalized)?)
}

fn strip_json_comments(content: &str) -> CompactString {
    let mut output = CompactString::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            }
            continue;
        }

        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            }
            continue;
        }

        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            block_comment = true;
            continue;
        }

        output.push(ch);
    }

    output
}

fn strip_trailing_commas(content: &str) -> CompactString {
    let mut output = CompactString::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut lookahead = index + 1;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && matches!(chars[lookahead], '}' | ']') {
                index += 1;
                continue;
            }
        }

        output.push(ch);
        index += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{
        AUTO_IMPORT_STUBS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject, parse_jsonc_value,
        source_type_for_path, strip_json_comments,
    };
    use crate::batch::SfcBlockType;
    use crate::virtual_ts::VirtualTsOptions;
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
    }

    fn assert_ts_parses(source: &str) {
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
        assert!(
            parsed.errors.is_empty(),
            "virtual TS should parse without errors: {:?}",
            parsed.errors
        );
    }

    #[test]
    fn test_virtual_project_new() {
        let case_dir = unique_case_dir("new");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(&case_dir).unwrap();

        let project = VirtualProject::new(&case_dir).unwrap();

        assert_eq!(project.project_root(), case_dir.as_path());
        assert!(project.virtual_root().ends_with("node_modules/.vize/canon"));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_materialize_writes_vue_module_stubs() {
        let case_dir = unique_case_dir("vue-module-stubs");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let main_path = src_dir.join("main.ts");
        fs::write(&main_path, "import App from './App.vue';\nvoid App;\n").unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&main_path).unwrap();
        project.materialize().unwrap();

        let stubs =
            fs::read_to_string(project.virtual_root().join("__vize_vue_modules.d.ts")).unwrap();
        assert!(stubs.contains(r#"declare module "*.vue.ts""#));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_rewrites_child_imports() {
        let case_dir = unique_case_dir("register-vue");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("App.vue");
        let vue_content = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child :count="count" />
</template>
"#;
        fs::write(&vue_path, vue_content).unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        let virtual_file = project.find_by_original(&vue_path).unwrap();
        insta::assert_snapshot!(virtual_file.content.as_str());

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_rewrites_options_api_export_default() {
        let case_dir = unique_case_dir("options-api-export-default");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("OptionsApi.vue");
        let vue_content = r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "OptionsApi",
});
</script>

<template>
  <div>hello</div>
</template>
"#;
        fs::write(&vue_path, vue_content).unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        let virtual_file = project.find_by_original(&vue_path).unwrap();
        insta::assert_snapshot!(virtual_file.content.as_str());
        assert_ts_parses(virtual_file.content.as_str());

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_reports_script_parse_error_with_fallback() {
        let case_dir = unique_case_dir("script-parse-error");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("Broken.vue");
        let vue_content = r#"<script setup lang="ts">
const count =
</script>

<template>
  <div>{{ count }}</div>
</template>
"#;

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Script parse error"));
        assert_eq!(diagnostics[0].block_type, Some(SfcBlockType::ScriptSetup));

        let virtual_file = project.find_by_original(&vue_path).unwrap();
        assert!(
            virtual_file
                .content
                .contains("export default __vize_component")
        );
        assert!(!virtual_file.content.contains("const count ="));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_reports_props_destructure_default_type_mismatch() {
        // Regression for: `const { msg = 0 } = defineProps<{ msg?: string }>()` should
        // surface in `vize check`. TypeScript itself does not flag the mismatch
        // (destructure defaults widen the binding's type), so the diagnostic has
        // to come from the SFC compiler's validator.
        let case_dir = unique_case_dir("props-destructure-default-type");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("Bad.vue");
        let vue_content = r#"<script setup lang="ts">
const { msg = 0 } = defineProps<{ msg?: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 1, "expected one SFC compile diagnostic");
        let diagnostic = &diagnostics[0];
        assert!(
            diagnostic
                .message
                .contains("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE"),
            "expected DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE in message, got: {}",
            diagnostic.message
        );
        assert!(
            diagnostic.message.contains("Default value of prop \"msg\""),
            "expected message to name the prop, got: {}",
            diagnostic.message
        );
        assert_eq!(diagnostic.severity, 1);

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_allows_matching_props_destructure_default() {
        let case_dir = unique_case_dir("props-destructure-default-ok");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("Good.vue");
        let vue_content = r#"<script setup lang="ts">
const { msg = "ok" } = defineProps<{ msg?: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        assert!(
            project.diagnostics().is_empty(),
            "no diagnostics expected for matching default, got: {:?}",
            project.diagnostics()
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_reports_template_parse_error_with_fallback() {
        let case_dir = unique_case_dir("template-parse-error");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("BrokenTemplate.vue");
        let vue_content = r#"<script setup lang="ts">
const count = 1
</script>

<template><div>{{ count }}</template>
"#;

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        let diagnostics = project.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Template parse error"));
        assert_eq!(diagnostics[0].block_type, Some(SfcBlockType::Template));

        let virtual_file = project.find_by_original(&vue_path).unwrap();
        assert!(
            virtual_file
                .content
                .contains("export default __vize_component")
        );
        assert!(!virtual_file.content.contains("__vize_check_template"));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_virtual_ts_exposes_props_from_reexported_vue_interface() {
        let case_dir = unique_case_dir("reexported-vue-interface-props");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let base = src_dir.join("Base.vue");
        let index = src_dir.join("index.ts");
        let child = src_dir.join("Child.vue");
        let parent = src_dir.join("ParentWidget.vue");

        fs::write(
            &base,
            r#"<script lang="ts">
export interface BaseProps {
  as?: string;
  asChild?: boolean;
}
</script>
<template><div></div></template>"#,
        )
        .unwrap();
        fs::write(&index, r#"export { type BaseProps } from "./Base.vue";"#).unwrap();
        fs::write(
            &child,
            r#"<script setup lang="ts">
defineProps<{ as?: string; asChild?: boolean }>();
</script>
<template><div></div></template>"#,
        )
        .unwrap();
        fs::write(
            &parent,
            r#"<script lang="ts">
import type { BaseProps } from "./index";

export interface ParentWidgetProps extends BaseProps {}
</script>
<script setup lang="ts">
import Child from "./Child.vue";

const props = defineProps<ParentWidgetProps>();
</script>
<template>
  <Child :as="as" :as-child="props.asChild" />
</template>"#,
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project
            .register_paths(&[base, index, child, parent.clone()])
            .unwrap();

        let virtual_parent = project.find_by_original(&parent).unwrap();
        assert_ts_parses(&virtual_parent.content);
        assert!(
            virtual_parent
                .content
                .contains(r#"const _as = props["as"];"#),
            "{}",
            virtual_parent.content
        );
        assert!(
            virtual_parent.content.contains(r#"void (props["as"]);"#),
            "{}",
            virtual_parent.content
        );
        assert!(
            virtual_parent
                .content
                .contains(r#"type Props = ParentWidgetProps;"#),
            "{}",
            virtual_parent.content
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_virtual_ts_preserves_ts_as_assertions_when_prop_is_named_as() {
        let case_dir = unique_case_dir("template-as-assertion-prop");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let vue_path = src_dir.join("App.vue");
        fs::write(
            &vue_path,
            r#"<script setup lang="ts">
defineProps<{
  as?: string
}>()

const value = 'demo'
const onFocus = (target: HTMLElement) => {
  target.dataset.focused = 'true'
}
</script>

<template>
  <div
    :data-value="(value as any)"
    :style="{
      ['--demo-value' as any]: value,
    }"
    v-on="{
      focusin: (event: FocusEvent) => {
        onFocus(event.target as HTMLElement)
      },
    }"
  ></div>
</template>
"#,
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project
            .register_vue_file(&vue_path, &fs::read_to_string(&vue_path).unwrap())
            .unwrap();

        let virtual_file = project.find_by_original(&vue_path).unwrap();
        assert_ts_parses(&virtual_file.content);
        assert!(
            virtual_file.content.contains("void ((value as any));"),
            "{}",
            virtual_file.content
        );
        assert!(
            virtual_file
                .content
                .contains("['--demo-value' as any]: value"),
            "{}",
            virtual_file.content
        );
        assert!(
            virtual_file
                .content
                .contains("onFocus(event.target as HTMLElement)"),
            "{}",
            virtual_file.content
        );
        assert!(
            !virtual_file.content.contains(r#"value props["as"] any"#),
            "{}",
            virtual_file.content
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_materialize_writes_tsconfig_and_virtual_files() {
        let case_dir = unique_case_dir("materialize");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("App.vue");
        fs::write(
            &vue_path,
            r#"<script setup lang="ts">
const message = 'Hello'
</script>

<template>
  <div>{{ message }}</div>
</template>
"#,
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        let mut options = VirtualTsOptions::default();
        options
            .auto_import_stubs
            .push("declare function autoGenerated(): string;".into());
        project.set_virtual_ts_options(options);
        project.register_path(&vue_path).unwrap();
        project.materialize().unwrap();

        let virtual_vue_path = case_dir.join("node_modules/.vize/canon/src/App.vue.ts");
        let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
        let auto_imports_path = case_dir.join("node_modules/.vize/canon/__vize_auto_imports.d.ts");

        assert!(virtual_vue_path.exists());
        assert!(tsconfig_path.exists());
        assert!(auto_imports_path.exists());
        assert!(
            !fs::read_to_string(&virtual_vue_path)
                .unwrap()
                .contains("autoGenerated")
        );
        assert!(
            fs::read_to_string(&auto_imports_path)
                .unwrap()
                .contains("autoGenerated")
        );
        assert!(
            fs::read_to_string(&tsconfig_path)
                .unwrap()
                .contains("__vize_auto_imports.d.ts")
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_materialize_writes_relative_json_modules() {
        let case_dir = unique_case_dir("materialize-json-modules");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        let token_dir = src_dir.join("tokens/source");
        fs::create_dir_all(&token_dir).unwrap();
        let ts_path = src_dir.join("tokens.ts");
        let json_path = token_dir.join("colors.tokens.json");
        fs::write(
            &ts_path,
            "import colors from './tokens/source/colors.tokens.json'\nvoid colors\n",
        )
        .unwrap();
        fs::write(&json_path, "{\"primary\":\"#0057ff\"}\n").unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&ts_path).unwrap();
        project.materialize().unwrap();

        let virtual_json_path =
            case_dir.join("node_modules/.vize/canon/src/tokens/source/colors.tokens.json");
        assert_eq!(
            fs::read_to_string(&virtual_json_path).unwrap(),
            "{\"primary\":\"#0057ff\"}\n"
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn materialize_prunes_stale_virtual_project_entries() {
        let case_dir = unique_case_dir("materialize-gc");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("App.vue");
        fs::write(
            &vue_path,
            r#"<script setup lang="ts">
const message = 'Hello'
</script>

<template>
  <div>{{ message }}</div>
</template>
"#,
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        let mut options = VirtualTsOptions::default();
        options
            .auto_import_stubs
            .push("declare function autoGenerated(): string;".into());
        project.set_virtual_ts_options(options);
        project.register_path(&vue_path).unwrap();
        project.materialize().unwrap();

        let virtual_root = project.virtual_root().to_path_buf();
        let stale_file = virtual_root.join("src/Old.vue.ts");
        let stale_dir_file = virtual_root.join("stale/nested/Unused.vue.ts");
        let stale_dts_config = virtual_root.join("tsconfig.declaration.json");
        let stale_package = virtual_root.join("node_modules/unused/package.json");
        fs::write(&stale_file, "export default {}").unwrap();
        fs::create_dir_all(stale_dir_file.parent().unwrap()).unwrap();
        fs::write(&stale_dir_file, "export default {}").unwrap();
        fs::write(&stale_dts_config, "{}").unwrap();
        fs::create_dir_all(stale_package.parent().unwrap()).unwrap();
        fs::write(&stale_package, "{}").unwrap();
        #[cfg(unix)]
        {
            let expected_virtual_file = virtual_root.join("src/App.vue.ts");
            let hijack_target = case_dir.join("hijack.ts");
            fs::write(&hijack_target, "hijacked").unwrap();
            fs::remove_file(&expected_virtual_file).unwrap();
            std::os::unix::fs::symlink(&hijack_target, &expected_virtual_file).unwrap();
        }

        let mut next_project = VirtualProject::new(&case_dir).unwrap();
        next_project.register_path(&vue_path).unwrap();
        next_project.materialize().unwrap();

        assert!(!stale_file.exists());
        assert!(!stale_dir_file.exists());
        assert!(!stale_dir_file.parent().unwrap().exists());
        assert!(!stale_dts_config.exists());
        assert!(!virtual_root.join(AUTO_IMPORT_STUBS_FILE).exists());
        assert!(!stale_package.exists());
        assert!(!stale_package.parent().unwrap().exists());
        assert!(virtual_root.join("src/App.vue.ts").exists());
        #[cfg(unix)]
        {
            let virtual_file_metadata =
                fs::symlink_metadata(virtual_root.join("src/App.vue.ts")).unwrap();
            assert!(!virtual_file_metadata.file_type().is_symlink());
            assert_eq!(
                fs::read_to_string(case_dir.join("hijack.ts")).unwrap(),
                "hijacked"
            );
        }
        assert!(virtual_root.join(VUE_MODULE_STUBS_FILE).exists());
        assert!(virtual_root.join("tsconfig.json").exists());

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn materialized_tsconfig_preserves_original_path_option_bases() {
        let case_dir = unique_case_dir("tsconfig-path-bases");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    },
    "rootDirs": ["src", "generated"],
    "typeRoots": ["types"]
  }
}"#,
        )
        .unwrap();
        let vue_path = src_dir.join("App.vue");
        fs::write(
            &vue_path,
            "<script setup lang=\"ts\">const count = 1</script>",
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&vue_path).unwrap();
        project.materialize().unwrap();

        let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
        let compiler_options = value["compilerOptions"].as_object().unwrap();

        assert_eq!(compiler_options["strict"], serde_json::Value::Bool(true));
        assert_eq!(
            compiler_options["allowImportingTsExtensions"],
            serde_json::Value::Bool(true)
        );
        for option in ["baseUrl", "rootDir", "rootDirs", "typeRoots"] {
            assert!(
                !compiler_options.contains_key(option),
                "{option} should remain owned by the extended tsconfig"
            );
        }

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn materialized_tsconfig_reanchors_paths_into_virtual_mirror() {
        let case_dir = unique_case_dir("tsconfig-paths-reanchor");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r##"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "#shared": ["./shared/index.ts"]
    }
  }
}"##,
        )
        .unwrap();
        let vue_path = src_dir.join("App.vue");
        fs::write(
            &vue_path,
            "<script setup lang=\"ts\">const count = 1</script>",
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&vue_path).unwrap();
        project.materialize().unwrap();

        let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
        let paths = value["compilerOptions"]["paths"].as_object().unwrap();

        // Each target gets a mirror candidate (relative to the virtual tsconfig
        // in `node_modules/.vize/canon`) first, then the real-tree fallback.
        assert_eq!(
            paths["@/*"],
            serde_json::json!(["./src/*", "../../../src/*"])
        );
        assert_eq!(
            paths["#shared"],
            serde_json::json!(["./shared/index.ts", "../../../shared/index.ts"])
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn materialized_tsconfig_reanchors_extended_paths_from_declaring_config_dir() {
        let case_dir = unique_case_dir("tsconfig-extended-paths-reanchor");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join(".nuxt")).unwrap();
        fs::create_dir_all(case_dir.join("app/components")).unwrap();
        fs::write(
            case_dir.join(".nuxt/tsconfig.json"),
            r##"{
  "compilerOptions": {
    "paths": {
      "~/*": ["../app/*"],
      "#imports": ["./imports"]
    }
  }
}"##,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
        )
        .unwrap();
        let vue_path = case_dir.join("app/components/App.vue");
        fs::write(
            &vue_path,
            "<script setup lang=\"ts\">const count = 1</script>",
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&vue_path).unwrap();
        project.materialize().unwrap();

        let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
        let paths = value["compilerOptions"]["paths"].as_object().unwrap();

        assert_eq!(
            paths["~/*"],
            serde_json::json!(["./app/*", "../../../app/*"])
        );
        assert_eq!(
            paths["#imports"],
            serde_json::json!(["./.nuxt/imports", "../../../.nuxt/imports"])
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_parse_jsonc_value_handles_comments_and_trailing_commas() {
        let value = parse_jsonc_value(
            r#"{
  // comment
  "compilerOptions": {
    "strict": true,
    "paths": {
      "@/*": ["src/*",],
    },
  },
}"#,
        )
        .unwrap();

        assert_eq!(
            value["compilerOptions"]["paths"]["@/*"][0],
            serde_json::Value::String("src/*".into())
        );
    }

    #[test]
    fn test_strip_json_comments_preserves_strings() {
        let stripped = strip_json_comments(r#"{ "url": "https://example.com" }"#);
        insta::assert_snapshot!(stripped.as_str());
    }

    #[test]
    fn test_source_type_for_path() {
        assert_eq!(
            source_type_for_path(Path::new("foo.ts")),
            Some(oxc_span::SourceType::ts())
        );
        assert_eq!(
            source_type_for_path(Path::new("foo.tsx")),
            Some(oxc_span::SourceType::tsx())
        );
        assert_eq!(source_type_for_path(Path::new("foo.vue")), None);
    }
}
