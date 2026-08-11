//! Reconcile importer-scoped package routes after persistent source edits.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::FxHashSet;

use super::VirtualProject;
use super::dependency_scan::resolve_dependency;

impl VirtualProject {
    pub(crate) fn reconcile_package_routes_for_importers(&mut self, changed: &[PathBuf]) {
        let changed = changed
            .iter()
            .map(|path| {
                let absolute = if path.is_absolute() {
                    path.clone()
                } else {
                    self.project_root.join(path)
                };
                vize_carton::path::canonicalize_non_verbatim(&absolute)
            })
            .collect::<FxHashSet<_>>();
        if changed.is_empty() {
            return;
        }
        let removed_keys = changed
            .iter()
            .flat_map(|importer| {
                self.package_route_importers
                    .get(importer)
                    .into_iter()
                    .flatten()
                    .cloned()
            })
            .collect::<Vec<_>>();
        let previous = removed_keys.len();
        for key in removed_keys {
            self.remove_package_route_binding(&key);
        }

        let importers = changed
            .iter()
            .filter_map(|path| {
                let file = self.find_by_original(path)?;
                let source_type = if file
                    .virtual_path
                    .extension()
                    .is_some_and(|extension| extension == "tsx")
                {
                    SourceType::tsx()
                } else {
                    SourceType::ts()
                };
                Some((path.clone(), file.content.clone(), source_type))
            })
            .collect::<Vec<_>>();
        let aliases = self.dependency_alias_map();
        let resolution = self.package_resolution_settings();
        let source_options = crate::PackageSourceOptions::new(
            self.source_policy.allows_javascript(),
            self.jsx_typecheck,
        );
        let mut discovered = Vec::new();
        let mut resolver = self
            .package_route_resolver
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (importer, content, source_type) in importers {
            let importer_dir = importer.parent().unwrap_or(importer.as_path());
            for (specifier, occurrence_mode) in self
                .rewriter
                .collect_all_specifier_occurrences(&content, source_type)
            {
                // Canon's generated Vue modules reference these runtime types;
                // they are supplied by the virtual project itself and are not
                // authored importer-local package routes. Treating their
                // absence beside an external workspace source as a watchable
                // cold package would grow one persistent route per SFC.
                if specifier == "vue"
                    || specifier.starts_with("@vue/")
                    || specifier == "vite/client"
                {
                    continue;
                }
                if specifier.starts_with('.')
                    || Path::new(specifier.as_str()).is_absolute()
                    || resolve_dependency(&specifier, importer_dir, &self.project_root, &aliases)
                        .is_some()
                {
                    continue;
                }
                let (context, context_inputs) =
                    resolution.context(&mut resolver, &importer, occurrence_mode);
                let lookup = resolver.lookup_with_context(
                    importer_dir,
                    &specifier,
                    source_options,
                    context.clone(),
                );
                let watchable_negative = lookup.is_watchable_negative();
                let (route, mut invalidation_paths) = lookup.into_parts();
                let needs_shadow = route.as_ref().is_some_and(|route| {
                    package_route_reaches_vue(
                        route,
                        &aliases,
                        &resolution,
                        &mut resolver,
                        source_options,
                    )
                });
                if !watchable_negative && !needs_shadow {
                    continue;
                }
                invalidation_paths.extend(resolution.input_paths().iter().cloned());
                invalidation_paths.extend(context_inputs);
                invalidation_paths.push(importer.clone());
                invalidation_paths.sort();
                invalidation_paths.dedup();
                discovered.push(crate::PackageRouteBinding {
                    importer_path: importer.clone(),
                    specifier,
                    occurrence_mode,
                    context,
                    route,
                    invalidation_paths,
                });
            }
        }
        drop(resolver);
        let discovered_count = discovered.len();
        for binding in discovered {
            self.insert_package_route_binding(binding);
        }
        self.package_route_refresh_keys
            .retain(|key| !changed.contains(&key.importer_path));
        self.package_routes_need_refresh = !self.package_route_refresh_keys.is_empty();
        let reconciled = previous.max(discovered_count);
        if reconciled > 0
            && let Ok(mut resolver) = self.package_route_resolver.lock()
        {
            resolver.record_refresh_scope(self.package_routes.len(), reconciled, reconciled);
        }
    }
}

#[allow(clippy::disallowed_types)] // Compiler-option aliases originate in serde_json maps.
pub(crate) fn package_route_reaches_vue(
    route: &crate::PackageRoute,
    aliases: &[(std::string::String, std::string::String)],
    resolution: &super::package_resolution::PackageResolutionSettings,
    resolver: &mut crate::PackageRouteResolver,
    source_options: crate::PackageSourceOptions,
) -> bool {
    let mut queue = route
        .all_source_paths()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut visited = FxHashSet::default();
    let rewriter = crate::batch::ImportRewriter::new();
    while let Some(path) = queue.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "vue") {
            return true;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let source_type = if path.extension().is_some_and(|extension| extension == "tsx") {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        let Some(importer_dir) = path.parent() else {
            continue;
        };
        for (specifier, mode) in rewriter.collect_all_specifier_occurrences(&content, source_type) {
            if let Some(dependency) =
                resolve_dependency(&specifier, importer_dir, &route.package_root, aliases)
            {
                if !visited.contains(&dependency) {
                    queue.push(dependency);
                }
                continue;
            }
            if specifier.starts_with('.') || Path::new(specifier.as_str()).is_absolute() {
                continue;
            }
            let (context, _) = resolution.context(resolver, &path, mode);
            let nested = resolver
                .lookup_with_context(importer_dir, &specifier, source_options, context)
                .into_parts()
                .0;
            if let Some(nested) = nested {
                queue.extend(nested.all_source_paths().into_iter().cloned());
            }
        }
    }
    false
}
