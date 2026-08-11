use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

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
}

pub(super) type VirtualRegistrationCache = FxHashMap<(PathBuf, bool), CachedVirtualRegistration>;

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
    if let Some(cached) = cache.get(&cache_key) {
        discovery
            .package_routes
            .extend(cached.discovery.package_routes.iter().cloned());
        discovery
            .package_sources
            .extend(cached.discovery.package_sources.iter().cloned());
        return cached.needs_registration;
    }

    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue = vec![path.to_path_buf()];
    let mut discovered = Vec::new();
    let needs_registration = source_needs_virtual_registration(
        &mut visited,
        &mut queue,
        canonical_paths,
        options,
        aliases,
        packages,
        &mut discovered,
    );
    let resolved_discovery = VirtualRegistrationDiscovery {
        package_routes: discovered,
        package_sources: if needs_registration {
            visited.into_iter().collect()
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
    cache.insert(
        cache_key,
        CachedVirtualRegistration {
            needs_registration,
            discovery: resolved_discovery,
        },
    );
    needs_registration
}

fn source_needs_virtual_registration(
    visited: &mut FxHashSet<PathBuf>,
    queue: &mut Vec<PathBuf>,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
    mut packages: Option<&mut vize_canon::PackageRouteResolver>,
    discovered_routes: &mut Vec<vize_canon::PackageRouteBinding>,
) -> bool {
    let mut needs_registration = false;
    while let Some(file) = queue.pop() {
        if !visited.insert(file.clone()) {
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
                            std::iter::empty::<vize_carton::String>(),
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
                    if let Some(route) = route {
                        let reachability = vize_canon::batch::scan_package_route_reachability(
                            &route,
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
                                        resolve_import_base_with_inputs(
                                            candidate,
                                            canonical_paths,
                                            options,
                                        )
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
                                    Some(aliases) => aliases.package_resolution_context(
                                        packages,
                                        importer,
                                        nested_mode,
                                    ),
                                    None => packages.resolution_context(
                                        importer,
                                        nested_mode,
                                        None,
                                        None,
                                        std::iter::empty::<vize_carton::String>(),
                                    ),
                                };
                                let (nested, consulted) = packages
                                    .lookup_with_context(
                                        nested_dir,
                                        nested_specifier,
                                        vize_canon::PackageSourceOptions::new(
                                            options.include_js,
                                            options.include_jsx,
                                        ),
                                        nested_context,
                                    )
                                    .into_parts();
                                nested_inputs.extend(consulted);
                                (nested, nested_inputs)
                            },
                        );
                        reachability.record_work(packages);
                        track_reachability = reachability.requires_tracking();
                        let needs_shadow = reachability.requires_shadow();
                        needs_registration |= needs_shadow;
                        invalidation_paths.extend(reachability.inputs);
                        if needs_shadow {
                            binding_route = Some(route);
                        }
                    }
                    invalidation_paths.sort();
                    invalidation_paths.dedup();
                    if track_reachability || watchable_negative {
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
            if !visited.contains(&resolved) {
                queue.push(resolved);
            }
        }
    }

    needs_registration
}
