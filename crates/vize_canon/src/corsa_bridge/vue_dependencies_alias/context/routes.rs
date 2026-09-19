//! Package-route recording and bounded dependency-specifier discovery.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String as CompactString};

use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::{
    PackageRouteReachability, package_resolution::PackageResolutionSettings,
};

#[allow(clippy::disallowed_types)] // Compiler-option aliases originate in serde_json maps.
type CompilerAlias = (std::string::String, std::string::String);

type ReachabilityCacheKey = (
    PathBuf,
    PathBuf,
    CompactString,
    crate::PackageResolutionContext,
    u8,
);

#[allow(clippy::disallowed_types)] // Compiler-option aliases originate in serde_json maps.
pub(super) struct RouteDiscovery<'a> {
    settings: &'a PackageResolutionSettings,
    resolver: &'a mut crate::PackageRouteResolver,
    routes: &'a mut FxHashMap<(PathBuf, CompactString), crate::PackageRoute>,
    reachability: &'a mut FxHashMap<ReachabilityCacheKey, PackageRouteReachability>,
    bindings: &'a mut Vec<crate::PackageRouteBinding>,
    inputs: &'a mut Vec<PathBuf>,
    aliases: &'a [CompilerAlias],
}

impl<'a> RouteDiscovery<'a> {
    pub(super) fn new(
        settings: &'a PackageResolutionSettings,
        resolver: &'a mut crate::PackageRouteResolver,
        routes: &'a mut FxHashMap<(PathBuf, CompactString), crate::PackageRoute>,
        reachability: &'a mut FxHashMap<ReachabilityCacheKey, PackageRouteReachability>,
        bindings: &'a mut Vec<crate::PackageRouteBinding>,
        inputs: &'a mut Vec<PathBuf>,
        aliases: &'a [CompilerAlias],
    ) -> Self {
        Self {
            settings,
            resolver,
            routes,
            reachability,
            bindings,
            inputs,
            aliases,
        }
    }

    pub(super) fn resolve(
        &mut self,
        importer: &Path,
        specifier: &str,
        mode: crate::PackageResolutionMode,
    ) -> bool {
        if crate::batch::virtual_project::is_vue_runtime_support_specifier(specifier) {
            return false;
        }
        let importer_dir = importer.parent().unwrap_or(importer);
        let (context, context_inputs) = self.settings.context(self.resolver, importer, mode);
        let lookup = self.resolver.lookup_with_context(
            importer_dir,
            specifier,
            crate::PackageSourceOptions::new(true, true),
            context.clone(),
        );
        let watchable_negative = lookup.is_watchable_negative();
        let (route, consulted) = lookup.into_parts();
        if route.is_some() || watchable_negative {
            self.inputs.extend(consulted.iter().cloned());
            self.inputs
                .extend(self.settings.input_paths().iter().cloned());
            self.inputs.extend(context_inputs.iter().cloned());
        }
        let reachability = route
            .as_ref()
            .map_or_else(PackageRouteReachability::default, |route| {
                let key = (
                    logical_absolute(importer),
                    route.manifest_path.clone(),
                    CompactString::from(specifier),
                    context.clone(),
                    crate::batch::PACKAGE_REACHABILITY_BUDGET_REVISION,
                );
                self.reachability
                    .entry(key)
                    .or_insert_with(|| {
                        let scanned = crate::batch::virtual_project::package_route_reaches_vue(
                            route,
                            self.aliases,
                            self.settings,
                            self.resolver,
                            crate::PackageSourceOptions::new(true, true),
                        );
                        scanned.record_work(self.resolver);
                        scanned
                    })
                    .clone()
            });
        let needs_shadow = reachability.requires_shadow()
            || route
                .as_ref()
                .is_some_and(crate::PackageRoute::requires_workspace_source_shadow);
        let track_reachability = reachability.requires_tracking();
        self.inputs.extend(reachability.inputs);
        if needs_shadow && let Some(route) = route.as_ref() {
            self.routes.insert(
                (logical_absolute(importer_dir), specifier.into()),
                route.clone(),
            );
        }
        if needs_shadow || track_reachability || watchable_negative {
            self.bindings.push(crate::PackageRouteBinding {
                importer_path: importer.to_path_buf(),
                specifier: specifier.into(),
                occurrence_mode: mode,
                context,
                route: needs_shadow.then_some(route).flatten(),
                invalidation_paths: consulted
                    .into_iter()
                    .chain(self.settings.input_paths().iter().cloned())
                    .chain(context_inputs)
                    .chain(std::iter::once(importer.to_path_buf()))
                    .collect(),
            });
        }
        needs_shadow
    }
}

pub(super) fn package_specifiers_from_frontier(
    project: &VirtualProject,
    scanned: &mut vize_carton::FxHashSet<PathBuf>,
) -> Vec<CompactString> {
    let rewriter = crate::batch::ImportRewriter::new();
    let mut specifiers = project
        .virtual_files_sorted()
        .into_iter()
        .filter(|file| scanned.insert(file.original_path.clone()))
        .flat_map(|file| {
            let source_type = if file
                .virtual_path
                .extension()
                .is_some_and(|extension| extension == "tsx")
            {
                oxc_span::SourceType::tsx()
            } else {
                oxc_span::SourceType::ts()
            };
            rewriter.collect_all_specifiers(&file.content, source_type)
        })
        .filter(|specifier| {
            !specifier.starts_with('.') && !Path::new(specifier.as_str()).is_absolute()
        })
        .collect::<Vec<_>>();
    specifiers.sort();
    specifiers.dedup();
    specifiers
}

pub(super) fn logical_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "routes_budget_tests.rs"]
mod budget_tests;
