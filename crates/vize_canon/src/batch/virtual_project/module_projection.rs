//! Resolve lowered module identities after the authored graph is registered.

use std::path::Path;

use rayon::prelude::*;
use vize_carton::{FxHashSet, String};

use super::{VirtualFile, VirtualProject};
use crate::batch::import_rewriter::{VirtualProjectRewriteOptions, apply_captured_vue_stars};
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
        if !self.jsx_typecheck && !self.virtual_files.values().any(is_denied_javascript_sfc) {
            self.finalize_declaration_imports();
            return;
        }

        let aliases = self.dependency_alias_map();
        let settings = self.package_resolution_settings();
        let mirrorable = self
            .original_index
            .keys()
            .cloned()
            .collect::<FxHashSet<_>>();
        // Files rewrite independently; each worker keeps its own manifest cache.
        let rewrites = self
            .virtual_files_sorted()
            .into_par_iter()
            .map_init(
                crate::PackageRouteResolver::default,
                |package_resolver, file| {
                    let source = self.module_source(file)?;
                    let context = settings
                        .context(
                            package_resolver,
                            &file.original_path,
                            PackageResolutionMode::Contextual,
                        )
                        .0;
                    let resolver = |specifier: &str, mode| {
                        let path = super::dependency_scan::resolve_dependency(
                            specifier,
                            file.original_path.parent()?,
                            &self.project_root,
                            &aliases,
                        );
                        let target = path.as_ref().and_then(|path| {
                            self.find_by_original(&vize_carton::path::canonicalize_non_verbatim(
                                path,
                            ))
                        });
                        if target.is_some_and(is_denied_javascript_sfc) {
                            // The project owns this edge through its authored declaration;
                            // a generated TypeScript projection cannot admit a denied JS module.
                            return Some(specifier.to_owned());
                        }
                        // Native Node ESM still owns invalid extensionless imports.
                        // Projection must not turn an invalid authored edge valid.
                        if requires_explicit_extension(specifier, mode, &context) {
                            return None;
                        }
                        let target = target?;
                        is_lowered_jsx(target)
                            .then(|| target.virtual_path.to_string_lossy().replace('\\', "/"))
                    };
                    let options = VirtualProjectRewriteOptions {
                        preserve_relative_declarations: self
                            .is_package_route_path(&file.original_path)
                            || self.session_scripts,
                        mirrorable_project_files: Some(&mirrorable),
                        alias_rewrite_policy: Some(self.alias_rewrite_policy()),
                        module_resolver: Some(&resolver),
                    };
                    let source_type = super::build::source_type_for_path(&file.virtual_path)?;
                    let roots = (self.project_root.as_path(), self.virtual_root.as_path());
                    let (rewritten, stars) = if file.source_map.sfc_map.is_some() {
                        self.rewriter
                            .rewrite_generated_for_virtual_project_with_captured_stars(
                                source,
                                source_type,
                                roots,
                                file.original_path.parent(),
                                options,
                            )
                    } else {
                        self.rewriter
                            .rewrite_for_virtual_project_with_captured_stars(
                                source,
                                source_type,
                                roots,
                                file.original_path.parent(),
                                options,
                            )
                    };
                    let rewritten = apply_captured_vue_stars(
                        &rewritten.code,
                        &rewritten.source_map,
                        &stars,
                        |specifier, projected, _mode| {
                            self.known_vue_star_target(file, specifier, projected)
                        },
                    );
                    Some((file.virtual_path.clone(), rewritten))
                },
            )
            .collect::<Vec<_>>();
        for (path, rewritten) in rewrites.into_iter().flatten() {
            if let Some(file) = self.virtual_files.get_mut(&path) {
                if file.content != rewritten.code {
                    self.incremental_materialized_candidates.insert(path);
                }
                file.content = rewritten.code;
                file.source_map.import_map = rewritten.source_map;
            }
        }
    }

    /// A declaration can be registered before a referenced declaration has
    /// entered the mirror. Resolve its imports again once the graph is complete
    /// so an authored path and its mirrored path cannot load the same ambient
    /// module twice.
    fn finalize_declaration_imports(&mut self) {
        let mirrorable = self
            .original_index
            .keys()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .collect::<FxHashSet<_>>();
        let rewrites = self
            .virtual_files_sorted()
            .into_iter()
            .filter(|file| {
                self.source_artifacts
                    .get(&file.original_path)
                    .is_some_and(|owner| owner.star_projection.is_some())
                    || file
                        .original_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| {
                            name.ends_with(".d.ts")
                                || name.ends_with(".d.mts")
                                || name.ends_with(".d.cts")
                        })
            })
            .filter_map(|file| {
                let declaration =
                    crate::batch::declaration_path::is_declaration_file(&file.original_path);
                if !declaration {
                    let base = self
                        .source_artifacts
                        .get(&file.original_path)?
                        .star_projection
                        .as_ref()?;
                    let rewritten = apply_captured_vue_stars(
                        &base.code,
                        &base.source_map,
                        &base.stars,
                        |specifier, projected, _mode| {
                            self.known_vue_star_target(file, specifier, projected)
                        },
                    );
                    return Some((file.virtual_path.clone(), rewritten));
                }
                let source = self.original_contents.get(&file.virtual_path)?;
                if !crate::batch::import_rewriter::source_may_contain_relative_specifier(source) {
                    return None;
                }
                let source_type = super::build::source_type_for_path(&file.virtual_path)?;
                let roots = (self.project_root.as_path(), self.virtual_root.as_path());
                let (rewritten, stars) = self
                    .rewriter
                    .rewrite_for_virtual_project_with_captured_stars(
                        source,
                        source_type,
                        roots,
                        file.original_path.parent(),
                        VirtualProjectRewriteOptions {
                            preserve_relative_declarations: self
                                .is_package_route_path(&file.original_path)
                                || self.session_scripts,
                            mirrorable_project_files: Some(&mirrorable),
                            alias_rewrite_policy: Some(self.alias_rewrite_policy()),
                            module_resolver: None,
                        },
                    );
                let rewritten = apply_captured_vue_stars(
                    &rewritten.code,
                    &rewritten.source_map,
                    &stars,
                    |specifier, projected, _mode| {
                        self.known_vue_star_target(file, specifier, projected)
                    },
                );
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

fn is_denied_javascript_sfc(file: &VirtualFile) -> bool {
    file.source_map.sfc_map.is_some()
        && file
            .original_path
            .extension()
            .is_some_and(|extension| extension == "vue")
        && is_javascript_projection_path(&file.virtual_path)
}

pub(super) fn is_javascript_projection_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".vue.__vize_javascript__.ts")
                || name.ends_with(".vue.__vize_javascript__.tsx")
        })
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
