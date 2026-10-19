//! `is_package_route_path` reads an owner-counted index of route source paths;
//! it must follow every binding insert, replace and removal.

use std::fs;
use std::path::{Path, PathBuf};

use crate::batch::virtual_project::VirtualProject;
use crate::{PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding};

use super::case_dir::unique_case_dir;

fn route(
    package_root: &Path,
    source_paths: Vec<PathBuf>,
    nested: Vec<PackageRoute>,
) -> PackageRoute {
    PackageRoute {
        source_paths,
        dependency_paths: Vec::new(),
        source_targets: Vec::new(),
        package_root: package_root.to_path_buf(),
        package_link_root: package_root.to_path_buf(),
        manifest_path: package_root.join("package.json"),
        package_name: Some("@w/ui".into()),
        workspace_source: false,
        nested_routes: nested,
    }
}

fn binding(importer_path: &Path, route: PackageRoute) -> PackageRouteBinding {
    PackageRouteBinding {
        importer_path: importer_path.to_path_buf(),
        specifier: "@w/ui".into(),
        occurrence_mode: PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(route),
        invalidation_paths: Vec::new(),
        shared_invalidation_paths: Vec::new(),
    }
}

#[test]
fn route_source_lookup_follows_binding_lifecycle() {
    let root = unique_case_dir("package-route-source-paths");
    let package_root = root.join("packages/ui");
    let nested_root = root.join("packages/icons");
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::create_dir_all(nested_root.join("src")).unwrap();
    let shared = package_root.join("src/index.ts");
    let nested = nested_root.join("src/index.ts");
    let unrelated = root.join("src/main.ts");
    for path in [&shared, &nested] {
        fs::write(path, "export {};\n").unwrap();
    }

    let first_importer = root.join("apps/a/src/App.vue");
    let second_importer = root.join("apps/b/src/App.vue");
    let first = binding(
        &first_importer,
        route(
            &package_root,
            vec![shared.clone()],
            vec![route(&nested_root, vec![nested.clone()], Vec::new())],
        ),
    );
    let second = binding(
        &second_importer,
        route(&package_root, vec![shared.clone()], Vec::new()),
    );

    let mut project = VirtualProject::new(&root).unwrap();
    project.replace_package_route_bindings([first.clone(), second.clone()]);
    assert!(project.is_package_route_path(&shared));
    assert!(
        project.is_package_route_path(&nested),
        "nested route sources count"
    );
    assert!(!project.is_package_route_path(&unrelated));

    // `shared` is still owned by the second binding.
    project.remove_package_route_binding(&first.key());
    assert!(project.is_package_route_path(&shared));
    assert!(!project.is_package_route_path(&nested));

    project.remove_package_route_binding(&second.key());
    assert!(!project.is_package_route_path(&shared));

    project.insert_package_route_binding(first);
    assert!(project.is_package_route_path(&nested));
    project.replace_package_route_bindings([second]);
    assert!(project.is_package_route_path(&shared));
    assert!(
        !project.is_package_route_path(&nested),
        "replace drops old sources"
    );

    fs::remove_dir_all(&root).ok();
}
