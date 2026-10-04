#![cfg(unix)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use std::{fs, path::Path};

use super::VirtualProject;

const SOURCES: [&str; 5] = [
    "packages/a/src/index.ts",
    "packages/b/src/index.ts",
    "packages/c/src/Btn.vue",
    "packages/c/src/index.ts",
    "packages/c/src/util.ts",
];

fn fixture(pnpm: bool) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/pnpm-workspace-routes");
    copy_tree(&corpus, temp.path());
    let links: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(corpus.join("links.json")).unwrap()).unwrap();
    for group in if pnpm {
        &["root", "pnpm"][..]
    } else {
        &["root"][..]
    } {
        for (path, target) in links[group].as_object().unwrap() {
            let path = temp.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(target.as_str().unwrap(), path).unwrap();
        }
    }
    temp
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &path);
        } else {
            fs::copy(entry.path(), path).unwrap();
        }
    }
}

fn load(root: &Path) -> VirtualProject {
    let mut project = VirtualProject::new(root).unwrap();
    let entry = root.join(SOURCES[0]);
    project.register_path(&entry).unwrap();
    project.reconcile_package_routes_for_importers(&[entry]);
    project.register_package_route_targets().unwrap();
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();
    project.materialize().unwrap();
    project
}

fn assert_complete(project: &VirtualProject, root: &Path) {
    let expected = SOURCES.map(|path| root.join(path));
    assert_eq!(project.registered_original_paths_sorted(), expected);
    let package = root.join("packages/c");
    let indexed = project.package_source_index.get(&package).unwrap();
    for relative in ["src/index.ts", "src/util.ts", "src/Btn.vue"] {
        let original = package.join(relative);
        assert!(indexed.contains_key(&original), "unindexed {original:?}");
        let materialized = project
            .preferred_materialized_path_for_original(&original)
            .unwrap();
        assert!(materialized.is_file(), "missing {materialized:?}");
        assert_eq!(
            project
                .find_by_virtual(&materialized)
                .unwrap()
                .original_path,
            original,
        );
    }
    for (shadow, canonical) in &project.package_shadow_files {
        assert_eq!(
            fs::read_to_string(shadow).unwrap(),
            project
                .package_shadow_content(shadow, canonical)
                .unwrap()
                .as_str(),
            "materialized shadow bytes differ: {shadow:?}",
        );
    }
}

fn mapping_rows(project: &VirtualProject) -> Vec<String> {
    let mut rows = project
        .materialized_source_documents()
        .into_iter()
        .map(|document| {
            format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
                document.materialized_path,
                document.source_path,
                document.source,
                document.code,
                document.mappings,
                document.semantic_links,
                document.import_source_map,
                document.mapping_kind,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

#[test]
fn real_workspace_links_keep_relative_sources_across_route_rebinding() {
    for pnpm in [false, true] {
        let temp = fixture(pnpm);
        let root = temp.path().canonicalize().unwrap();
        let mut project = load(&root);
        assert_complete(&project, &root);
        let before = mapping_rows(&project);
        // Refreshing a sole route owner must not forget its already-registered
        // util.ts or Vue source, which are not package export entry points.
        for binding in project.package_routes_snapshot() {
            project.insert_package_route_binding(binding);
        }
        project.finalize_package_routes().unwrap();
        project.materialize().unwrap();
        assert_complete(&project, &root);
        assert_eq!(mapping_rows(&project), before);

        let importers = SOURCES.map(|path| root.join(path));
        project.reconcile_package_routes_for_importers(&importers);
        project.finalize_package_routes().unwrap();
        project.materialize().unwrap();
        assert_complete(&project, &root);
        assert_eq!(mapping_rows(&project), before);
    }
}

#[test]
fn rebound_topology_does_not_resurrect_deleted_sources_or_removed_package_roots() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    let source = root.join("packages/c/src/Btn.vue");
    fs::remove_file(&source).unwrap();
    project.remove_registered_source(&source);
    for binding in project.package_routes_snapshot() {
        project.insert_package_route_binding(binding);
    }
    assert!(project.find_by_original(&source).is_none());
    assert!(
        project
            .package_source_index
            .values()
            .all(|sources| !sources.contains_key(&source))
    );

    // Removing the final importing edges must release the package's index.
    let a = root.join(SOURCES[0]);
    let b = root.join(SOURCES[1]);
    fs::write(&a, "export {};\n").unwrap();
    fs::write(&b, "export {};\n").unwrap();
    project.register_paths(&[a.clone(), b.clone()]).unwrap();
    project.reconcile_package_routes_for_importers(&[a, b]);
    assert!(
        !project
            .package_route_roots
            .contains_key(&root.join("packages/c"))
    );
    assert!(
        !project
            .package_source_index
            .contains_key(&root.join("packages/c"))
    );
}
