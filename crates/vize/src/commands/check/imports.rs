//! Transitive resolution of source imports for `vize check` virtual projects.
//!
//! A check run may intentionally report diagnostics for only a subset of
//! sources. This module separates authored files that TypeScript can resolve
//! in place from files that must enter Vize's mirror for Vue import rewriting.

use std::path::{Path, PathBuf};

use vize_canon::batch::is_vue_runtime_support_specifier;
use vize_canon::{
    PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding,
    PackageRouteResolver, PackageSourceOptions,
};
#[cfg(test)]
use vize_l0::cstr;
use vize_l0::{FxHashMap, FxHashSet, String};

use super::imports_aliases::PathAliasResolver;
use super::imports_package_routes::sort_package_route_bindings;
use super::path_cache::CanonicalPathCache;

mod cache;
mod deferred;
mod registration;
use registration::non_relative_import_needs_virtual_registration;
mod resolution;
use resolution::{absolutize, is_declaration_file, is_node_modules_path, resolve_relative_import};
pub(super) use resolution::{resolve_import_base, resolve_import_base_with_inputs};
mod parallel;
mod specifiers;
use specifiers::{extract_module_specifier_occurrences, is_relative_specifier};
mod session;
pub(super) use session::LocalImportSession;

/// Source extensions whose imports carry TypeScript types worth pulling into the
/// virtual project, in module-resolution precedence order.
///
/// `.d.ts` is deliberately excluded: ambient declaration files (e.g. a project
/// `shims.d.ts` with a top-level `declare module "vue"`) shadow the real module
/// when registered as program roots, so pulling them in would break `vue`
/// resolution for every file. TypeScript still loads reachable `.d.ts` on demand.
mod options;
pub(super) use options::{ImportFileOptions, TransitiveLocalImports};

#[derive(Default)]
struct ImportReachabilityGraph {
    edges: FxHashMap<PathBuf, FxHashSet<PathBuf>>,
    direct_materialization: FxHashSet<PathBuf>,
}

impl ImportReachabilityGraph {
    fn add_edge(&mut self, importer: &Path, dependency: &Path) {
        self.edges
            .entry(importer.to_path_buf())
            .or_default()
            .insert(dependency.to_path_buf());
    }

    fn mark_direct_materialization(&mut self, path: &Path) {
        self.direct_materialization.insert(path.to_path_buf());
    }

    fn reaches_materialized_source(&self) -> FxHashSet<PathBuf> {
        let mut reverse: FxHashMap<&Path, Vec<&Path>> = FxHashMap::default();
        let mut reached = self.direct_materialization.clone();
        let mut queue = self
            .direct_materialization
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for (importer, dependencies) in &self.edges {
            if importer
                .extension()
                .is_some_and(|extension| extension == "vue")
                && reached.insert(importer.clone())
            {
                queue.push(importer.clone());
            }
            for dependency in dependencies {
                reverse.entry(dependency).or_default().push(importer);
                if dependency
                    .extension()
                    .is_some_and(|extension| extension == "vue")
                    && reached.insert(dependency.clone())
                {
                    queue.push(dependency.clone());
                }
            }
        }
        while let Some(path) = queue.pop() {
            if let Some(importers) = reverse.get(path.as_path()) {
                for importer in importers {
                    if reached.insert((*importer).to_path_buf()) {
                        queue.push((*importer).to_path_buf());
                    }
                }
            }
        }
        reached
    }

    fn reachable_files(&self, roots: impl IntoIterator<Item = PathBuf>) -> FxHashSet<PathBuf> {
        let mut reached = FxHashSet::default();
        let mut queue = roots.into_iter().collect::<Vec<_>>();
        while let Some(path) = queue.pop() {
            if !reached.insert(path.clone()) {
                continue;
            }
            if let Some(dependencies) = self.edges.get(&path) {
                queue.extend(dependencies.iter().cloned());
            }
        }
        reached
    }
}

struct DeferredWorkspaceRoute {
    importer_path: PathBuf,
    specifier: String,
    occurrence_mode: PackageResolutionMode,
    context: PackageResolutionContext,
    route: PackageRoute,
    invalidation_paths: Vec<PathBuf>,
    package_graph: bool,
}

/// Walk the local import graph reachable from `roots`. The returned sets keep
/// virtual registrations separate from authored in-place diagnostic sources;
/// roots are excluded from both and every returned path is absolute.
#[cfg(test)]
pub(super) fn collect_transitive_local_imports(
    roots: &[PathBuf],
    cwd: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: impl Into<ImportFileOptions>,
    aliases: Option<&PathAliasResolver>,
) -> TransitiveLocalImports {
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::new(&mut packages);
    collect_transitive_local_imports_with_session(
        roots,
        cwd,
        canonical_paths,
        options,
        aliases,
        &mut packages,
        &mut session,
    )
}

pub(super) fn collect_transitive_local_imports_with_session(
    roots: &[PathBuf],
    cwd: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: impl Into<ImportFileOptions>,
    aliases: Option<&PathAliasResolver>,
    packages: &mut PackageRouteResolver,
    session: &mut LocalImportSession,
) -> TransitiveLocalImports {
    let options = options.into();
    let alias_scope = self::session::AliasScope::new(aliases, options);
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut registered: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue: Vec<(PathBuf, bool, bool)> = Vec::new();

    // Seed the visited set with the roots so they are never re-registered.
    for root in roots {
        if let Some(absolute) = absolutize(root, cwd, canonical_paths)
            && visited.insert(absolute.clone())
        {
            registered.insert(absolute.clone());
            queue.push((absolute, true, false));
        }
    }

    let mut registrations: Vec<PathBuf> = Vec::new();
    let mut authored: Vec<PathBuf> = Vec::new();
    let mut package_routes = Vec::new();
    let mut reachability_graph = ImportReachabilityGraph::default();
    let mut deferred_workspace_routes = Vec::new();

    // Queue entries below this length were already handed to a frontier.
    // The queue is a stack, so only entries pushed since then are new.
    let mut prepared_queue_len = 0;
    while let Some((file, materialized_parent, package_graph)) = queue.pop() {
        let Some(dir) = file.parent() else {
            continue;
        };
        // Default runs walk the same roots again after adding hidden ambient
        // declarations; the session keeps each file's specifiers for that pass.
        // Package and `paths` misses for this frontier resolve off this thread.
        let new_entries = queue
            .get(prepared_queue_len.min(queue.len())..)
            .unwrap_or_default();
        parallel::prepare_frontier(
            &file,
            new_entries,
            session,
            packages,
            aliases,
            options,
            canonical_paths,
        );
        prepared_queue_len = queue.len();
        let Some(cached) = session.registration_cache.module_specifiers.remove(&file) else {
            continue;
        };
        let Some(occurrences) = cached else {
            session
                .registration_cache
                .module_specifiers
                .insert(file, None);
            continue;
        };
        for occurrence in occurrences.iter() {
            let specifier = occurrence.specifier.clone();
            let relative_specifier = is_relative_specifier(&specifier);
            let absolute_specifier = Path::new(specifier.as_str()).is_absolute();
            let mut package_lookup = None;
            let resolved = if relative_specifier {
                resolve_relative_import(dir, &specifier, canonical_paths, options)
            } else if absolute_specifier {
                resolve_import_base(Path::new(specifier.as_str()), canonical_paths, options)
            } else {
                let aliased = aliases.and_then(|aliases| {
                    session
                        .alias_targets_mut(alias_scope)
                        .entry(specifier.clone())
                        .or_insert_with(|| {
                            aliases.resolve(
                                &specifier,
                                canonical_paths,
                                options,
                                resolve_import_base,
                            )
                        })
                        .clone()
                });
                match aliased {
                    Some(resolved) => Some(resolved),
                    None if is_vue_runtime_support_specifier(&specifier) => None,
                    None => {
                        let context_key = (
                            dir.to_path_buf(),
                            file.extension().map(std::ffi::OsString::from),
                            occurrence.mode,
                        );
                        let (context, context_inputs) = session
                            .resolution_contexts
                            .entry(context_key)
                            .or_insert_with(|| match aliases {
                                Some(aliases) => aliases.package_resolution_context(
                                    packages,
                                    &file,
                                    occurrence.mode,
                                ),
                                None => packages.resolution_context(
                                    &file,
                                    occurrence.mode,
                                    None,
                                    None,
                                    std::iter::empty::<String>(),
                                ),
                            })
                            .clone();
                        let source_options =
                            PackageSourceOptions::new(options.include_js, options.include_jsx);
                        let lookup_key = (
                            dir.to_path_buf(),
                            specifier.clone(),
                            source_options,
                            context.clone(),
                        );
                        let lookup = session
                            .package_lookups
                            .entry(lookup_key)
                            .or_insert_with(|| {
                                packages.lookup_with_context(
                                    dir,
                                    &specifier,
                                    source_options,
                                    context.clone(),
                                )
                            })
                            .clone();
                        let watchable_negative = lookup.is_watchable_negative();
                        let (route, mut consulted) = lookup.into_parts();
                        consulted.extend(context_inputs);
                        let resolved = route.as_ref().and_then(|route| {
                            route.all_source_paths().first().map(|path| (*path).clone())
                        });
                        package_lookup = Some((
                            route,
                            consulted,
                            occurrence.mode,
                            context,
                            watchable_negative,
                        ));
                        resolved
                    }
                }
            };
            if let Some((None, invalidation_paths, mode, context, true)) = package_lookup.as_ref() {
                package_routes.push(PackageRouteBinding {
                    importer_path: file.clone(),
                    specifier: specifier.clone(),
                    occurrence_mode: *mode,
                    context: context.clone(),
                    route: None,
                    invalidation_paths: invalidation_paths.clone(),
                    shared_invalidation_paths: Vec::new(),
                });
            }
            let Some(resolved) = resolved else {
                continue;
            };
            let package_route = package_lookup
                .as_ref()
                .and_then(|(route, _, _, _, _)| route.as_ref());
            // Never register an ambient declaration file — its `declare module`
            // statements would shadow real modules as a program root.
            if package_route.is_none()
                && !package_graph
                && (is_declaration_file(&resolved) || is_node_modules_path(&resolved))
            {
                continue;
            }
            if package_route.is_none() {
                reachability_graph.add_edge(&file, &resolved);
            }
            let own_package_entry = !package_graph
                && package_route.is_some_and(|route| {
                    registration::importer_is_inside_own_package_entry(
                        &file,
                        route,
                        specifier.as_str(),
                    )
                });
            if let Some(route) = package_route
                && route.workspace_source
                && !own_package_entry
                && !specifier.starts_with('#')
                && let Some((_, invalidation_paths, mode, context, _)) = package_lookup.as_ref()
            {
                for candidate in route.all_source_paths() {
                    reachability_graph.add_edge(&file, candidate);
                    if visited.insert(candidate.clone()) {
                        if !is_declaration_file(candidate) && candidate.is_file() {
                            authored.push(candidate.clone());
                        }
                        queue.push((candidate.clone(), false, true));
                    }
                }
                if route.requires_workspace_source_shadow() {
                    reachability_graph.mark_direct_materialization(&file);
                }
                deferred_workspace_routes.push(DeferredWorkspaceRoute {
                    importer_path: file.clone(),
                    specifier: specifier.clone(),
                    occurrence_mode: *mode,
                    context: context.clone(),
                    route: route.clone(),
                    invalidation_paths: invalidation_paths.clone(),
                    package_graph,
                });
                continue;
            }
            let mut discovery = registration::VirtualRegistrationDiscovery::default();
            let needs_registration = if let Some(route) = package_route {
                let mut needs =
                    specifier.starts_with('#') || route.requires_workspace_source_shadow();
                for candidate in route.all_source_paths() {
                    needs |= non_relative_import_needs_virtual_registration(
                        candidate,
                        canonical_paths,
                        options,
                        aliases,
                        Some(packages),
                        session,
                        &mut discovery,
                    );
                }
                needs
            } else if relative_specifier {
                materialized_parent
            } else {
                non_relative_import_needs_virtual_registration(
                    &resolved,
                    canonical_paths,
                    options,
                    aliases,
                    None,
                    session,
                    &mut discovery,
                )
            };
            // A package checked on its own should not widen to its sibling
            // sources through a self-import. Once an app has already reached
            // that package, however, nested self-references are authored
            // members of the same dependency graph and own diagnostics.
            if !own_package_entry {
                package_routes.extend(discovery.package_routes);
            }
            let in_package_graph = package_graph || package_route.is_some();
            // A positive package route always comes from a package lookup.
            if let Some(route) = package_route
                && needs_registration
                && !own_package_entry
                && let Some((_, invalidation_paths, mode, context, _)) = package_lookup.as_ref()
            {
                reachability_graph.mark_direct_materialization(&file);
                if !package_graph && registered.insert(file.clone()) {
                    registrations.push(file.clone());
                }
                let mut route = route.clone();
                let before = route.dependency_paths.len();
                route.dependency_paths.extend(
                    discovery
                        .package_sources
                        .into_iter()
                        .filter(|path| path.starts_with(&route.package_root)),
                );
                if route.dependency_paths.len() != before {
                    route.dependency_paths.sort();
                    route.dependency_paths.dedup();
                }
                package_routes.push(PackageRouteBinding {
                    importer_path: file.clone(),
                    specifier: specifier.clone(),
                    occurrence_mode: *mode,
                    context: context.clone(),
                    route: Some(route.clone()),
                    invalidation_paths: invalidation_paths.clone(),
                    shared_invalidation_paths: Vec::new(),
                });
                for candidate in route.all_source_paths() {
                    if is_declaration_file(candidate) {
                        continue;
                    }
                    if visited.insert(candidate.clone()) {
                        // Synthetic Vue targets preserve native resolution topology;
                        // only physical sources belong to the reported authored set.
                        if candidate.is_file() {
                            authored.push(candidate.clone());
                        }
                        queue.push((candidate.clone(), true, true));
                    }
                    registered.insert(candidate.clone());
                }
            } else if let Some(route) = package_route
                && route.workspace_source
                && !needs_registration
            {
                // Workspace source still owns diagnostics without a Vize mirror (#4137).
                for candidate in route.all_source_paths() {
                    if is_declaration_file(candidate) {
                        continue;
                    }
                    if visited.insert(candidate.clone()) {
                        if candidate.is_file() {
                            authored.push(candidate.clone());
                        }
                        queue.push((candidate.clone(), false, true));
                    }
                }
            }
            let first_visit = package_route.is_none() && visited.insert(resolved.clone());
            let first_registration = package_route.is_none()
                && needs_registration
                && registered.insert(resolved.clone());
            if first_visit && !is_declaration_file(&resolved) {
                authored.push(resolved.clone());
            }
            // Canon registers bare routes after fixing the project root. Adding
            // them to user roots would widen the project to the workspace.
            if first_registration && !in_package_graph {
                registrations.push(resolved.clone());
            }
            if first_visit || first_registration {
                queue.push((resolved, needs_registration, in_package_graph));
            }
        }
        session
            .registration_cache
            .module_specifiers
            .insert(file, Some(occurrences));
    }

    deferred::complete_routes(
        &reachability_graph,
        deferred_workspace_routes,
        &mut registered,
        &mut registrations,
        &mut package_routes,
    );

    sort_package_route_bindings(&mut package_routes);
    TransitiveLocalImports {
        registrations,
        authored,
        package_routes,
    }
}

#[cfg(test)]
mod test_modules;
