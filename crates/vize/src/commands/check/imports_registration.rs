use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::super::imports_aliases::PathAliasResolver;
use super::super::path_cache::CanonicalPathCache;
use super::{
    ImportFileOptions, extract_module_specifier_occurrences, is_declaration_file,
    is_relative_specifier, resolve_import_base, resolve_relative_import,
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
    let resolved_discovery = if needs_registration {
        VirtualRegistrationDiscovery {
            package_routes: discovered,
            package_sources: visited.into_iter().collect(),
        }
    } else {
        VirtualRegistrationDiscovery::default()
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
    let mut reaches_vue = false;
    while let Some(file) = queue.pop() {
        if !visited.insert(file.clone()) {
            continue;
        }
        if file.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            reaches_vue = true;
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
                    let route = packages.lookup_with_context(
                        dir,
                        &specifier,
                        vize_canon::PackageSourceOptions::new(
                            options.include_js,
                            options.include_jsx,
                        ),
                        context.clone(),
                    );
                    let watchable_negative = route.is_watchable_negative();
                    let (route, mut invalidation_paths) = route.into_parts();
                    invalidation_paths.extend(context_inputs);
                    invalidation_paths.push(file.clone());
                    invalidation_paths.sort();
                    invalidation_paths.dedup();
                    if route.is_some() || watchable_negative {
                        discovered_routes.push(vize_canon::PackageRouteBinding {
                            importer_path: file.clone(),
                            specifier: specifier.clone(),
                            occurrence_mode: occurrence.mode,
                            context: context.clone(),
                            route: route.clone(),
                            invalidation_paths,
                        });
                    }
                    if let Some(route) = route {
                        for source in route.all_source_paths() {
                            if source
                                .extension()
                                .is_some_and(|extension| extension == "vue")
                            {
                                reaches_vue = true;
                                continue;
                            }
                            if !visited.contains(source) {
                                queue.push(source.clone());
                            }
                        }
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
                reaches_vue = true;
                continue;
            }
            if !visited.contains(&resolved) {
                queue.push(resolved);
            }
        }
    }

    reaches_vue
}
