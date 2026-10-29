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
    route_with_dependencies(package_root, source_paths, Vec::new(), nested)
}

fn route_with_dependencies(
    package_root: &Path,
    source_paths: Vec<PathBuf>,
    dependency_paths: Vec<PathBuf>,
    nested: Vec<PackageRoute>,
) -> PackageRoute {
    PackageRoute {
        source_paths,
        dependency_paths,
        source_targets: Vec::new(),
        package_root: package_root.to_path_buf(),
        package_link_root: package_root.to_path_buf(),
        manifest_path: package_root.join("package.json"),
        package_name: Some("@w/ui".into()),
        workspace_source: false,
        nested_routes: nested,
    }
}

fn expected_source_index<'a>(
    project: &VirtualProject,
    root: &Path,
    sources: impl IntoIterator<Item = &'a Path>,
) -> vize_carton::FxHashMap<PathBuf, (PathBuf, PathBuf)> {
    let root = vize_carton::path::canonicalize_non_verbatim(root);
    sources
        .into_iter()
        .map(|source| {
            let source = vize_carton::path::canonicalize_non_verbatim(source);
            let relative = source.strip_prefix(&root).unwrap().to_path_buf();
            let virtual_path = project.original_index.get(&source).unwrap().clone();
            (source, (relative, virtual_path))
        })
        .collect()
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

#[test]
fn route_source_index_preserves_scopes_dependencies_and_canonical_collisions() {
    let root = unique_case_dir("package-route-source-index-scopes");
    let _ = fs::remove_dir_all(&root);
    let package_root = root.join("packages/host");
    let first_root = package_root.join("node_modules/first");
    let second_root = package_root.join("node_modules/second");
    for directory in [&package_root, &first_root, &second_root] {
        fs::create_dir_all(directory.join("src")).unwrap();
    }

    let outer = package_root.join("src/outer.ts");
    let first = first_root.join("src/first.ts");
    let first_dependency = first_root.join("src/dependency.ts");
    let same_root_nested = first_root.join("src/nested.ts");
    let second = second_root.join("src/second.ts");
    let outside = root.join("outside.ts");
    let registered = vec![
        outer.clone(),
        first.clone(),
        first_dependency.clone(),
        same_root_nested.clone(),
        second.clone(),
        outside.clone(),
    ];
    let mut first_dependencies = vec![first_dependency.clone()];

    #[cfg(unix)]
    let non_utf8 = {
        use std::os::unix::ffi::OsStringExt;

        let non_utf8 = first_root
            .join("src")
            .join(std::ffi::OsString::from_vec(b"non-\x80.ts".to_vec()));
        first_dependencies.push(non_utf8.clone());
        non_utf8
    };

    for source in &registered {
        fs::write(source, "export {}\n").unwrap();
    }
    #[cfg(unix)]
    fs::write(&non_utf8, "export {}\n").unwrap();

    let lexical_alias = first_root.join("src/../src/first.ts");
    let mut first_sources = vec![first.clone(), lexical_alias, first.clone()];
    #[cfg(unix)]
    {
        let symlink_root = root.join("first-link");
        std::os::unix::fs::symlink(&first_root, &symlink_root).unwrap();
        first_sources.push(symlink_root.join("src/first.ts"));
    }

    let route = route_with_dependencies(
        &package_root,
        vec![outer.clone(), outside.clone()],
        Vec::new(),
        vec![
            route_with_dependencies(
                &first_root,
                first_sources,
                first_dependencies.clone(),
                vec![route(
                    &first_root,
                    vec![same_root_nested.clone(), first.clone()],
                    Vec::new(),
                )],
            ),
            route(&second_root, vec![second.clone()], Vec::new()),
        ],
    );

    let mut project = VirtualProject::new(&root).unwrap();
    for source in &registered {
        project.register_path(source).unwrap();
    }
    #[cfg(unix)]
    assert!(
        project.register_path(&non_utf8).is_err(),
        "the registration contract must refuse a non-UTF-8 authored path"
    );
    project.index_existing_package_route_sources(&route, None);

    let first_expected_sources = vec![
        first.as_path(),
        first_dependency.as_path(),
        same_root_nested.as_path(),
    ];
    let first_expected = expected_source_index(
        &project,
        &first_root,
        first_expected_sources.iter().copied(),
    );
    let second_expected = expected_source_index(&project, &second_root, [second.as_path()]);
    let outer_expected = expected_source_index(
        &project,
        &package_root,
        [outer.as_path()]
            .into_iter()
            .chain(first_expected_sources.iter().copied())
            .chain([second.as_path()]),
    );

    let canonical_outer = vize_carton::path::canonicalize_non_verbatim(&package_root);
    let canonical_first = vize_carton::path::canonicalize_non_verbatim(&first_root);
    let canonical_second = vize_carton::path::canonicalize_non_verbatim(&second_root);
    assert_eq!(project.package_source_index.len(), 3);
    assert_eq!(
        project.package_source_index.get(&canonical_outer),
        Some(&outer_expected)
    );
    assert_eq!(
        project.package_source_index.get(&canonical_first),
        Some(&first_expected)
    );
    assert_eq!(
        project.package_source_index.get(&canonical_second),
        Some(&second_expected)
    );
    let canonical_outside = vize_carton::path::canonicalize_non_verbatim(&outside);
    assert!(
        project
            .package_source_index
            .values()
            .all(|sources| !sources.contains_key(&canonical_outside)),
        "registered sources outside every route root must be refused"
    );
    #[cfg(unix)]
    {
        let canonical_non_utf8 = vize_carton::path::canonicalize_non_verbatim(&non_utf8);
        assert!(
            project
                .package_source_index
                .values()
                .all(|sources| !sources.contains_key(&canonical_non_utf8)),
            "an authored non-UTF-8 dependency without a registered source must be skipped"
        );
    }

    fs::remove_dir_all(&root).ok();
}

#[test]
fn route_source_index_visits_each_direct_item_once() {
    const DEPTH: usize = 130;
    const ITEMS_PER_ROUTE: usize = 3;

    let root = unique_case_dir("package-route-source-index-work");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let mut nested = Vec::new();
    for level in (0..DEPTH).rev() {
        let package_root = root.join(format!("route-{level}"));
        let source = package_root.join("src/index.ts");
        nested = vec![route_with_dependencies(
            &package_root,
            vec![source.clone(), source],
            vec![package_root.join("src/dependency.ts")],
            nested,
        )];
    }
    let route = nested.pop().unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    let work = project.package_route_source_index_work_for_test(&route);

    assert_eq!(work.route_visits, DEPTH);
    assert_eq!(work.source_visits, ITEMS_PER_ROUTE * DEPTH);
    assert_eq!(
        work.ancestor_candidates,
        ITEMS_PER_ROUTE * DEPTH * (DEPTH + 1) / 2,
        "each direct item considers its ancestor stack without revisiting descendant suffixes"
    );

    fs::remove_dir_all(&root).ok();
}
