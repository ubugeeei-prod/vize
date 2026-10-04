//! Actual link/mapping custody across persistent workspace edits.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{VirtualProject, copy_tree, fixture, load, mapping_rows};

#[path = "pnpm_workspace_alias_filesystem.rs"]
mod filesystem;

#[path = "pnpm_workspace_shadow_cleanup.rs"]
mod cleanup;

pub(super) fn begin_warm(project: &mut VirtualProject) {
    project.capture_materialized_package_links();
    project.discard_incremental_materialization();
}

#[derive(Debug, PartialEq, Eq)]
struct DiskReceipt {
    files: Vec<(PathBuf, Vec<u8>)>,
    links: Vec<(PathBuf, PathBuf, PathBuf)>,
}

fn disk_receipt(project: &VirtualProject) -> DiskReceipt {
    assert_eq!(
        project.materialized_package_links,
        project.desired_package_links()
    );
    let mut files = project
        .expected_materialized_files()
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    let mut links = project
        .materialized_package_links
        .iter()
        .map(|(path, target)| {
            let raw = fs::read_link(path).unwrap();
            let actual = path.canonicalize().unwrap();
            assert_eq!(
                actual,
                target.canonicalize().unwrap(),
                "actual package target {path:?}"
            );
            (path.clone(), raw, actual)
        })
        .collect::<Vec<_>>();
    files.sort();
    links.sort();
    DiskReceipt { files, links }
}

pub(super) fn assert_warm_matches_cold(project: &mut VirtualProject, root: &Path) {
    // Capture physical bytes/links BEFORE a full materialize can repair them.
    let warm = disk_receipt(project);
    let noop = project.materialize_incremental_delta().unwrap();
    assert!(noop.delta.is_empty());
    assert_eq!(noop.considered, 0, "no claim/refcount dirtiness may leak");
    assert_eq!(disk_receipt(project), warm);
    let rows = mapping_rows(project);
    let mut cold = load(root);
    cold.capture_materialized_package_links();
    assert_eq!(rows, mapping_rows(&cold));
    let cold_disk = disk_receipt(&cold);
    assert_eq!(warm.links, cold_disk.links);
    assert_eq!(warm.files.len(), cold_disk.files.len());
    for ((path, bytes), (cold_path, cold_bytes)) in warm.files.iter().zip(&cold_disk.files) {
        assert_eq!(path, cold_path);
        assert_eq!(bytes, cold_bytes, "physical bytes differ at {path:?}");
    }
}

fn refresh(project: &mut VirtualProject, changed: &Path) {
    let keys = project.package_route_keys_for_changes(&[changed.to_path_buf()]);
    assert!(!keys.is_empty(), "the selected link/source must be watched");
    project.refresh_package_route_keys(keys);
    project.register_package_route_targets().unwrap();
}

fn assert_alias_custody(project: &VirtualProject) {
    let aliases = project.workspace_alias_links();
    assert!(
        !aliases.is_empty(),
        "real pnpm scopes must use owned aliases"
    );
    let expected = project.expected_materialized_files();
    for link in aliases {
        assert!(link.real_dir.starts_with(project.virtual_root()));
        assert_eq!(
            link.virtual_dir.canonicalize().unwrap(),
            link.real_dir.canonicalize().unwrap()
        );
        assert!(link.virtual_dir.is_symlink());
        assert!(!expected.contains(&link.virtual_dir.join("package.json")));
        assert!(
            !project
                .package_shadow_files
                .keys()
                .any(|path| path.starts_with(&link.virtual_dir))
        );
        assert!(
            project
                .package_shadow_manifests
                .contains_key(&link.real_dir.join("package.json"))
        );
    }
}

#[test]
fn warm_source_edits_and_new_relative_files_match_full_cold_bytes_maps_and_links() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    assert_alias_custody(&project);
    begin_warm(&mut project);
    let button = root.join("packages/c/src/Btn.vue");
    fs::write(&button, "<script setup lang=\"ts\">const count: number = 2</script>\n<template>{{ count }}</template>\n").unwrap();
    project.register_path(&button).unwrap();
    refresh(&mut project, &button);
    project.finalize_package_routes().unwrap();
    let patch = project.materialize_incremental_delta().unwrap();
    assert!(!patch.delta.is_empty());
    assert_alias_custody(&project);
    assert_warm_matches_cold(&mut project, &root);
    begin_warm(&mut project);

    let util = root.join("packages/c/src/util.ts");
    fs::write(
        &util,
        "export { default as Btn } from './Btn.vue';\nexport { value } from './value';\n",
    )
    .unwrap();
    let value = root.join("packages/c/src/value.ts");
    fs::write(&value, "export const value = 42;\n").unwrap();
    project.register_path(&util).unwrap();
    refresh(&mut project, &util);
    project
        .register_reachable_dependencies_from(&[util])
        .unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize_incremental_delta().unwrap();
    assert_alias_custody(&project);
    assert_warm_matches_cold(&mut project, &root);
    assert!(project.registered_original_paths_sorted().contains(&value));
    for link in project.workspace_alias_links() {
        let value = link.virtual_dir.join("src/value.ts");
        assert_eq!(
            fs::read_to_string(value).unwrap(),
            "export const value = 42;\n"
        );
    }
}

fn assert_b_routes_to(project: &VirtualProject, root: &Path, manifest: &Path) {
    for document in project.materialized_source_documents() {
        if document.source_path != root.join("packages/b/src/index.ts") {
            continue;
        }
        let target = document
            .materialized_path
            .parent()
            .unwrap()
            .ancestors()
            .map(|dir| dir.join("node_modules/@x/c/package.json"))
            .find(|path| path.is_file())
            .unwrap()
            .canonicalize()
            .unwrap();
        assert!(target.starts_with(project.virtual_root()));
        assert_eq!(
            project.package_shadow_manifests.get(&target),
            Some(&manifest.to_path_buf())
        );
    }
}

#[test]
fn retargeted_real_link_and_shared_scope_drift_keep_unchanged_parent_owners_current() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let variant = root.join("packages/c-variant");
    copy_tree(&root.join("packages/c"), &variant);
    fs::write(
        variant.join("src/Btn.vue"),
        "<template><button data-variant=\"second\" /></template>\n",
    )
    .unwrap();
    let provider = variant.join("node_modules/__vize_pnpm_fixture_provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        "{\"name\":\"__vize_pnpm_fixture_provider\",\"types\":\"./index.d.ts\"}\n",
    )
    .unwrap();
    fs::write(
        provider.join("index.d.ts"),
        "export declare const authority: \"second\";\n",
    )
    .unwrap();
    fs::write(variant.join("src/index.ts"), "export * from './util';\nimport { authority } from '__vize_pnpm_fixture_provider';\nconst providerMustBeTyped: 'second' = authority;\nvoid providerMustBeTyped;\n").unwrap();
    fs::write(
        root.join("packages/a/src/index.ts"),
        "export * from '@x/b';\nexport { Btn as DirectBtn } from '@x/c';\n",
    )
    .unwrap();
    let mut project = load(&root);
    assert_alias_custody(&project);
    begin_warm(&mut project);
    let link = root.join("packages/b/node_modules/@x/c");
    fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink("../../../c-variant", &link).unwrap();
    refresh(&mut project, &link);
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize_incremental_delta().unwrap();
    assert_b_routes_to(&project, &root, &variant.join("package.json"));
    for (shadow, original) in &project.package_shadow_manifests {
        if original == &variant.join("package.json") {
            assert_eq!(
                fs::read_to_string(
                    shadow
                        .parent()
                        .unwrap()
                        .join("node_modules/__vize_pnpm_fixture_provider/index.d.ts")
                )
                .unwrap(),
                "export declare const authority: \"second\";\n"
            );
        }
    }
    assert_warm_matches_cold(&mut project, &root);
    begin_warm(&mut project);

    fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink("../../../c", &link).unwrap();
    refresh(&mut project, &link);
    // Registration ownership belongs to the caller; retire the unreachable
    // variant sources as an editor's persistent dependency owner does.
    for source in project.registered_original_paths_sorted() {
        if source.starts_with(&variant) {
            project.remove_registered_source(&source);
        }
    }
    project.finalize_package_routes().unwrap();
    project.materialize_incremental_delta().unwrap();
    assert_alias_custody(&project);
    assert_b_routes_to(&project, &root, &root.join("packages/c/package.json"));
    assert_warm_matches_cold(&mut project, &root);
    assert_eq!(fs::read_link(&link).unwrap(), Path::new("../../../c"));
}

#[test]
fn editor_union_rejects_conflicting_internal_alias_identity_without_touching_target() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let project = load(&root);
    let link = project.workspace_alias_links().into_iter().next().unwrap();
    let manifest = link.real_dir.join("package.json");
    let bytes = fs::read(&manifest).unwrap();
    let preserved = vize_carton::FxHashMap::from_iter([(
        link.virtual_dir.clone(),
        project.virtual_root().join("packages/c"),
    )]);
    let result =
        project.materialize_editor_union(&vize_carton::FxHashSet::default(), &preserved, &[]);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Conflicting workspace package alias targets")
    );
    assert_eq!(fs::read(manifest).unwrap(), bytes);
    assert_eq!(
        link.virtual_dir.canonicalize().unwrap(),
        link.real_dir.canonicalize().unwrap()
    );
}
