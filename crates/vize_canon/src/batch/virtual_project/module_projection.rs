//! Resolve lowered module identities after the authored graph is registered.

use std::path::Path;

use vize_carton::{FxHashSet, String};

use super::{VirtualFile, VirtualProject};
use crate::batch::import_rewriter::VirtualProjectRewriteOptions;
use crate::{PackageResolutionContext, PackageResolutionMode};

impl VirtualProject {
    /// Module graph edges are resolved from the pre-rewrite projection. A
    /// previous materialized path must never become the next source identity.
    pub(super) fn module_source(&self, file: &VirtualFile) -> Option<&String> {
        self.pre_rewrite_code.get(&file.virtual_path).or_else(|| {
            file.source_map
                .sfc_map
                .is_none()
                .then(|| self.original_contents.get(&file.virtual_path))
                .flatten()
        })
    }

    pub(super) fn finalize_module_imports(&mut self) {
        if self.editor_document_options.is_some() {
            self.finalize_editor_imports();
            return;
        }
        if !self.jsx_typecheck {
            return;
        }

        let aliases = self.dependency_alias_map();
        let settings = self.package_resolution_settings();
        let mut package_resolver = crate::PackageRouteResolver::default();
        let mirrorable = self
            .original_index
            .keys()
            .cloned()
            .collect::<FxHashSet<_>>();
        let rewrites = self
            .virtual_files_sorted()
            .into_iter()
            .filter_map(|file| {
                let source = self.module_source(file)?;
                let context = settings
                    .context(
                        &mut package_resolver,
                        &file.original_path,
                        PackageResolutionMode::Contextual,
                    )
                    .0;
                let resolver = |specifier: &str, mode| {
                    // Native Node ESM still owns invalid extensionless imports.
                    // Projection must not turn an invalid authored edge valid.
                    if requires_explicit_extension(specifier, mode, &context) {
                        return None;
                    }
                    let path = super::dependency_scan::resolve_dependency(
                        specifier,
                        file.original_path.parent()?,
                        &self.project_root,
                        &aliases,
                    )?;
                    let target = self
                        .find_by_original(&vize_carton::path::canonicalize_non_verbatim(&path))?;
                    is_lowered_jsx(target)
                        .then(|| target.virtual_path.to_string_lossy().replace('\\', "/"))
                };
                let options = VirtualProjectRewriteOptions {
                    preserve_relative_declarations: self.is_package_route_path(&file.original_path)
                        || self.session_scripts,
                    mirrorable_project_files: Some(&mirrorable),
                    alias_rewrite_policy: Some(self.alias_rewrite_policy()),
                    module_resolver: Some(&resolver),
                };
                let source_type = super::build::source_type_for_path(&file.virtual_path)?;
                let roots = (self.project_root.as_path(), self.virtual_root.as_path());
                let rewritten = if file.source_map.sfc_map.is_some() {
                    self.rewriter
                        .rewrite_generated_for_virtual_project_with_alias_policy(
                            source,
                            source_type,
                            roots,
                            file.original_path.parent(),
                            options,
                        )
                } else {
                    self.rewriter.rewrite_for_virtual_project_with_policy(
                        source,
                        source_type,
                        roots,
                        file.original_path.parent(),
                        options,
                    )
                };
                Some((file.virtual_path.clone(), rewritten))
            })
            .collect::<Vec<_>>();
        for (path, rewritten) in rewrites {
            if let Some(file) = self.virtual_files.get_mut(&path) {
                if file.content != rewritten.code {
                    self.incremental_materialized_candidates.insert(path);
                }
                file.content = rewritten.code;
                file.source_map.import_map = rewritten.source_map;
            }
        }
    }
}

fn is_lowered_jsx(file: &VirtualFile) -> bool {
    file.source_map.sfc_map.is_some()
        && file
            .original_path
            .extension()
            .is_some_and(|extension| extension == "jsx" || extension == "tsx")
}

fn requires_explicit_extension(
    specifier: &str,
    mode: PackageResolutionMode,
    context: &PackageResolutionContext,
) -> bool {
    let mode = if mode == PackageResolutionMode::Contextual {
        context.mode
    } else {
        mode
    };
    mode == PackageResolutionMode::Import
        && context
            .module_resolution
            .as_deref()
            .is_some_and(|resolution| {
                matches!(resolution, "node16" | "node18" | "node20" | "nodenext")
            })
        && (specifier.starts_with("./") || specifier.starts_with("../"))
        && Path::new(specifier)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_none_or(|extension| {
                !matches!(
                    extension,
                    "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts"
                )
            })
}
