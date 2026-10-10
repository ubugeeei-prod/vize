//! The same effective settings and namespace for cold and reused mirrors.

use std::path::Path;

use crate::batch::virtual_project::VirtualProject;
use crate::corsa_bridge::types::CorsaBridgeError;

pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn configured_project(
    source_path: &Path,
    options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
    environment: crate::corsa_bridge::vue_document::CorsaProjectEnvironment<'_>,
) -> Result<VirtualProject, CorsaBridgeError> {
    let discovered_config = source_path.ancestors().skip(1).find_map(|dir| {
        ["tsconfig.json", "jsconfig.json"]
            .into_iter()
            .map(|name| dir.join(name))
            .find(|path| path.is_file())
    });
    let root = environment
        .project_root
        .map(Path::to_path_buf)
        .or_else(|| {
            discovered_config
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| source_path.parent().unwrap_or(source_path).to_path_buf());
    let root = vize_carton::path::canonicalize_non_verbatim(&root);
    let configured_tsconfig = environment.tsconfig_path.map(|path| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    });

    let mut project = VirtualProject::new(&root).map_err(super::bridge_error)?;
    project.set_virtual_ts_options(environment.virtual_ts_options.clone());
    project.set_options_api(options.options_api);
    project.set_legacy_vue2(options.legacy_vue2);
    project.set_jsx_typecheck(options.jsx_typecheck);
    project.set_experimental_patterned_template(options.experimental_patterned_template);
    project.set_dialect(options.dialect);
    // The workspace root selects the mirror's filesystem scope, not the
    // compiler options for every package beneath it. Anchor the source to its
    // nearest config before resolving a solution-style project's references.
    if let Some(tsconfig) = configured_tsconfig.or(discovered_config) {
        project.set_tsconfig_path(Some(tsconfig));
    }
    project.use_effective_tsconfig_for_source(source_path);
    let namespace_identity = super::super::namespace::editor_namespace_identity(
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
    project.set_package_route_resolver(environment.package_routes.clone());
    Ok(project)
}
