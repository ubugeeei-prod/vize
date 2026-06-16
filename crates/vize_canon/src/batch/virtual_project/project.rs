//! `VirtualProject` lifecycle: construction, configuration, and file
//! registration. Registration delegates the expensive per-file work to
//! [`super::build`] so it can run in parallel, then absorbs the results into
//! the project's indexes.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use rayon::prelude::*;
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{FxHashMap, profile};

use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::ImportRewriter;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions, ref_unwrap_helper_for};

use super::VirtualProject;
use super::build::{
    RegisteredFile, VirtualBuildContext, build_registered_file, build_script_registered_file,
    build_vue_registered_file, source_type_for_path,
};

impl VirtualProject {
    /// Create a new virtual project.
    pub fn new(project_root: &Path) -> CorsaResult<Self> {
        let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
        let virtual_root = project_root
            .join("node_modules")
            .join(".vize")
            .join("canon");

        let mut project = Self {
            project_root,
            virtual_root,
            tsconfig_path: None,
            preserve_unused_diagnostics: false,
            virtual_ts_options: VirtualTsOptions::default(),
            virtual_ts_check_options: VirtualTsCheckOptions::default(),
            options_api: false,
            legacy_vue2: false,
            jsx_typecheck: false,
            dialect: vize_carton::config::VueVersion::default(),
            ref_unwrap_helper: ref_unwrap_helper_for(
                false,
                vize_carton::config::VueVersion::default(),
            ),
            template_syntax: TemplateSyntaxMode::default(),
            virtual_files: FxHashMap::default(),
            passthrough_files: FxHashMap::default(),
            original_index: FxHashMap::default(),
            original_contents: FxHashMap::default(),
            diagnostics: Vec::new(),
            rewriter: ImportRewriter::new(),
        };
        project.preserve_unused_diagnostics =
            project.resolve_tsconfig_preserves_unused_diagnostics();
        Ok(project)
    }

    /// Set the tsconfig path to extend.
    pub fn set_tsconfig_path(&mut self, tsconfig_path: Option<PathBuf>) {
        self.tsconfig_path = tsconfig_path.map(vize_carton::path::normalize_windows_verbatim_path);
        self.preserve_unused_diagnostics = self.resolve_tsconfig_preserves_unused_diagnostics();
    }

    /// Set the shared virtual TS options.
    pub fn set_virtual_ts_options(&mut self, options: VirtualTsOptions) {
        self.virtual_ts_options = options;
    }

    pub(crate) fn set_virtual_ts_check_options(&mut self, options: VirtualTsCheckOptions) {
        self.virtual_ts_check_options = options;
    }

    pub(crate) fn set_options_api(&mut self, enabled: bool) {
        self.options_api = enabled;
    }

    pub(crate) fn set_legacy_vue2(&mut self, enabled: bool) {
        self.legacy_vue2 = enabled;
        self.sync_ref_unwrap_helper();
    }

    /// Enable opt-in type-checking of `.jsx`/`.tsx` Vue components (#1497).
    pub(crate) fn set_jsx_typecheck(&mut self, enabled: bool) {
        self.jsx_typecheck = enabled;
    }

    /// Set the configured Vue dialect (default [`VueVersion::V3`]).
    ///
    /// Carried into virtual-TS generation for dialect-aware instance and helper
    /// typing while keeping default-V3 output stable.
    pub(crate) fn set_dialect(&mut self, dialect: vize_carton::config::VueVersion) {
        self.dialect = dialect;
        self.sync_ref_unwrap_helper();
    }

    pub(crate) fn set_template_syntax(&mut self, template_syntax: TemplateSyntaxMode) {
        self.template_syntax = template_syntax;
    }

    fn sync_ref_unwrap_helper(&mut self) {
        self.ref_unwrap_helper = ref_unwrap_helper_for(self.legacy_vue2, self.dialect);
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
                preserve_unused_diagnostics: self.tsconfig_preserves_unused_diagnostics(),
                options_api: self.options_api,
                legacy_vue2: self.legacy_vue2,
                ref_unwrap_helper: self.ref_unwrap_helper,
                jsx_typecheck: self.jsx_typecheck,
                dialect: self.dialect,
                template_syntax: self.template_syntax,
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

        let preserve_unused_diagnostics = self.tsconfig_preserves_unused_diagnostics();
        let build_context = VirtualBuildContext {
            project_root: self.project_root.as_path(),
            virtual_root: self.virtual_root.as_path(),
            virtual_ts_options: &self.virtual_ts_options,
            virtual_ts_check_options: self.virtual_ts_check_options,
            preserve_unused_diagnostics,
            options_api: self.options_api,
            legacy_vue2: self.legacy_vue2,
            ref_unwrap_helper: self.ref_unwrap_helper,
            jsx_typecheck: self.jsx_typecheck,
            dialect: self.dialect,
            template_syntax: self.template_syntax,
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
                preserve_unused_diagnostics: self.tsconfig_preserves_unused_diagnostics(),
                options_api: self.options_api,
                legacy_vue2: self.legacy_vue2,
                ref_unwrap_helper: self.ref_unwrap_helper,
                jsx_typecheck: self.jsx_typecheck,
                dialect: self.dialect,
                template_syntax: self.template_syntax,
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
}
