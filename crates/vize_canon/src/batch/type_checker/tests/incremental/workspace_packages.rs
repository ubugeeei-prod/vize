use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding};

#[path = "workspace_packages/external_symlink.rs"]
mod external_symlink;
#[path = "workspace_packages/routes.rs"]
mod routes;

fn package_binding(
    importer_path: &std::path::Path,
    specifier: &str,
    package_root: &std::path::Path,
    source_path: &std::path::Path,
) -> PackageRouteBinding {
    PackageRouteBinding {
        importer_path: importer_path.to_path_buf(),
        specifier: specifier.into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(PackageRoute {
            source_paths: vec![source_path.to_path_buf()],
            dependency_paths: Vec::new(),
            source_targets: vec![crate::PackageRouteSource {
                target_path: source_path.to_path_buf(),
                source_path: source_path.to_path_buf(),
                native_probe_path: source_path.with_extension("d.vue.ts"),
            }],
            package_root: package_root.to_path_buf(),
            package_link_root: package_root.to_path_buf(),
            manifest_path: package_root.join("package.json"),
            package_name: Some(specifier.into()),
            workspace_source: true,
            nested_routes: Vec::new(),
        }),
        invalidation_paths: vec![source_path.to_path_buf(), package_root.join("package.json")],
    }
}
