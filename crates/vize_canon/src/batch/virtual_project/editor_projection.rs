//! One generated document, including source maps, for both native editor
//! overlays and the materialized project. Mixing two producers under one URI
//! makes references query offsets from a different program than diagnostics.

use super::{VirtualProject, VueDocumentVirtualTs, VueDocumentVirtualTsOptions};
use std::path::{Path, PathBuf};

impl VirtualProject {
    /// Keep reachable implementation files in one stable configured project.
    /// Declarations and package internals stay inferred through their imports.
    pub(crate) fn editor_query_paths(&self, host: &Path) -> Vec<PathBuf> {
        let mut paths = self
            .registered_original_paths_sorted()
            .into_iter()
            .filter(|source| {
                !crate::batch::declaration_path::is_declaration_file(source)
                    && !source
                        .components()
                        .any(|part| part.as_os_str() == "node_modules")
            })
            .filter_map(|source| self.preferred_materialized_path_for_original(&source))
            .collect::<Vec<_>>();
        paths.extend(self.preferred_materialized_path_for_original(host));
        paths.sort();
        paths.dedup();
        paths
    }

    /// Include failed probes: creating a previously missing dependency, or a
    /// higher-priority TypeScript companion, changes native module identity.
    pub(crate) fn editor_resolution_inputs(&self) -> Vec<PathBuf> {
        let aliases = self.dependency_alias_map();
        let mut inputs = Vec::new();
        for file in self.virtual_files_sorted() {
            let Some(source) = self
                .pre_rewrite_code
                .get(&file.virtual_path)
                .or_else(|| self.original_contents.get(&file.virtual_path))
            else {
                continue;
            };
            let Some(directory) = file.original_path.parent() else {
                continue;
            };
            let Some(source_type) = super::build::source_type_for_path(&file.virtual_path) else {
                continue;
            };
            for (specifier, _) in self
                .rewriter
                .collect_all_specifier_occurrences(source, source_type)
            {
                inputs.extend(
                    super::dependency_scan::resolve_dependency_with_inputs(
                        &specifier,
                        directory,
                        &self.project_root,
                        &aliases,
                    )
                    .1,
                );
            }
        }
        inputs.sort();
        inputs.dedup();
        inputs
    }

    pub(crate) fn editor_script_document(
        &self,
        source: &Path,
    ) -> Option<(PathBuf, crate::batch::RewriteResult)> {
        let source = vize_carton::path::canonicalize_non_verbatim(source);
        let file = self.find_by_original(&source)?;
        if file
            .original_path
            .extension()
            .is_some_and(|extension| extension == "vue")
        {
            return None;
        }
        Some((
            self.preferred_materialized_path_for_original(&source)?,
            crate::batch::RewriteResult {
                code: file.content.clone(),
                source_map: file.source_map.import_map.clone(),
            },
        ))
    }

    pub(crate) fn set_editor_document_options(&mut self, options: VueDocumentVirtualTsOptions) {
        self.editor_document_options = Some(options);
    }

    /// Resolve relative and alias edges after the complete reachable graph is
    /// registered. Out-of-root modules have private mirror identities, so an
    /// authored `../shared` cannot be left relative to the generated host.
    pub(crate) fn finalize_editor_imports(&mut self) {
        if self.editor_document_options.is_none() {
            return;
        }
        let aliases = self.dependency_alias_map();
        let rewrites = self
            .virtual_files_sorted()
            .into_iter()
            .filter_map(|file| {
                let source = self.module_source(file)?;
                let resolver = |specifier: &str, _| {
                    let path = super::dependency_scan::resolve_dependency(
                        specifier,
                        file.original_path.parent()?,
                        &self.project_root,
                        &aliases,
                    )?;
                    let target = self
                        .find_by_original(&vize_carton::path::canonicalize_non_verbatim(&path))?;
                    let path = if crate::batch::declaration_path::is_declaration_file(
                        &target.virtual_path,
                    ) {
                        crate::batch::declaration_module_path(&target.virtual_path)
                    } else {
                        target.virtual_path.clone()
                    };
                    Some(path.to_string_lossy().replace('\\', "/"))
                };
                let source_type = super::build::source_type_for_path(&file.virtual_path)?;
                let rewritten = self
                    .rewriter
                    .rewrite_with_alias_resolver_and_missing_vue_policy(
                        source,
                        source_type,
                        file.original_path.parent(),
                        &resolver,
                        true,
                    );
                Some((file.virtual_path.clone(), rewritten))
            })
            .collect::<Vec<_>>();
        for (path, rewritten) in rewrites {
            if let Some(file) = self.virtual_files.get_mut(&path) {
                file.content = rewritten.code;
                file.source_map.import_map = rewritten.source_map;
            }
        }
    }

    pub(crate) fn editor_vue_document(
        &self,
        source: &Path,
    ) -> Option<(PathBuf, VueDocumentVirtualTs)> {
        let source = vize_carton::path::canonicalize_non_verbatim(source);
        let file = self.find_by_original(&source)?;
        let pre_rewrite_code = self.pre_rewrite_code.get(&file.virtual_path)?.clone();
        let path = self.preferred_materialized_path_for_original(&source)?;
        self.find_by_diagnostic_virtual(&path)?;
        let map = file.source_map.sfc_map.as_ref()?;
        let tsx = file
            .virtual_path
            .extension()
            .is_some_and(|extension| extension == "tsx");
        Some((
            path,
            VueDocumentVirtualTs {
                code: file.content.clone(),
                pre_rewrite_code,
                mapping: map.projection().clone(),
                import_source_map: file.source_map.import_map.clone(),
                source_type: if tsx {
                    oxc_span::SourceType::tsx()
                } else {
                    oxc_span::SourceType::ts()
                },
                virtual_suffix: if tsx { ".tsx" } else { ".ts" },
            },
        ))
    }
}
