//! Fail-closed construction of one editor alias/package-route snapshot.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::AliasContext;
use super::routes::{RouteDiscovery, package_specifiers_from_frontier};
use crate::batch::virtual_project::dependency_scan::resolve_dependency;
use crate::corsa_bridge::types::CorsaBridgeError;

mod settings;
pub(super) use settings::configured_project;

pub(super) struct SourceRevision<'a> {
    pub requested_sources: &'a [(PathBuf, &'a str)],
    pub overlay_identity: u64,
}

pub(super) fn build(
    source_path: &Path,
    content: &str,
    overlays: &FxHashMap<PathBuf, &str>,
    revision: SourceRevision<'_>,
    resolver: &mut crate::PackageRouteResolver,
    options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
    environment: crate::corsa_bridge::vue_document::CorsaProjectEnvironment<'_>,
) -> Result<AliasContext, CorsaBridgeError> {
    // A caller may reach the same file through a logical symlink spelling
    // (`/var` vs `/private/var` on macOS). Package routes already use physical
    // source identity, so normalize the host once before deriving its mirror
    // path and nearest package scope. Mixing the two spellings splits a
    // package-private `imports` manifest from its generated source companions.
    let source_path = vize_carton::path::canonicalize_non_verbatim(source_path);
    let source_path = source_path.as_path();
    let mut project = configured_project(source_path, options, environment)?;
    let root = project.project_root().to_path_buf();
    let package_resolution = project.package_resolution_settings();
    let aliases = project.dependency_alias_map();
    let mut package_routes = FxHashMap::default();
    let mut package_reachability = FxHashMap::default();
    let mut package_bindings = Vec::new();
    let mut route_inputs = Vec::new();
    let mut seed_paths = vec![source_path.to_path_buf()];

    let register_phase = crate::corsa_bridge::preparation_trace::Phase::start(
        "alias_register_inputs",
        revision.requested_sources.len() + 1,
    );
    project
        .register_path_with_content(source_path, content)
        .map_err(bridge_error)?;
    // An overlay edit invalidates generated contexts, not live membership.
    // Rebuild previously registered open sources in the same project revision
    // before stale contexts are pruned. Otherwise querying a dependency would
    // delete its still-open importer and recreate it on the next query.
    let live_sources = environment
        .editor_session
        .cache()
        .project_sources_to_refresh(project.virtual_root(), revision.overlay_identity);
    for path in live_sources {
        if path != source_path
            && let Some(source) = overlays.get(&path)
        {
            project
                .register_path_with_content(&path, source)
                .map_err(bridge_error)?;
            seed_paths.push(path);
        }
    }
    for (path, source) in revision.requested_sources {
        let path = vize_carton::path::canonicalize_non_verbatim(path);
        if path == source_path {
            continue;
        }
        let source = overlays.get(&path).copied().unwrap_or(source);
        project
            .register_path_with_content(&path, source)
            .map_err(bridge_error)?;
        seed_paths.push(path);
    }
    seed_paths.sort();
    seed_paths.dedup();
    register_phase.finish();
    let reachable_phase =
        crate::corsa_bridge::preparation_trace::Phase::start("alias_reachable_dependencies", 1);
    let virtual_file = project.find_by_original(source_path).ok_or_else(|| {
        CorsaBridgeError::CommunicationError(vize_carton::cstr!(
            "Canon did not retain registered host {}",
            source_path.display()
        ))
    })?;
    let source_type = if virtual_file
        .virtual_path
        .extension()
        .is_some_and(|extension| extension == "tsx")
    {
        oxc_span::SourceType::tsx()
    } else {
        oxc_span::SourceType::ts()
    };
    let host_specifiers = crate::batch::ImportRewriter::new()
        .collect_all_specifier_occurrences(&virtual_file.content, source_type);
    {
        let mut discovery = RouteDiscovery::new(
            &package_resolution,
            resolver,
            &mut package_routes,
            &mut package_reachability,
            &mut package_bindings,
            &mut route_inputs,
            &aliases,
        );
        let mut resolve_package =
            |importer: &Path, specifier: &str, mode: crate::PackageResolutionMode| {
                discovery.resolve(importer, specifier, mode)
            };
        let mut specifiers = host_specifiers
            .iter()
            .filter_map(|(specifier, mode)| {
                let importer_dir = source_path.parent().unwrap_or(source_path);
                if resolve_dependency(specifier, importer_dir, &root, &aliases).is_some() {
                    return None;
                }
                resolve_package(source_path, specifier.as_str(), *mode).then(|| specifier.clone())
            })
            .collect::<Vec<_>>();
        specifiers.sort();
        specifiers.dedup();
        project
            .register_reachable_dependencies_with_package_resolver(
                overlays,
                &specifiers,
                &mut resolve_package,
            )
            .map_err(bridge_error)?;
    }

    reachable_phase.finish();
    let routes_phase =
        crate::corsa_bridge::preparation_trace::Phase::start("alias_package_routes", 1);
    let mut scanned_package_sources = FxHashSet::default();
    loop {
        project.set_package_routes(package_bindings.clone());
        project
            .register_package_route_targets()
            .map_err(bridge_error)?;
        let specifiers = package_specifiers_from_frontier(&project, &mut scanned_package_sources);
        if specifiers.is_empty() {
            break;
        }
        {
            let mut discovery = RouteDiscovery::new(
                &package_resolution,
                resolver,
                &mut package_routes,
                &mut package_reachability,
                &mut package_bindings,
                &mut route_inputs,
                &aliases,
            );
            let mut resolve_package =
                |importer: &Path, specifier: &str, mode: crate::PackageResolutionMode| {
                    discovery.resolve(importer, specifier, mode)
                };
            project
                .register_reachable_dependencies_with_package_resolver(
                    overlays,
                    &specifiers,
                    &mut resolve_package,
                )
                .map_err(bridge_error)?;
        }
        package_bindings.sort_by(|left, right| {
            (&left.importer_path, &left.specifier).cmp(&(&right.importer_path, &right.specifier))
        });
        package_bindings.dedup_by(|left, right| left == right);
    }
    project.set_package_routes(package_bindings);
    project
        .register_package_route_targets()
        .map_err(bridge_error)?;
    project.finalize_package_routes().map_err(bridge_error)?;
    routes_phase.finish();
    route_inputs.sort();
    route_inputs.dedup();
    // A host must retain one session-private identity as dependencies appear
    // and disappear. Switching back to the authored path would leave live
    // native overlays in the previous project after a rename or deletion.
    let mirror = Some(project);

    Ok(AliasContext {
        project_root: root,
        aliases,
        package_routes,
        route_inputs,
        seed_paths,
        #[cfg(test)]
        graph_reused: false,
        mirror,
        virtual_ts_options: environment.virtual_ts_options.clone(),
        query_surface: Default::default(),
    })
}

fn bridge_error(error: impl std::fmt::Display) -> CorsaBridgeError {
    CorsaBridgeError::CommunicationError(vize_carton::cstr!("{error}"))
}

#[cfg(test)]
mod jsconfig_tests;
