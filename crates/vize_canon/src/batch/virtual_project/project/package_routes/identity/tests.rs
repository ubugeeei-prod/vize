use std::path::Path;

use crate::{PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding};
use vize_carton::FxHashSet;

use super::VirtualProject;

fn binding(
    importer: &Path,
    specifier: &str,
    mode: PackageResolutionMode,
    root: Option<&Path>,
    workspace: bool,
) -> PackageRouteBinding {
    PackageRouteBinding {
        importer_path: importer.to_path_buf(),
        specifier: specifier.into(),
        occurrence_mode: mode,
        context: PackageResolutionContext::default(),
        route: root.map(|root| PackageRoute {
            source_paths: Vec::new(),
            dependency_paths: Vec::new(),
            source_targets: Vec::new(),
            package_root: root.to_path_buf(),
            package_link_root: root.to_path_buf(),
            manifest_path: root.join("package.json"),
            package_name: None,
            workspace_source: workspace,
            nested_routes: Vec::new(),
        }),
        invalidation_paths: Vec::new(),
        shared_invalidation_paths: Vec::new(),
    }
}

fn assert_complete(project: &VirtualProject, importer: &Path) {
    let expected: FxHashSet<_> = project
        .package_routes
        .keys()
        .filter(|key| key.importer_path == importer)
        .cloned()
        .collect();
    let actual = project
        .package_route_importers
        .get(importer)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        actual, expected,
        "positive and negative routes must remain indexed"
    );
}

fn assert_exact(project: &VirtualProject, importer: &Path, specifier: &str) {
    assert_eq!(
        project.workspace_package_route_identity(importer, specifier),
        project.workspace_package_route_identity_scan(importer, specifier),
        "the index must preserve the complete returned identity",
    );
    assert_complete(project, importer);
}

#[test]
fn identity_is_bounded_to_the_importer_without_crossing_package_visibility() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let importer = root.join("app/entry.ts");
    let workspace = root.join("packages/workspace");
    let mut project = VirtualProject::new(root).unwrap();
    let mut bindings = Vec::new();
    for index in 0..256 {
        let other = root
            .join("neighbors")
            .join(vize_carton::cstr!("{index}").as_str())
            .join("entry.ts");
        bindings.push(binding(
            &other,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&workspace),
            true,
        ));
    }
    bindings.push(binding(
        &importer,
        "@scope/workspace",
        PackageResolutionMode::Import,
        Some(&workspace),
        true,
    ));
    bindings.push(binding(
        &importer,
        "@scope/installed",
        PackageResolutionMode::Import,
        Some(&workspace),
        false,
    ));
    project.replace_package_route_bindings(bindings);
    assert_eq!(
        project
            .package_route_importers
            .get(&importer)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        Some(workspace.as_path())
    );
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/installed"),
        None
    );
    for specifier in ["@scope/workspace", "@scope/installed", "@scope/missing"] {
        assert_exact(&project, &importer, specifier);
    }
    assert_exact(&project, &root.join("unowned/entry.ts"), "@scope/workspace");
}

#[test]
fn negative_late_add_unlink_and_mode_replacement_keep_the_importer_index_complete() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let importer = root.join("app/entry.ts");
    let workspace = root.join("packages/first");
    let other = root.join("packages/second");
    let mut project = VirtualProject::new(root).unwrap();
    let negative = binding(
        &importer,
        "@scope/workspace",
        PackageResolutionMode::Import,
        None,
        true,
    );
    project.replace_package_route_bindings([negative]);
    assert_exact(&project, &importer, "@scope/workspace");
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        None
    );

    let positive = binding(
        &importer,
        "@scope/workspace",
        PackageResolutionMode::Import,
        Some(&workspace),
        true,
    );
    project.insert_package_route_binding(positive.clone());
    assert_exact(&project, &importer, "@scope/workspace");
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        Some(workspace.as_path())
    );

    let require = binding(
        &importer,
        "@scope/workspace",
        PackageResolutionMode::Require,
        Some(&workspace),
        true,
    );
    project.insert_package_route_binding(require.clone());
    assert_exact(&project, &importer, "@scope/workspace");
    project.remove_package_route_binding(&positive.key());
    assert_exact(&project, &importer, "@scope/workspace");
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        Some(workspace.as_path())
    );

    let replacement = binding(
        &importer,
        "@scope/workspace",
        PackageResolutionMode::Require,
        Some(&other),
        true,
    );
    project.insert_package_route_binding(replacement.clone());
    assert_exact(&project, &importer, "@scope/workspace");
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        Some(other.as_path())
    );
    project.remove_package_route_binding(&replacement.key());
    assert_exact(&project, &importer, "@scope/workspace");
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        None
    );
}

#[test]
fn ambiguous_mode_roots_use_the_original_exact_first_choice() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let importer = root.join("app/entry.ts");
    let mut project = VirtualProject::new(root).unwrap();
    let first = root.join("packages/import");
    let second = root.join("packages/require");
    project.replace_package_route_bindings([
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&first),
            true,
        ),
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Require,
            Some(&second),
            true,
        ),
    ]);
    let actual = project
        .workspace_package_route_identity(&importer, "@scope/workspace")
        .unwrap();
    let previous = project
        .workspace_package_route_identity_scan(&importer, "@scope/workspace")
        .unwrap();
    assert!(
        std::ptr::eq(actual, previous),
        "ambiguous occurrences must return the original binding's path"
    );
    assert_complete(&project, &importer);

    // Path equality normalizes trailing '/.', but display() preserves it.
    // A shard key must keep the original complete returned spelling.
    let lexical_variant = first.join(".");
    assert_eq!(first, lexical_variant);
    assert_ne!(first.as_os_str(), lexical_variant.as_os_str());
    project.replace_package_route_bindings([
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&first),
            true,
        ),
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Require,
            Some(&lexical_variant),
            true,
        ),
    ]);
    let actual = project
        .workspace_package_route_identity(&importer, "@scope/workspace")
        .unwrap();
    let previous = project
        .workspace_package_route_identity_scan(&importer, "@scope/workspace")
        .unwrap();
    assert_eq!(actual.as_os_str(), previous.as_os_str());
    assert!(
        std::ptr::eq(actual, previous),
        "component-equal but byte-different identities must fall back"
    );
}

#[test]
fn replacing_duplicate_and_reduced_bindings_never_keeps_a_removed_owner() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let importer = root.join("app/entry.ts");
    let removed = root.join("other/entry.ts");
    let first = root.join("packages/first");
    let second = root.join("packages/second");
    let mut project = VirtualProject::new(root).unwrap();
    project.replace_package_route_bindings([
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&first),
            true,
        ),
        binding(
            &removed,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&first),
            true,
        ),
    ]);
    project.replace_package_route_bindings([
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&first),
            true,
        ),
        binding(
            &importer,
            "@scope/workspace",
            PackageResolutionMode::Import,
            Some(&second),
            true,
        ),
    ]);
    assert_exact(&project, &importer, "@scope/workspace");
    assert_exact(&project, &removed, "@scope/workspace");
    assert_eq!(
        project.workspace_package_route_identity(&importer, "@scope/workspace"),
        Some(second.as_path())
    );
    assert_eq!(
        project.workspace_package_route_identity(&removed, "@scope/workspace"),
        None
    );
    project.clear_package_route_bindings_for_rebuild();
    assert_exact(&project, &importer, "@scope/workspace");
}
