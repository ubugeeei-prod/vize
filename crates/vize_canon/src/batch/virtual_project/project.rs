//! `VirtualProject` lifecycle: construction, configuration, and file
//! registration. Registration delegates the expensive per-file work to
//! [`super::build`] so it can run in parallel, then absorbs the results into
//! the project's indexes.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use rayon::prelude::*;
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{FxHashMap, FxHashSet, profile};

use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::ImportRewriter;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

mod config;

use super::VirtualProject;
use super::build::{
    RegisteredFile, VirtualBuildContext, build_registered_file, build_script_registered_file,
    build_vue_registered_file, source_type_for_path,
};

const MUSEA_DEFINE_ART_STUB: &str =
    "declare function defineArt(source: string, options?: Record<string, any>): void;";
fn is_musea_art_vue_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".art.vue"))
}

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
            check_js: false,
            virtual_ts_options: VirtualTsOptions::default(),
            diagnostic_paths: FxHashSet::default(),
            declaration_roots: None,
            virtual_ts_check_options: VirtualTsCheckOptions::default(),
            virtual_module_aliases: FxHashMap::default(),
            options_api: false,
            session_scripts: false,
            legacy_vue2: false,
            jsx_typecheck: false,
            dialect: vize_carton::config::VueVersion::default(),
            template_syntax: TemplateSyntaxMode::default(),
            experimental_in_tag_comments: false,
            virtual_files: FxHashMap::default(),
            passthrough_files: FxHashMap::default(),
            original_index: FxHashMap::default(),
            original_contents: FxHashMap::default(),
            unchecked_javascript_files: FxHashSet::default(),
            diagnostics: Vec::new(),
            rewriter: ImportRewriter::new(),
        };
        project.preserve_unused_diagnostics =
            project.resolve_tsconfig_preserves_unused_diagnostics();
        project.check_js = project.resolve_tsconfig_checks_javascript();
        Ok(project)
    }

    /// Create an empty project snapshot with the same generation settings:
    /// correctness-safe refreshes register source files again from disk while
    /// every caller-configured dialect and virtual TS option stays identical.
    pub(crate) fn empty_with_same_options(&self) -> CorsaResult<Self> {
        let mut project = Self::new(&self.project_root)?;
        project.set_tsconfig_path(self.tsconfig_path.clone());
        project.virtual_ts_options = self.virtual_ts_options.clone();
        project.diagnostic_paths = self.diagnostic_paths.clone();
        project.declaration_roots = self.declaration_roots.clone();
        project.virtual_ts_check_options = self.virtual_ts_check_options;
        project.virtual_module_aliases = self.virtual_module_aliases.clone();
        project.options_api = self.options_api;
        project.legacy_vue2 = self.legacy_vue2;
        project.jsx_typecheck = self.jsx_typecheck;
        project.dialect = self.dialect;
        project.template_syntax = self.template_syntax;
        project.experimental_in_tag_comments = self.experimental_in_tag_comments;
        Ok(project)
    }

    /// Register a supported file path.
    pub(super) fn rewriter(&self) -> &crate::batch::import_rewriter::ImportRewriter {
        &self.rewriter
    }

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
                jsx_typecheck: self.jsx_typecheck,
                dialect: self.dialect,
                template_syntax: self.template_syntax,
                experimental_in_tag_comments: self.experimental_in_tag_comments,
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
            jsx_typecheck: self.jsx_typecheck,
            dialect: self.dialect,
            template_syntax: self.template_syntax,
            experimental_in_tag_comments: self.experimental_in_tag_comments,
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
                jsx_typecheck: self.jsx_typecheck,
                dialect: self.dialect,
                template_syntax: self.template_syntax,
                experimental_in_tag_comments: self.experimental_in_tag_comments,
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
            (&self.project_root, &self.virtual_root),
            &self.rewriter,
        )?;
        self.absorb_registered_file(registered);
        Ok(())
    }

    fn absorb_registered_file(&mut self, registered: RegisteredFile) {
        if is_musea_art_vue_path(&registered.file.original_path)
            && !self
                .virtual_ts_options
                .auto_import_stubs
                .iter()
                .any(|stub| stub.contains("defineArt"))
        {
            self.virtual_ts_options
                .auto_import_stubs
                .push(MUSEA_DEFINE_ART_STUB.into());
        }
        self.diagnostics.extend(registered.diagnostics);
        self.original_index.insert(
            registered.file.original_path.clone(),
            registered.file.virtual_path.clone(),
        );
        self.original_contents.insert(
            registered.file.virtual_path.clone(),
            registered.original_content,
        );
        // Re-registration must refresh the classification, not accumulate it: a
        // script block converted to TypeScript (or opted in with `// @ts-check`)
        // would otherwise stay gated behind a stale entry.
        self.unchecked_javascript_files
            .remove(&registered.file.virtual_path);
        if registered.unchecked_javascript {
            self.unchecked_javascript_files
                .insert(registered.file.virtual_path.clone());
        }
        for (virtual_path, original_path) in registered.passthrough_files {
            if !self.virtual_files.contains_key(&virtual_path) {
                self.passthrough_files.insert(virtual_path, original_path);
            }
        }
        for file in registered.extra_virtual_files {
            self.passthrough_files.remove(&file.virtual_path);
            self.virtual_files.insert(file.virtual_path.clone(), file);
        }
        self.passthrough_files.remove(&registered.file.virtual_path);
        self.virtual_files
            .insert(registered.file.virtual_path.clone(), registered.file);
    }
}
