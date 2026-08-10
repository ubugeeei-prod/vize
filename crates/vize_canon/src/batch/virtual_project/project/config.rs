//! `VirtualProject` configuration: generation options, declaration roots, and
//! virtual module aliases carried into virtual-TS generation.

use std::path::{Path, PathBuf};

use vize_atelier_core::TemplateSyntaxMode;

use crate::batch::error::CorsaResult;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

use super::super::VirtualProject;

impl VirtualProject {
    /// Set the tsconfig path to extend.
    pub fn set_tsconfig_path(&mut self, tsconfig_path: Option<PathBuf>) {
        self.tsconfig_path = tsconfig_path.map(vize_carton::path::normalize_windows_verbatim_path);
        self.preserve_unused_diagnostics = self.resolve_tsconfig_preserves_unused_diagnostics();
        self.check_js = self.resolve_tsconfig_checks_javascript();
    }

    /// Whether TypeScript diagnostics landing in `virtual_path` must be dropped
    /// because the file is a JavaScript SFC and the project does not enable
    /// `checkJs` (#3322).
    pub(crate) fn skips_typescript_diagnostics(&self, virtual_path: &Path) -> bool {
        !self.check_js && self.unchecked_javascript_files.contains(virtual_path)
    }

    /// Set the shared virtual TS options.
    pub fn set_virtual_ts_options(&mut self, options: VirtualTsOptions) {
        self.virtual_ts_options = options;
    }

    pub(crate) fn set_virtual_ts_check_options(&mut self, options: VirtualTsCheckOptions) {
        self.virtual_ts_check_options = options;
    }

    pub(crate) fn set_virtual_module_aliases(
        &mut self,
        aliases: impl IntoIterator<Item = (vize_carton::String, PathBuf)>,
    ) {
        self.virtual_module_aliases.clear();
        for (specifier, source_path) in aliases {
            let source_path = vize_carton::path::canonicalize_non_verbatim(&source_path);
            let targets = self.virtual_module_aliases.entry(specifier).or_default();
            if !targets.contains(&source_path) {
                targets.push(source_path);
            }
        }
        for targets in self.virtual_module_aliases.values_mut() {
            targets.sort();
        }
    }

    pub(crate) fn set_declaration_roots(&mut self, paths: &[PathBuf]) {
        self.declaration_roots = Some(
            paths
                .iter()
                .filter(|path| path.is_file())
                .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
                .collect(),
        );
    }

    pub(crate) fn is_declaration_root(&self, original_path: &Path) -> bool {
        self.declaration_roots
            .as_ref()
            .is_none_or(|roots| roots.contains(original_path))
    }

    pub(crate) fn register_virtual_module_alias_targets(&mut self) -> CorsaResult<()> {
        let mut targets: Vec<PathBuf> = self
            .virtual_module_aliases
            .values()
            .flatten()
            .cloned()
            .collect();
        targets.sort();
        targets.dedup();
        self.register_paths(&targets)
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
}
