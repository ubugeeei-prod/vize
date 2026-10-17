//! An owned alias replaces a previous member's ordinary external link.

use std::{fs, path::Path};

use vize_carton::{FxHashMap, FxHashSet};

use super::super::VirtualProject;

fn fixture() -> (tempfile::TempDir, VirtualProject) {
    fn copy_tree(source: &Path, target: &Path) {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let target = target.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/pnpm-workspace-routes");
    copy_tree(&corpus, &root);
    let links: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(corpus.join("links.json")).unwrap()).unwrap();
    for group in ["root", "pnpm"] {
        for (path, target) in links[group].as_object().unwrap() {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(target.as_str().unwrap(), path).unwrap();
        }
    }
    let mut project = VirtualProject::new(&root).unwrap();
    let entry = root.join("packages/a/src/index.ts");
    project.register_path(&entry).unwrap();
    project.reconcile_package_routes_for_importers(&[entry]);
    project.register_package_route_targets().unwrap();
    for round in 0..16 {
        let before = project.registered_original_paths_sorted();
        project.register_reachable_dependencies().unwrap();
        let importers = project.registered_original_paths_sorted();
        project.reconcile_package_routes_for_importers(&importers);
        project.register_package_route_targets().unwrap();
        if project.registered_original_paths_sorted() == before {
            break;
        }
        assert!(round < 15, "fixture preparation must converge");
    }
    project.finalize_package_routes().unwrap();
    project.materialize().unwrap();
    (temp, project)
}

#[test]
fn editor_union_replaces_external_link_with_owned_workspace_alias() {
    let (temp, mut project) = fixture();
    let root = temp.path().canonicalize().unwrap();
    let alias = project.workspace_alias_links().into_iter().next().unwrap();
    let external = project.package_shadow_manifests[&alias.real_dir.join("package.json")]
        .parent()
        .unwrap()
        .to_path_buf();
    let manifest = external.join("package.json");
    let component = root.join("packages/c/src/Btn.vue");
    let before = (fs::read(&manifest).unwrap(), fs::read(&component).unwrap());

    // Model a completed member that linked the installed package before a
    // second member discovered the owned workspace topology. The new alias
    // owns no bytes at its link path; its manifest lives at the private target.
    let files = project.expected_materialized_files();
    assert!(
        !files
            .iter()
            .any(|path| path.starts_with(&alias.virtual_dir))
    );
    assert!(files.contains(&alias.real_dir.join("package.json")));
    assert!(
        project
            .workspace_alias_claims()
            .contains(&alias.virtual_dir.join("package.json"))
    );
    fs::remove_file(&alias.virtual_dir).unwrap();
    std::os::unix::fs::symlink(&external, &alias.virtual_dir).unwrap();
    project
        .materialized_package_links
        .insert(alias.virtual_dir.clone(), external.clone());
    let preserved = FxHashMap::from_iter([(alias.virtual_dir.clone(), external)]);

    let snapshot = project
        .materialize_editor_union(&FxHashSet::default(), &preserved, &[], false)
        .unwrap();
    assert_eq!(
        snapshot.package_links().get(&alias.virtual_dir),
        Some(&alias.real_dir)
    );
    assert_eq!(fs::read_link(&alias.virtual_dir).unwrap(), alias.real_dir);
    assert_eq!(
        (fs::read(&manifest).unwrap(), fs::read(&component).unwrap()),
        before
    );
    assert_eq!(
        fs::read_link(root.join("packages/b/node_modules/@x/c")).unwrap(),
        Path::new("../../../c")
    );
}

#[test]
fn editor_union_still_rejects_distinct_owned_alias_targets() {
    let (_temp, project) = fixture();
    let alias = project.workspace_alias_links().into_iter().next().unwrap();
    let other_target = project.virtual_root().join("packages/b");
    let other_manifest = other_target.join("package.json");
    let before = fs::read(&other_manifest).unwrap();
    assert_ne!(other_target, alias.real_dir);
    assert!(
        project
            .expected_materialized_files()
            .contains(&other_manifest)
    );
    let preserved = FxHashMap::from_iter([(alias.virtual_dir.clone(), other_target)]);
    let error = project
        .materialize_editor_union(&FxHashSet::default(), &preserved, &[], false)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Conflicting workspace package alias targets in editor union")
    );
    assert_eq!(fs::read(other_manifest).unwrap(), before);
    assert_eq!(fs::read_link(alias.virtual_dir).unwrap(), alias.real_dir);
}
