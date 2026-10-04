//! Owned shadow ancestors remain protected after their final route disappears.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{VirtualProject, begin_warm, fixture, load};

#[derive(PartialEq, Eq, Debug)]
struct RawReceipt {
    files: Vec<(PathBuf, Vec<u8>)>,
    links: Vec<(PathBuf, PathBuf)>,
}

fn raw_receipt(root: &Path) -> RawReceipt {
    let files = [
        "packages/a/package.json",
        "packages/a/src/index.ts",
        "packages/b/package.json",
        "packages/b/src/index.ts",
        "packages/c/package.json",
        "packages/c/src/index.ts",
        "packages/c/src/util.ts",
        "packages/c/src/Btn.vue",
    ]
    .into_iter()
    .map(|relative| {
        let path = root.join(relative);
        let bytes = fs::read(&path).unwrap();
        (path, bytes)
    })
    .collect();
    let links = [
        "node_modules/@x/a",
        "node_modules/@x/b",
        "node_modules/@x/c",
        "packages/a/node_modules/@x/b",
        "packages/b/node_modules/@x/c",
    ]
    .into_iter()
    .map(|relative| {
        let path = root.join(relative);
        let target = fs::read_link(&path).unwrap();
        (path, target)
    })
    .collect();
    RawReceipt { files, links }
}

fn assert_raw_unchanged(root: &Path, before: &RawReceipt) {
    assert_eq!(&raw_receipt(root), before);
    for relative in ["packages/c/src/Btn.vue.ts", "packages/c/src/Btn.d.vue.ts"] {
        assert!(
            !root.join(relative).exists(),
            "no generated authored companion"
        );
    }
}

fn retire_routes(project: &mut VirtualProject, root: &Path) {
    let roots = [
        root.join("packages/a/src/index.ts"),
        root.join("packages/b/src/index.ts"),
    ];
    for source in &roots {
        fs::write(source, "export {};\n").unwrap();
        project.register_path(source).unwrap();
    }
    project.reconcile_package_routes_for_importers(&roots);
    project.finalize_package_routes().unwrap();
    assert!(project.workspace_alias_links().is_empty());
    assert!(!project.retired_package_shadow_paths.is_empty());
}

fn assert_declines_without_cleanup(project: &mut VirtualProject, path: &Path, raw: &Path) {
    let previous = project.materialized_package_links.clone();
    let candidates = project.incremental_materialized_candidates.clone();
    let retired = project.retired_package_shadow_paths.clone();
    let error = project.materialize_incremental_delta().unwrap_err();
    assert!(error.to_string().contains("requires owned directories"));
    assert_eq!(fs::read_link(path).unwrap(), raw);
    assert_eq!(project.materialized_package_links, previous);
    assert_eq!(project.incremental_materialized_candidates, candidates);
    assert_eq!(project.retired_package_shadow_paths, retired);
}

#[test]
fn last_alias_owner_cleanup_declines_raw_parent_for_local_and_full_link_patches() {
    for full in [false, true] {
        let temp = fixture(true);
        let root = temp.path().canonicalize().unwrap();
        let mut project = load(&root);
        begin_warm(&mut project);
        let selected = project.virtual_root().join("packages/b/node_modules/@x/c");
        assert!(
            project
                .materialized_package_links
                .get(&selected)
                .unwrap()
                .starts_with(project.virtual_root())
        );
        retire_routes(&mut project, &root);
        if full {
            project.mark_incremental_link_topology();
        }
        assert_eq!(project.incremental_link_topology_dirty, full);
        assert!(!project.incremental_package_link_scopes.is_empty());
        let parent = selected.parent().unwrap();
        let raw = root.join("packages/b/node_modules/@x");
        let before = raw_receipt(&root);
        fs::remove_dir_all(parent).unwrap();
        std::os::unix::fs::symlink(&raw, parent).unwrap();
        assert_declines_without_cleanup(&mut project, parent, &raw);
        assert_raw_unchanged(&root, &before);
        assert!(
            project
                .materialize()
                .unwrap_err()
                .to_string()
                .contains("requires owned directories")
        );
        assert_raw_unchanged(&root, &before);
        assert_eq!(fs::read_link(parent).unwrap(), raw);
        fs::remove_file(parent).unwrap();
    }
}

#[test]
fn current_shadow_src_and_preserved_union_src_decline_before_writes_or_gc() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    begin_warm(&mut project);
    let alias = project
        .workspace_alias_links()
        .into_iter()
        .find(|link| {
            project
                .package_shadow_files
                .contains_key(&link.real_dir.join("src/util.ts"))
        })
        .unwrap();
    let src = alias.real_dir.join("src");
    let raw = root.join("packages/c/src");
    let before = raw_receipt(&root);
    let files = project.expected_materialized_files();
    let links = project.desired_package_links();
    let util = project
        .find_by_original(&root.join("packages/c/src/util.ts"))
        .unwrap()
        .virtual_path
        .clone();
    project.mark_package_shadow_source_changed(&util);
    assert!(!project.incremental_materialized_candidates.is_empty());
    fs::remove_dir_all(&src).unwrap();
    std::os::unix::fs::symlink(&raw, &src).unwrap();
    assert_declines_without_cleanup(&mut project, &src, &raw);
    assert_raw_unchanged(&root, &before);
    assert!(
        project
            .materialize()
            .unwrap_err()
            .to_string()
            .contains("requires owned directories")
    );
    assert_raw_unchanged(&root, &before);
    let mut query = VirtualProject::new(&root).unwrap();
    query
        .register_path(&root.join("packages/a/src/index.ts"))
        .unwrap();
    assert!(query.package_shadow_files.is_empty());
    assert!(
        query
            .materialize_editor_union(&files, &links, &[])
            .unwrap_err()
            .to_string()
            .contains("requires owned directories")
    );
    assert_raw_unchanged(&root, &before);
    assert_eq!(fs::read_link(&src).unwrap(), raw);
    fs::remove_file(src).unwrap();
}

#[test]
fn retired_shadow_src_declines_before_file_removal_and_cold_gc() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    begin_warm(&mut project);
    let src = project
        .workspace_alias_links()
        .into_iter()
        .find(|link| {
            project
                .package_shadow_files
                .contains_key(&link.real_dir.join("src/util.ts"))
                && link.real_dir != project.virtual_root().join("packages/c")
        })
        .unwrap()
        .real_dir
        .join("src");
    retire_routes(&mut project, &root);
    assert!(project.package_shadow_files.is_empty());
    assert!(
        project
            .retired_package_shadow_paths
            .iter()
            .any(|path| path.starts_with(&src))
    );
    let raw = root.join("packages/c/src");
    let before = raw_receipt(&root);
    fs::remove_dir_all(&src).unwrap();
    std::os::unix::fs::symlink(&raw, &src).unwrap();
    assert_declines_without_cleanup(&mut project, &src, &raw);
    assert_raw_unchanged(&root, &before);
    assert!(
        project
            .materialize()
            .unwrap_err()
            .to_string()
            .contains("requires owned directories")
    );
    assert_raw_unchanged(&root, &before);
    assert_eq!(fs::read_link(&src).unwrap(), raw);
    fs::remove_file(src).unwrap();
}
