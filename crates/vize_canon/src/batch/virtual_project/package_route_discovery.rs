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
                if is_vue_runtime_support_specifier(&specifier) {
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
                let reachability =
                    route
                        .as_ref()
                        .map_or_else(PackageRouteReachability::default, |route| {
                            package_route_reaches_vue(
                                route,
                                &aliases,
                                &resolution,
                                &mut resolver,
                                source_options,
                            )
                        });
                let needs_shadow = reachability.reaches_vue;
                if !watchable_negative && !needs_shadow {
                    continue;
                }
                invalidation_paths.extend(reachability.inputs);
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

/// Whether a module specifier belongs to Vue's shared runtime/type support.
///
/// Canon supplies these types to virtual documents. They are terminal support
/// edges rather than importer-scoped component packages, whether the edge came
/// from generated helpers or from a package declaration in their runtime graph.
pub(crate) fn is_vue_runtime_support_specifier(specifier: &str) -> bool {
    specifier == "vue" || specifier.starts_with("@vue/") || specifier == "vite/client"
}

#[cfg(test)]
mod runtime_support_tests {
    use super::is_vue_runtime_support_specifier as is_support;

    #[test]
    fn generated_runtime_support_filter_is_exact() {
        for specifier in [
            "vue",
            "@vue/runtime-core",
            "@vue/runtime-dom",
            "vite/client",
        ] {
            assert!(is_support(specifier), "{specifier}");
        }
        for specifier in [
            "vue-router",
            "@vueuse/core",
            "@vue",
            "vite",
            "vite/plugin-vue",
        ] {
            assert!(!is_support(specifier), "{specifier}");
        }
    }
}

#[allow(clippy::disallowed_types)] // Compiler-option aliases originate in serde_json maps.
#[derive(Clone, Debug, Default)]
pub(crate) struct PackageRouteReachability {
    pub(crate) reaches_vue: bool,
    pub(crate) inputs: Vec<PathBuf>,
}

#[allow(clippy::disallowed_types)] // Compiler-option aliases originate in serde_json maps.
pub(crate) fn package_route_reaches_vue(
    route: &crate::PackageRoute,
    aliases: &[(std::string::String, std::string::String)],
    resolution: &super::package_resolution::PackageResolutionSettings,
    resolver: &mut crate::PackageRouteResolver,
    source_options: crate::PackageSourceOptions,
) -> PackageRouteReachability {
    let mut queued = FxHashSet::default();
    let mut queue = route
        .all_source_paths()
        .into_iter()
        .filter(|path| queued.insert((*path).clone()))
        .cloned()
        .collect::<Vec<_>>();
    let mut inputs = Vec::new();
    let rewriter = crate::batch::ImportRewriter::new();
    while let Some(path) = queue.pop() {
        inputs.push(path.clone());
        if path.extension().is_some_and(|extension| extension == "vue") {
            return reachability(true, inputs);
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
                if queued.insert(dependency.clone()) {
                    queue.push(dependency);
                }
                continue;
            }
            if specifier.starts_with('.') || Path::new(specifier.as_str()).is_absolute() {
                continue;
            }
            // Vue's compiler/runtime type packages are supplied by the virtual
            // project itself. They are terminal support edges, not
            // importer-scoped component packages: descending through their
            // transitive compiler graph can walk the whole installed toolchain
            // and can misclassify an internal fixture SFC as the caller's
            // package identity. Resolve user aliases above before applying
            // this native-package boundary.
            if is_vue_runtime_support_specifier(&specifier) {
                continue;
            }
            let (context, context_inputs) = resolution.context(resolver, &path, mode);
            let (nested, consulted) = resolver
                .lookup_with_context(importer_dir, &specifier, source_options, context)
                .into_parts();
            inputs.extend(context_inputs);
            inputs.extend(consulted);
            if let Some(nested) = nested {
                queue.extend(
                    nested
                        .all_source_paths()
                        .into_iter()
                        .filter(|source| queued.insert((*source).clone()))
                        .cloned(),
                );
            }
        }
    }
    reachability(false, inputs)
}

fn reachability(reaches_vue: bool, mut inputs: Vec<PathBuf>) -> PackageRouteReachability {
    inputs.sort();
    inputs.dedup();
    PackageRouteReachability {
        reaches_vue,
        inputs,
    }
}
