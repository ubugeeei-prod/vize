//! Fail-closed construction of one editor alias/package-route snapshot.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::AliasContext;
use super::routes::{RouteDiscovery, package_specifiers_from_frontier};
use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::dependency_scan::resolve_dependency;
use crate::corsa_bridge::types::CorsaBridgeError;

pub(super) fn build(
    source_path: &Path,
    content: &str,
    overlays: &FxHashMap<PathBuf, &str>,
    requested_sources: &[(PathBuf, &str)],
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
    let discovered_root = source_path
        .ancestors()
        .skip(1)
        .find(|dir| dir.join("tsconfig.json").is_file())
        .map(Path::to_path_buf);
    let root = environment
        .project_root
        .map(Path::to_path_buf)
        .or(discovered_root)
        .unwrap_or_else(|| source_path.parent().unwrap_or(source_path).to_path_buf());
    let root = vize_carton::path::canonicalize_non_verbatim(&root);
    let configured_tsconfig = environment.tsconfig_path.map(|path| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    });

    let mut project = VirtualProject::new(&root).map_err(bridge_error)?;
    project.set_virtual_ts_options(environment.virtual_ts_options.clone());
    project.set_options_api(options.options_api);
    project.set_legacy_vue2(options.legacy_vue2);
    project.set_experimental_patterned_template(options.experimental_patterned_template);
    project.set_dialect(options.dialect);
    if let Some(tsconfig) = configured_tsconfig {
        project.set_tsconfig_path(Some(tsconfig));
    } else {
        project.use_effective_tsconfig_for_source(source_path);
    }
    let namespace_identity = super::namespace::editor_namespace_identity(
        options,
        environment.virtual_ts_options,
        Some(&root),
        project.effective_tsconfig_path().as_deref(),
    );
    project.scope_editor_namespace(environment.editor_session.root()?, namespace_identity);
    project.set_session_script_registration(true);
    project.set_editor_document_options(
        crate::batch::virtual_project::VueDocumentVirtualTsOptions {
            options_api: options.options_api,
            legacy_vue2: options.legacy_vue2,
            experimental_patterned_template: options.experimental_patterned_template,
            preserve_event_navigation: options.preserve_event_navigation,
            dialect: options.dialect,
            preserve_missing_vue_diagnostics: true,
        },
    );
    // The native editor queries this one importer. Reachable declarations must
    // be mirrored so user `paths` and relative declaration barrels resolve from
    // the session-private root, but they must stay inferred modules rather than
    // ambient program roots.
    project.set_declaration_roots(&[source_path.to_path_buf()]);
    project.set_package_route_resolver(resolver.clone());
    let package_resolution = project.package_resolution_settings();
    let aliases = project.dependency_alias_map();
    let mut package_routes = FxHashMap::default();
    let mut package_reachability = FxHashMap::default();
    let mut package_bindings = Vec::new();
    let mut route_inputs = Vec::new();

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
        .project_source_paths(project.virtual_root());
    for path in live_sources {
        if path != source_path
            && let Some(source) = overlays.get(&path)
        {
            project
                .register_path_with_content(&path, source)
                .map_err(bridge_error)?;
        }
    }
    for (path, source) in requested_sources {
        let path = vize_carton::path::canonicalize_non_verbatim(path);
        if path == source_path {
            continue;
        }
        let source = overlays.get(&path).copied().unwrap_or(source);
        project
            .register_path_with_content(&path, source)
            .map_err(bridge_error)?;
    }
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
    project.finalize_editor_imports();
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
        mirror,
        virtual_ts_options: environment.virtual_ts_options.clone(),
    })
}

fn bridge_error(error: impl std::fmt::Display) -> CorsaBridgeError {
    CorsaBridgeError::CommunicationError(vize_carton::cstr!("{error}"))
}
