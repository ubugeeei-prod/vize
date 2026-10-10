//! Ownership/invalidation laws, with the complete historical prefix oracle.

use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::VirtualProject;
use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding};

fn route(root: &Path) -> PackageRoute {
    PackageRoute {
        source_paths: Vec::new(),
        dependency_paths: Vec::new(),
        source_targets: Vec::new(),
        package_root: root.to_path_buf(),
        package_link_root: root.to_path_buf(),
        manifest_path: root.join("package.json"),
        package_name: Some("ownership-fixture".into()),
        workspace_source: true,
        nested_routes: Vec::new(),
    }
}

fn binding(importer: &Path, name: &str, route: PackageRoute) -> PackageRouteBinding {
    PackageRouteBinding {
        importer_path: importer.to_path_buf(),
        specifier: name.into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: PackageResolutionContext::default(),
        route: Some(route),
        invalidation_paths: Vec::new(),
    }
}

fn historical_roots(project: &VirtualProject, importer: &Path) -> Vec<PathBuf> {
    project
        .package_routes
        .values()
        .filter_map(|binding| binding.route.as_ref())
        .flat_map(PackageRoute::all_routes)
        .filter(|route| importer.starts_with(&route.package_root))
        .map(|route| route.package_root.clone())
        .collect()
}

fn current_roots(project: &VirtualProject, importer: &Path, canonical: bool) -> Vec<PathBuf> {
    let (complete, _) = project.raw_package_roots_are_indexed();
    project.package_roots_for_importer(importer, canonical && complete)
}

fn membership(roots: &[PathBuf], targets: &[PathBuf]) -> Vec<bool> {
    targets
        .iter()
        .map(|target| roots.iter().any(|root| target.starts_with(root)))
        .collect()
}

#[test]
fn indexed_package_ownership_keeps_nested_shared_and_prefix_boundaries() {
    let storage = tempfile::tempdir().unwrap();
    let root = storage.path().canonicalize().unwrap();
    let package = root.join("package");
    let nested = package.join("nested");
    let importer = nested.join("src/importer.ts");
    let mut parent = route(&package);
    parent.nested_routes.push(route(&nested));
    let mut project = VirtualProject::new(&root).unwrap();
    let mut bindings = vec![
        binding(&importer, "one", parent),
        binding(&importer, "shared", route(&package)),
    ];
    bindings.extend((0..512).map(|index| {
        binding(
            &importer,
            cstr!("unrelated-{index}").as_str(),
            route(&root.join(cstr!("unrelated-{index}").as_str())),
        )
    }));
    project.set_package_routes(bindings);
    let targets = [
        package.join("value.ts"),
        nested.join("value.d.ts"),
        root.join("package-old/value.ts"),
        root.join("unrelated-0/value.d.ts"),
        root.join("outside/value.ts"),
    ];
    let indexed = current_roots(&project, &importer, true);
    assert_eq!(indexed, [nested, package]);
    assert_eq!(
        membership(&indexed, &targets),
        [true, true, false, false, false]
    );
    assert_eq!(
        membership(&indexed, &targets),
        membership(&historical_roots(&project, &importer), &targets)
    );
}

#[test]
fn indexed_package_ownership_tracks_last_owner_and_atomic_rebuild() {
    let storage = tempfile::tempdir().unwrap();
    let root = storage.path().canonicalize().unwrap();
    let package = root.join("package");
    let nested = package.join("nested");
    let importer = nested.join("importer.ts");
    let one = binding(&importer, "one", route(&package));
    let two = binding(&importer, "two", route(&package));
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_package_routes([one.clone(), two.clone()]);
    project.remove_package_route_binding(&one.key());
    assert_eq!(current_roots(&project, &importer, true), [package.clone()]);
    project.remove_package_route_binding(&two.key());
    assert!(current_roots(&project, &importer, true).is_empty());
    project.set_package_routes([binding(&importer, "nested", route(&nested))]);
    let targets = [package.join("value.ts"), nested.join("value.ts")];
    let indexed = current_roots(&project, &importer, true);
    assert_eq!(membership(&indexed, &targets), [false, true]);
    assert_eq!(
        membership(&indexed, &targets),
        membership(&historical_roots(&project, &importer), &targets)
    );
}

#[cfg(unix)]
#[test]
fn logical_package_roots_do_not_gain_canonical_importers() {
    let storage = tempfile::tempdir().unwrap();
    let root = storage.path().canonicalize().unwrap();
    let physical = root.join("physical");
    let logical = root.join("logical");
    std::fs::create_dir(&physical).unwrap();
    std::os::unix::fs::symlink(&physical, &logical).unwrap();
    let importer = physical.join("importer.ts");
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_package_routes([binding(&importer, "logical", route(&logical))]);
    assert!(current_roots(&project, &importer, true).is_empty());
    assert!(historical_roots(&project, &importer).is_empty());
    for importer in [logical.join("importer.ts"), logical.join("missing/new.ts")] {
        let retained = current_roots(&project, &importer, false);
        assert_eq!(retained, historical_roots(&project, &importer));
        assert_eq!(
            membership(
                &retained,
                &[physical.join("value.ts"), logical.join("value.ts")]
            ),
            [false, true]
        );
    }
}

#[test]
fn dependency_registration_keeps_only_owned_package_local_declarations() {
    let storage = tempfile::tempdir().unwrap();
    let root = storage.path().canonicalize().unwrap();
    let package = root.join("node_modules/package");
    let unrelated = root.join("node_modules/package-other");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::create_dir_all(&unrelated).unwrap();
    let importer = package.join("importer.ts");
    let owned = package.join("value.d.ts");
    let excluded = unrelated.join("value.d.ts");
    std::fs::write(&importer, "import { value } from './value'; import { other } from '../package-other/value'; void value; void other;\n").unwrap();
    std::fs::write(&owned, "export declare const value: string;\n").unwrap();
    std::fs::write(&excluded, "export declare const other: number;\n").unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_package_routes([
        binding(&importer, "owned", route(&package)),
        binding(&importer, "unrelated", route(&unrelated)),
    ]);
    project.register_path(&importer).unwrap();
    project.register_reachable_dependencies().unwrap();
    assert_eq!(
        project.registered_original_paths_sorted(),
        [importer, owned]
    );
    assert!(project.find_by_original(&excluded).is_none());
}

#[cfg(unix)]
#[test]
fn replaced_or_retargeted_symlink_keeps_whole_historical_ownership() {
    let storage = tempfile::tempdir().unwrap();
    let root = storage.path().canonicalize().unwrap();
    let physical_a = root.join("physical-a");
    let physical_b = root.join("physical-b");
    let logical = root.join("node_modules/package");
    std::fs::create_dir_all(logical.parent().unwrap()).unwrap();
    std::fs::create_dir(&physical_a).unwrap();
    std::fs::create_dir(&physical_b).unwrap();
    std::os::unix::fs::symlink(&physical_a, &logical).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_package_routes([binding(&root.join("app.ts"), "logical", route(&logical))]);
    assert!(!project.raw_package_roots_are_indexed().0);

    std::fs::remove_file(&logical).unwrap();
    std::os::unix::fs::symlink(&physical_b, &logical).unwrap();
    let physical_importer = physical_b.join("importer.ts");
    std::fs::write(&physical_importer, "export {};\n").unwrap();
    assert_eq!(
        current_roots(&project, &physical_importer, true),
        historical_roots(&project, &physical_importer)
    );
    assert!(historical_roots(&project, &physical_importer).is_empty());

    std::fs::remove_file(&logical).unwrap();
    let nested = logical.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let importer = nested.join("importer.ts");
    let declaration = logical.join("sibling.d.ts");
    std::fs::write(
        &importer,
        "import { value } from '../sibling'; void value;\n",
    )
    .unwrap();
    std::fs::write(&declaration, "export declare const value: string;\n").unwrap();
    project.insert_package_route_binding(binding(&importer, "nested", route(&nested)));
    assert!(!project.raw_package_roots_are_indexed().0);
    assert_eq!(
        super::super::resolution::canonical_key(&importer),
        Some(importer.clone())
    );
    let unguarded = project.package_roots_for_importer(&importer, true);
    assert_eq!(unguarded, [nested]);
    assert_eq!(
        membership(&unguarded, std::slice::from_ref(&declaration)),
        [false]
    );
    let retained = current_roots(&project, &importer, true);
    assert_eq!(retained, historical_roots(&project, &importer));
    assert_eq!(
        membership(&retained, std::slice::from_ref(&declaration)),
        [true]
    );
    project.register_path(&importer).unwrap();
    project.register_reachable_dependencies().unwrap();
    assert_eq!(
        project.registered_original_paths_sorted(),
        [importer, declaration]
    );
}

#[test]
fn empty_relative_and_parent_components_keep_the_original_raw_prefix_law() {
    let storage = tempfile::tempdir().unwrap();
    let root = storage.path().canonicalize().unwrap();
    let package = root.join("package");
    std::fs::create_dir(&package).unwrap();
    let importer = package.join("importer.ts");
    std::fs::write(&importer, "export {};\n").unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    for raw in [
        PathBuf::new(),
        PathBuf::from("package"),
        root.join("package/../package"),
    ] {
        project.set_package_routes([binding(&importer, "raw", route(&raw))]);
        assert!(!project.raw_package_roots_are_indexed().0);
        let retained = current_roots(&project, &importer, true);
        assert_eq!(retained, historical_roots(&project, &importer));
        assert_eq!(
            membership(&retained, &[package.join("value.ts")]),
            [raw.as_os_str().is_empty()]
        );
        let raw_importer = raw.join("missing/importer.ts");
        let retained = current_roots(&project, &raw_importer, false);
        assert_eq!(retained, historical_roots(&project, &raw_importer));
        assert_eq!(membership(&retained, &[raw.join("value.ts")]), [true]);
    }
}
