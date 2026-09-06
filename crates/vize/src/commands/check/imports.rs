//! Transitive resolution of source imports for `vize check` virtual projects.
//!
//! A check run may intentionally report diagnostics for only a subset of
//! sources. This module separates authored files that TypeScript can resolve
//! in place from files that must enter Vize's mirror for Vue import rewriting.

use std::path::{Path, PathBuf};

use vize_canon::batch::is_vue_runtime_support_specifier;
use vize_canon::{PackageRouteBinding, PackageRouteResolver, PackageSourceOptions};
#[cfg(test)]
use vize_s0::cstr;
use vize_s0::{FxHashSet, String};

use super::imports_aliases::PathAliasResolver;
use super::imports_package_routes::sort_package_route_bindings;
use super::path_cache::CanonicalPathCache;

#[path = "imports_cache.rs"]
mod cache;
use cache::{PackageLookupCache, ResolutionContextCache};
#[path = "imports_registration.rs"]
mod registration;
use registration::VirtualRegistrationCache;
use registration::non_relative_import_needs_virtual_registration;
#[path = "imports_resolution.rs"]
mod resolution;
use resolution::{absolutize, is_declaration_file, is_node_modules_path, resolve_relative_import};
pub(super) use resolution::{resolve_import_base, resolve_import_base_with_inputs};
#[path = "imports_specifiers.rs"]
mod specifiers;
use specifiers::{extract_module_specifier_occurrences, is_relative_specifier};

/// Source extensions whose imports carry TypeScript types worth pulling into the
/// virtual project, in module-resolution precedence order.
///
/// `.d.ts` is deliberately excluded: ambient declaration files (e.g. a project
/// `shims.d.ts` with a top-level `declare module "vue"`) shadow the real module
/// when registered as program roots, so pulling them in would break `vue`
/// resolution for every file. TypeScript still loads reachable `.d.ts` on demand.
#[path = "imports_options.rs"]
mod options;
pub(super) use options::{ImportFileOptions, TransitiveLocalImports};

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
    collect_transitive_local_imports_with_resolver(
        roots,
        cwd,
        canonical_paths,
        options,
        aliases,
        &mut PackageRouteResolver::default(),
    )
}

pub(super) fn collect_transitive_local_imports_with_resolver(
    roots: &[PathBuf],
    cwd: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: impl Into<ImportFileOptions>,
    aliases: Option<&PathAliasResolver>,
    packages: &mut PackageRouteResolver,
) -> TransitiveLocalImports {
    packages.begin_validation_epoch();
    let options = options.into();
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut registered: FxHashSet<PathBuf> = FxHashSet::default();
    let mut registration_cache = VirtualRegistrationCache::default();
    let mut resolution_contexts = ResolutionContextCache::default();
    let mut package_lookups = PackageLookupCache::default();
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

    while let Some((file, materialized_parent, package_graph)) = queue.pop() {
        let Some(dir) = file.parent() else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        // Scan the raw file text directly — the byte scanner only reacts to
        // `import`/`from` string operands, so an SFC's `<template>`/`<style>`
        // are inert and no `.vue` parse is needed on this hot path.
        for occurrence in extract_module_specifier_occurrences(&source) {
            let specifier = occurrence.specifier;
            let relative_specifier = is_relative_specifier(&specifier);
            let absolute_specifier = Path::new(specifier.as_str()).is_absolute();
            let mut package_lookup = None;
            let resolved = if relative_specifier {
                resolve_relative_import(dir, &specifier, canonical_paths, options)
            } else if absolute_specifier {
                resolve_import_base(Path::new(specifier.as_str()), canonical_paths, options)
            } else {
                let aliased = aliases.and_then(|aliases| {
                    aliases.resolve(&specifier, canonical_paths, options, resolve_import_base)
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
                        let (context, context_inputs) = resolution_contexts
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
                        let lookup = package_lookups
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
                && (is_declaration_file(&resolved)
                    || (!package_graph && is_node_modules_path(&resolved)))
            {
                continue;
            }
            let mut discovery = registration::VirtualRegistrationDiscovery::default();
            let needs_registration = if let Some(route) = package_route {
                let mut needs_registration = route.requires_workspace_source_shadow();
                for candidate in route.all_source_paths() {
                    needs_registration |= non_relative_import_needs_virtual_registration(
                        candidate,
                        canonical_paths,
                        options,
                        aliases,
                        Some(packages),
                        &mut registration_cache,
                        &mut discovery,
                    );
                }
                needs_registration
            } else if relative_specifier {
                materialized_parent
            } else {
                non_relative_import_needs_virtual_registration(
                    &resolved,
                    canonical_paths,
                    options,
                    aliases,
                    None,
                    &mut registration_cache,
                    &mut discovery,
                )
            };
            package_routes.extend(discovery.package_routes);
            let in_package_graph = package_graph || package_route.is_some();
            if let Some(route) = package_route
                && needs_registration
            {
                if !package_graph && registered.insert(file.clone()) {
                    registrations.push(file.clone());
                }
                let (_, invalidation_paths, mode, context, _) = package_lookup
                    .as_ref()
                    .expect("a positive package route came from a package lookup");
                let mut route = route.clone();
                // Re-sort only when the extension actually added something; most
                // workspace packages arrive sorted and deduped, so the common case
                // avoids a component-wise path sort over long pnpm paths (#4426).
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
                });
                for candidate in route.all_source_paths() {
                    if is_declaration_file(candidate) {
                        continue;
                    }
                    if visited.insert(candidate.clone()) {
                        authored.push(candidate.clone());
                        queue.push((candidate.clone(), true, true));
                    }
                    registered.insert(candidate.clone());
                }
            } else if let Some(route) = package_route
                && route.workspace_source
            {
                // A workspace package reached through `node_modules` is real
                // project source, so it owns diagnostics even when nothing in
                // it needs a Vize mirror. Only the mirror is optional here:
                // skipping the walk would silently drop every error authored
                // inside a plain TypeScript workspace package (#4137).
                for candidate in route.all_source_paths() {
                    if is_declaration_file(candidate) {
                        continue;
                    }
                    if visited.insert(candidate.clone()) {
                        authored.push(candidate.clone());
                        queue.push((candidate.clone(), false, true));
                    }
                }
            }
            let first_visit = package_route.is_none() && visited.insert(resolved.clone());
            let first_registration = package_route.is_none()
                && needs_registration
                && registered.insert(resolved.clone());
            if first_visit {
                authored.push(resolved.clone());
            }
            // A bare package route is registered by Canon after the project
            // root is fixed. Adding it (or its relative descendants) to the
            // user's roots here would widen the project to the workspace and
            // defeat the external mirror.
            if first_registration && !in_package_graph {
                registrations.push(resolved.clone());
            }
            if first_visit || first_registration {
                queue.push((resolved, needs_registration, in_package_graph));
            }
        }
    }

    sort_package_route_bindings(&mut package_routes);
    TransitiveLocalImports {
        registrations,
        authored,
        package_routes,
    }
}

#[cfg(test)]
#[path = "imports_generated_tests.rs"]
mod generated_tests;
#[cfg(test)]
#[path = "imports_js_tests.rs"]
mod js_tests;
#[cfg(test)]
#[path = "imports_magic_comments_tests.rs"]
mod magic_comments_tests;
#[cfg(test)]
#[path = "imports_package_cache_tests.rs"]
mod package_cache_tests;
#[cfg(test)]
#[path = "imports_package_registration_cache_tests.rs"]
mod package_registration_cache_tests;
#[cfg(test)]
#[path = "imports_package_registration_shadow_tests.rs"]
mod package_registration_shadow_tests;
#[cfg(test)]
#[path = "imports_package_registration_tests.rs"]
mod package_registration_tests;
#[cfg(test)]
#[path = "imports_tests.rs"]
mod tests;
