use std::path::{Path, PathBuf};

use vize_s0::{FxHashMap, FxHashSet};

use super::super::imports_aliases::PathAliasResolver;
use super::super::path_cache::CanonicalPathCache;
use super::{
    ImportFileOptions, extract_module_specifier_occurrences, is_declaration_file,
    is_relative_specifier, resolve_import_base, resolve_import_base_with_inputs,
    resolve_relative_import,
};

#[derive(Clone, Default)]
pub(super) struct VirtualRegistrationDiscovery {
    pub(super) package_routes: Vec<vize_canon::PackageRouteBinding>,
    pub(super) package_sources: Vec<PathBuf>,
}

#[derive(Clone)]
pub(super) struct CachedVirtualRegistration {
    needs_registration: bool,
    discovery: VirtualRegistrationDiscovery,
    package_routes_reported: bool,
}

/// Reachability of one package route never depends on the importer that
/// reached it: the bounded scan resolves nested specifiers from the package's
/// own files. Keying on the manifest, spelling, and resolution context keeps
/// the scan to once per distinct route instead of once per occurrence.
type PackageReachabilityKey = (
    PathBuf,
    vize_s0::String,
    vize_canon::PackageResolutionContext,
    u8,
);

#[derive(Default)]
pub(super) struct VirtualRegistrationCache {
    registrations: FxHashMap<(PathBuf, bool), CachedVirtualRegistration>,
    reachability: FxHashMap<PackageReachabilityKey, vize_canon::batch::PackageRouteReachability>,
}

impl VirtualRegistrationCache {
    #[cfg(test)]
    pub(super) fn registration_entries(&self) -> usize {
        self.registrations.len()
    }
}

/// One walk of the sources reachable from a single registration candidate.
#[derive(Default)]
struct RegistrationFrontier {
    visited: FxHashSet<PathBuf>,
    queue: Vec<PathBuf>,
}

pub(super) fn non_relative_import_needs_virtual_registration(
    path: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
    packages: Option<&mut vize_canon::PackageRouteResolver>,
    cache: &mut VirtualRegistrationCache,
    discovery: &mut VirtualRegistrationDiscovery,
) -> bool {
    let cache_key = (path.to_path_buf(), packages.is_some());
    if let Some(cached) = cache.registrations.get_mut(&cache_key) {
        // Route bindings are collection-global facts; sources are caller-local
        // invalidation inputs for the outer route that reached this memo.
        if !cached.package_routes_reported {
            discovery
                .package_routes
                .extend(cached.discovery.package_routes.iter().cloned());
            cached.package_routes_reported = true;
        }
        discovery
            .package_sources
            .extend(cached.discovery.package_sources.iter().cloned());
        return cached.needs_registration;
    }

    let mut frontier = RegistrationFrontier {
        visited: FxHashSet::default(),
        queue: vec![path.to_path_buf()],
    };
    let mut discovered = Vec::new();
    let needs_registration = source_needs_virtual_registration(
        &mut frontier,
        canonical_paths,
        options,
        aliases,
        packages,
        &mut cache.reachability,
        &mut discovered,
    );
    let resolved_discovery = VirtualRegistrationDiscovery {
        package_routes: discovered,
        package_sources: if needs_registration {
            frontier.visited.into_iter().collect()
        } else {
            Vec::new()
        },
    };
    discovery
        .package_routes
        .extend(resolved_discovery.package_routes.iter().cloned());
    discovery
        .package_sources
        .extend(resolved_discovery.package_sources.iter().cloned());
    cache.registrations.insert(
        cache_key,
        CachedVirtualRegistration {
            needs_registration,
            discovery: resolved_discovery,
            package_routes_reported: true,
        },
    );
    needs_registration
}

fn source_needs_virtual_registration(
    frontier: &mut RegistrationFrontier,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
    mut packages: Option<&mut vize_canon::PackageRouteResolver>,
    reachability_cache: &mut FxHashMap<
        PackageReachabilityKey,
        vize_canon::batch::PackageRouteReachability,
    >,
    discovered_routes: &mut Vec<vize_canon::PackageRouteBinding>,
) -> bool {
    let mut needs_registration = false;
    while let Some(file) = frontier.queue.pop() {
        if !frontier.visited.insert(file.clone()) {
            continue;
        }
        if file.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            needs_registration = true;
            continue;
        }

        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Some(dir) = file.parent() else {
            continue;
        };

        for occurrence in extract_module_specifier_occurrences(&source) {
            let specifier = occurrence.specifier;
            let candidate = Path::new(specifier.as_str());
            let resolved = if is_relative_specifier(&specifier) {
                resolve_relative_import(dir, &specifier, canonical_paths, options)
            } else if candidate.is_absolute() {
                resolve_import_base(candidate, canonical_paths, options)
            } else {
                let aliased = aliases.and_then(|aliases| {
                    aliases.resolve(&specifier, canonical_paths, options, resolve_import_base)
                });
                if aliased.is_some() {
                    aliased
                } else if vize_canon::batch::is_vue_runtime_support_specifier(&specifier) {
                    None
                } else if let Some(packages) = packages.as_deref_mut() {
                    let (context, context_inputs) = match aliases {
                        Some(aliases) => {
                            aliases.package_resolution_context(packages, &file, occurrence.mode)
                        }
                        None => packages.resolution_context(
                            &file,
                            occurrence.mode,
                            None,
                            None,
                            std::iter::empty::<vize_s0::String>(),
                        ),
                    };
                    let lookup = packages.lookup_with_context(
                        dir,
                        &specifier,
                        vize_canon::PackageSourceOptions::new(
                            options.include_js,
                            options.include_jsx,
                        ),
                        context.clone(),
                    );
                    let watchable_negative = lookup.is_watchable_negative();
                    let (route, mut invalidation_paths) = lookup.into_parts();
                    invalidation_paths.extend(context_inputs);
                    invalidation_paths.push(file.clone());
                    let mut binding_route = None;
                    let mut track_reachability = false;
                    let mut needs_shadow = false;
                    if let Some(route) = route {
                        let reachability_key = (
                            route.manifest_path.clone(),
                            specifier.clone(),
                            context.clone(),
                            vize_canon::batch::PACKAGE_REACHABILITY_BUDGET_REVISION,
                        );
                        let reachability = match reachability_cache.get(&reachability_key) {
                            Some(cached) => cached.clone(),
                            None => {
                                let scanned = scan_route_reachability(
                                    &route,
                                    canonical_paths,
                                    options,
                                    aliases,
                                    packages,
                                );
                                scanned.record_work(packages);
                                reachability_cache.insert(reachability_key, scanned.clone());
                                scanned
                            }
                        };
                        track_reachability = reachability.requires_tracking();
                        needs_shadow = reachability.requires_shadow()
                            || route.requires_workspace_source_shadow();
                        needs_registration |= needs_shadow;
                        invalidation_paths.extend(reachability.inputs);
                        if needs_shadow {
                            binding_route = Some(route);
                        }
                    }
                    invalidation_paths.sort();
                    invalidation_paths.dedup();
                    if needs_shadow || track_reachability || watchable_negative {
                        discovered_routes.push(vize_canon::PackageRouteBinding {
                            importer_path: file.clone(),
                            specifier: specifier.clone(),
                            occurrence_mode: occurrence.mode,
                            context: context.clone(),
                            route: binding_route,
                            invalidation_paths,
                        });
                    }
                    None
                } else {
                    None
                }
            };
            let Some(resolved) = resolved else {
                continue;
            };
            if is_declaration_file(&resolved) && packages.is_none() {
                continue;
            }
            if resolved
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("vue")
            {
                needs_registration = true;
                continue;
            }
            if !frontier.visited.contains(&resolved) {
                frontier.queue.push(resolved);
            }
        }
    }

    needs_registration
}

/// Bounded Vue reachability for one package route, using the same local and
/// package resolution the check import walk uses for authored sources.
fn scan_route_reachability(
    route: &vize_canon::PackageRoute,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
    packages: &mut vize_canon::PackageRouteResolver,
) -> vize_canon::batch::PackageRouteReachability {
    vize_canon::batch::scan_package_route_reachability(
        route,
        |importer, nested_specifier| {
            let nested_dir = importer.parent().unwrap_or(importer);
            if is_relative_specifier(nested_specifier) {
                resolve_import_base_with_inputs(
                    &nested_dir.join(nested_specifier),
                    canonical_paths,
                    options,
                )
            } else {
                let candidate = Path::new(nested_specifier);
                if candidate.is_absolute() {
                    resolve_import_base_with_inputs(candidate, canonical_paths, options)
                } else {
                    aliases.map_or_else(
                        || (None, Vec::new()),
                        |aliases| {
                            aliases.resolve_with_inputs(
                                nested_specifier,
                                canonical_paths,
                                options,
                                resolve_import_base_with_inputs,
                            )
                        },
                    )
                }
            }
        },
        |importer, nested_specifier, nested_mode| {
            let nested_dir = importer.parent().unwrap_or(importer);
            let (nested_context, mut nested_inputs) = match aliases {
                Some(aliases) => {
                    aliases.package_resolution_context(packages, importer, nested_mode)
                }
                None => packages.resolution_context(
                    importer,
                    nested_mode,
                    None,
                    None,
                    std::iter::empty::<vize_s0::String>(),
                ),
            };
            let (nested, consulted) = packages
                .lookup_with_context(
                    nested_dir,
                    nested_specifier,
                    vize_canon::PackageSourceOptions::new(options.include_js, options.include_jsx),
                    nested_context,
                )
                .into_parts();
            nested_inputs.extend(consulted);
            (nested, nested_inputs)
        },
    )
}
