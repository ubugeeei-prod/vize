//! Reconcile importer-scoped package routes after persistent source edits.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::FxHashSet;

use super::VirtualProject;
use super::dependency_scan::resolve_dependency;
use super::package_route_reachability::{PackageRouteReachability, package_route_reaches_vue};

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
                let has_route = route.is_some();
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
                if has_route {
                    reachability.record_work(&mut resolver);
                }
                let needs_shadow = reachability.requires_shadow()
                    || route
                        .as_ref()
                        .is_some_and(crate::PackageRoute::requires_workspace_source_shadow);
                if !route_requires_invalidation_binding(has_route, watchable_negative) {
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
                    route: needs_shadow.then_some(route).flatten(),
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

fn route_requires_invalidation_binding(has_route: bool, watchable_negative: bool) -> bool {
    has_route || watchable_negative
}

/// Whether a module specifier belongs to Vue's shared runtime/type support.
///
/// Canon supplies these types to virtual documents. They are terminal support
/// edges rather than importer-scoped component packages, whether the edge came
/// from generated helpers, a package declaration, or a `vue/...` subpath
/// (Nuxt SSR's `vue/dist/vue.cjs.js`). `vue-router` is not a subpath.
pub fn is_vue_runtime_support_specifier(specifier: &str) -> bool {
    specifier == "vue"
        || specifier.starts_with("vue/")
        || specifier.starts_with("@vue/")
        || specifier == "vite/client"
}

#[cfg(test)]
mod runtime_support_tests {
    use super::{
        is_vue_runtime_support_specifier as is_support, route_requires_invalidation_binding,
    };

    #[test]
    fn generated_runtime_support_filter_is_exact() {
        for specifier in [
            "vue",
            "vue/dist/vue.cjs.js",
            "vue/server-renderer",
            "vue/jsx-runtime",
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

    #[test]
    fn positive_non_vue_routes_keep_an_invalidation_binding() {
        assert!(route_requires_invalidation_binding(true, false));
        assert!(route_requires_invalidation_binding(false, true));
        assert!(!route_requires_invalidation_binding(false, false));
    }
}
