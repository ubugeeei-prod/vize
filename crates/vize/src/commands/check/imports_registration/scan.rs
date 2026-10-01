use super::{
    CanonicalPathCache, FxHashMap, FxHashSet, ImportFileOptions, PackageReachabilityKey, Path,
    PathAliasResolver, RegistrationFrontier, extract_module_specifier_occurrences,
    is_declaration_file, is_relative_specifier, resolve_import_base,
    resolve_import_base_with_inputs, resolve_relative_import,
};

pub(super) fn source_needs_virtual_registration(
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
    negative_alias_sources: &FxHashSet<std::path::PathBuf>,
) -> bool {
    let mut needs_registration = false;
    while let Some(file) = frontier.queue.pop() {
        if !frontier.visited.insert(file.clone()) {
            continue;
        }
        if packages.is_none() && negative_alias_sources.contains(&file) {
            continue;
        }
        if file.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            needs_registration = true;
            continue;
        }

        #[cfg(test)]
        {
            frontier.source_reads += 1;
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
                            std::iter::empty::<vize_l0::String>(),
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
                    super::super::super::imports_package_routes::dedup_paths_sorted(
                        &mut invalidation_paths,
                    );
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
                    std::iter::empty::<vize_l0::String>(),
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
