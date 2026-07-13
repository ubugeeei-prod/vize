//! `VirtualProject` lifecycle: construction, configuration, and file
//! registration. Registration delegates the expensive per-file work to
//! [`super::build`] so it can run in parallel, then absorbs the results into
//! the project's indexes.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use rayon::prelude::*;
use vize_carton::{FxHashMap, ToCompactString, profile};
use vize_relief::TemplateSyntaxMode;

use crate::batch::error::{CorsaError, CorsaResult};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

use super::VirtualProject;
use super::artifact_recipe::build_registered_sources;
use super::artifact_source::RegisteredSource;
use super::build::{RegisteredFile, source_type_for_path};

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
            virtual_ts_options: VirtualTsOptions::default(),
            virtual_ts_check_options: VirtualTsCheckOptions::default(),
            options_api: false,
            legacy_vue2: false,
            jsx_typecheck: false,
            dialect: vize_carton::config::VueVersion::default(),
            template_syntax: TemplateSyntaxMode::default(),
            experimental_in_tag_comments: false,
            virtual_files: FxHashMap::default(),
            passthrough_files: FxHashMap::default(),
            original_index: FxHashMap::default(),
            original_contents: FxHashMap::default(),
            diagnostics: Vec::new(),
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
    }

    pub(crate) fn uses_shared_helpers(&self) -> bool {
        !self.legacy_vue2
            && !matches!(
                self.dialect,
                vize_carton::config::VueVersion::V2 | vize_carton::config::VueVersion::V2_7
            )
    }

    pub(crate) fn set_template_syntax(&mut self, template_syntax: TemplateSyntaxMode) {
        self.template_syntax = template_syntax;
    }

    pub(crate) fn set_experimental_in_tag_comments(&mut self, enabled: bool) {
        self.experimental_in_tag_comments = enabled;
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
        self.register_sources(vec![RegisteredSource {
            path: path.to_path_buf(),
            content: content.to_compact_string(),
            source_type: None,
        }])
    }

    /// Register a batch in one Atlas compilation and query independent sources
    /// through parallel sessions sharing the same immutable artifact cache.
    pub fn register_paths(&mut self, paths: &[PathBuf]) -> CorsaResult<()> {
        let valid_paths: Vec<&Path> = paths
            .iter()
            .filter(|path| path.is_file())
            .map(PathBuf::as_path)
            .collect();
        if valid_paths.is_empty() {
            return Ok(());
        }

        let sources: Result<Vec<RegisteredSource>, CorsaError> = valid_paths
            .par_iter()
            .map(|&path| {
                let content = profile!("canon.file.read", std::fs::read_to_string(path))?;
                Ok(RegisteredSource {
                    path: path.to_path_buf(),
                    content: content.into(),
                    source_type: None,
                })
            })
            .collect();
        self.register_sources(sources?)
    }

    /// Register a `.vue` file.
    pub fn register_vue_file(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        self.register_path_with_content(path, content)
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
        self.register_sources(vec![RegisteredSource {
            path: path.to_path_buf(),
            content: content.to_compact_string(),
            source_type: Some(source_type),
        }])
    }

    fn register_sources(&mut self, sources: Vec<RegisteredSource>) -> CorsaResult<()> {
        self.virtual_files.reserve(sources.len());
        for registered in build_registered_sources(self, sources)? {
            self.absorb_registered_file(registered);
        }
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
        for (virtual_path, original_path) in registered.passthrough_files {
            self.passthrough_files.insert(virtual_path, original_path);
        }
        for file in registered.extra_virtual_files {
            self.virtual_files.insert(file.virtual_path.clone(), file);
        }
        self.virtual_files
            .insert(registered.file.virtual_path.clone(), registered.file);
    }
}
