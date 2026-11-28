//! Install importer-scoped routes with the fixture's actual compiler options.

use std::path::Path;
use vize_canon::{
    BatchTypeChecker, PackageResolutionMode, PackageRouteBinding, PackageRouteResolver,
    PackageSourceOptions,
};

/// Install the package routes `vize check` resolves for each importer.
pub(super) fn install_package_routes(
    checker: &mut BatchTypeChecker,
    project_root: &Path,
    routes: &[(&str, &str)],
) {
    let options = vize_canon::batch::snapshot_tsconfig_compiler_options(
        project_root,
        &project_root.join("tsconfig.json"),
    )
    .expect("the fixture compiler options must load");
    let string_option = |name| options.get(name).and_then(serde_json::Value::as_str);
    let mut resolver = PackageRouteResolver::default();
    let bindings = routes
        .iter()
        .map(|&(importer, specifier)| {
            let importer = project_root.join(importer);
            let (context, mut context_inputs) = resolver.resolution_context(
                &importer,
                PackageResolutionMode::Import,
                string_option("moduleResolution"),
                string_option("module"),
                std::iter::empty::<&str>(),
            );
            let (route, mut invalidation_paths) = resolver
                .lookup_with_context(
                    importer.parent().expect("the importer must have a parent"),
                    specifier,
                    PackageSourceOptions::new(false, false),
                    context.clone(),
                )
                .into_parts();
            invalidation_paths.append(&mut context_inputs);
            PackageRouteBinding {
                importer_path: importer.clone(),
                specifier: specifier.into(),
                occurrence_mode: PackageResolutionMode::Import,
                context,
                route: Some(route.expect("the fixture package route must resolve")),
                invalidation_paths,
                shared_invalidation_paths: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    checker.set_package_route_resolver(resolver);
    checker.set_package_routes(bindings);
}
