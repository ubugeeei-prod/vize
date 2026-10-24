//! Cached link and editor-union safety before any authored source write.

use std::fs;

use super::super::load_entry;
use super::{VirtualProject, begin_warm, fixture, load};

#[test]
fn stale_target_and_root_scope_links_decline_before_raw_sources_are_touched() {
    for root_scope in [false, true] {
        let temp = fixture(true);
        let root = temp.path().canonicalize().unwrap();
        let mut project = load(&root);
        begin_warm(&mut project);
        let target = project
            .workspace_alias_links()
            .into_iter()
            .next()
            .unwrap()
            .real_dir;
        let path = if root_scope {
            target
                .ancestors()
                .find(|path| path.file_name().is_some_and(|name| name == "node_modules"))
                .unwrap()
                .to_path_buf()
        } else {
            target
        };
        let raw = if root_scope {
            root.join("node_modules")
        } else {
            root.join("packages/c")
        };
        let manifest = root.join("packages/c/package.json");
        let button = root.join("packages/c/src/Btn.vue");
        let before = (fs::read(&manifest).unwrap(), fs::read(&button).unwrap());
        fs::remove_dir_all(&path).unwrap();
        std::os::unix::fs::symlink(&raw, &path).unwrap();
        for error in [
            project.materialize().unwrap_err(),
            project.materialize_incremental_delta().unwrap_err(),
        ] {
            assert_eq!(
                error.to_string(),
                "IO error: Workspace package alias requires owned directories"
            );
            assert_eq!(
                (fs::read(&manifest).unwrap(), fs::read(&button).unwrap()),
                before
            );
            assert_eq!(fs::read_link(&path).unwrap(), raw);
        }
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn editor_union_rejects_raw_parent_link_before_rewriting_real_pnpm_install() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let project = load(&root);
    let alias = project.workspace_alias_links().into_iter().next().unwrap();
    let raw_link = root.join("packages/b/node_modules/@x/c");
    let before = (
        fs::read_link(&raw_link).unwrap(),
        fs::read(root.join("packages/c/package.json")).unwrap(),
    );
    let preserved = vize_carton::FxHashMap::from_iter([(
        alias.virtual_dir.parent().unwrap().to_path_buf(),
        root.join("packages/b/node_modules"),
    )]);
    let error = project
        .materialize_editor_union(&vize_carton::FxHashSet::default(), &preserved, &[], false)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "IO error: Workspace package alias overlaps another package link"
    );
    assert_eq!(
        (
            fs::read_link(&raw_link).unwrap(),
            fs::read(root.join("packages/c/package.json")).unwrap()
        ),
        before
    );
    assert_eq!(
        alias.virtual_dir.canonicalize().unwrap(),
        alias.real_dir.canonicalize().unwrap()
    );
}

#[test]
fn package_import_and_workspace_dependency_share_terminal_alias_targets() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("types")).unwrap();
    fs::write(
        root.join("package.json"),
        r##"{"name":"repro","private":true,"type":"module","imports":{"#app":"./types/app.d.ts"},"dependencies":{"@x/a":"workspace:*"}}"##,
    )
    .unwrap();
    fs::write(
        root.join("types/app.d.ts"),
        "export declare const appName: string;\n",
    )
    .unwrap();
    let entry = root.join("src/main.ts");
    fs::write(
        &entry,
        "import { appName } from '#app';\nimport * as workspace from '@x/a';\nexport { appName, workspace };\n",
    )
    .unwrap();

    let project = load_entry(&root, &entry);
    assert!(
        project
            .find_by_original(&root.join("types/app.d.ts"))
            .is_some()
    );
    assert!(
        project
            .find_by_original(&root.join("packages/c/src/Btn.vue"))
            .is_some()
    );
    let links = project.workspace_alias_links();
    assert!(!links.is_empty());
    let traversed_links = links
        .iter()
        .flat_map(|link| {
            links.iter().filter_map(move |other| {
                (other.virtual_dir != link.virtual_dir
                    && link.real_dir.strip_prefix(&other.virtual_dir).is_ok())
                .then(|| (link.virtual_dir.clone(), other.virtual_dir.clone()))
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        traversed_links,
        Vec::new(),
        "workspace alias targets must not traverse another package link"
    );
    for link in &links {
        assert_eq!(
            link.virtual_dir.canonicalize().unwrap(),
            link.real_dir.canonicalize().unwrap()
        );
    }
}

#[test]
fn committed_raw_parent_bridge_migrates_without_touching_the_real_package() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    begin_warm(&mut project);
    let alias = project.workspace_alias_links().into_iter().next().unwrap();
    let parent = alias.virtual_dir.parent().unwrap().to_path_buf();
    let raw_parent = root.join("packages/b/node_modules").canonicalize().unwrap();
    let raw_link = raw_parent.join("@x/c");
    let before = (
        fs::read_link(&raw_link).unwrap(),
        fs::read(root.join("packages/c/package.json")).unwrap(),
    );
    fs::remove_dir_all(&parent).unwrap();
    std::os::unix::fs::symlink(&raw_parent, &parent).unwrap();
    // Model the old committed bridge, not an unknown on-disk endpoint.
    project
        .materialized_package_links
        .insert(parent.clone(), raw_parent);
    project.mark_incremental_link_topology();
    project.materialize_incremental_delta().unwrap();
    assert!(!parent.is_symlink());
    assert_eq!(
        (
            fs::read_link(&raw_link).unwrap(),
            fs::read(root.join("packages/c/package.json")).unwrap()
        ),
        before
    );
    super::assert_warm_matches_cold(&mut project, &root);
}

#[test]
fn retargeted_committed_parent_declines_internal_cache_target_without_touching_sources() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    begin_warm(&mut project);
    let alias = project.workspace_alias_links().into_iter().next().unwrap();
    let parent = alias.virtual_dir.parent().unwrap().to_path_buf();
    fs::remove_dir_all(&parent).unwrap();
    // A target under the cache root is still untrusted unless it exactly
    // matches the persisted receipt from the completed materialization.
    let unexpected = project.virtual_root().join("uncommitted-package-target");
    std::os::unix::fs::symlink(&unexpected, &parent).unwrap();
    project
        .materialized_package_links
        .insert(parent.clone(), root.join("packages/b/node_modules"));
    let manifest = root.join("packages/c/package.json");
    let before = fs::read(&manifest).unwrap();
    let error = project.materialize_incremental_delta().unwrap_err();
    assert_eq!(
        error.to_string(),
        "IO error: Workspace package alias requires owned directories"
    );
    assert_eq!(fs::read(&manifest).unwrap(), before);
    assert_eq!(fs::read_link(&parent).unwrap(), unexpected);
    fs::remove_file(parent).unwrap();
}

#[test]
fn editor_union_preserves_alias_claims_when_only_the_previous_snapshot_owns_them() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let project = load(&root);
    let files = project.expected_materialized_files();
    let links = project.desired_package_links();
    let aliases = project.workspace_alias_links();
    assert!(!aliases.is_empty());
    let before = aliases
        .iter()
        .map(|alias| {
            (
                alias.virtual_dir.clone(),
                fs::read_link(&alias.virtual_dir).unwrap(),
                fs::read(alias.real_dir.join("package.json")).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let mut query = VirtualProject::new(&root).unwrap();
    query
        .register_path(&root.join("packages/a/src/index.ts"))
        .unwrap();
    assert!(query.workspace_alias_links().is_empty());
    query
        .materialize_editor_union(&files, &links, &[], false)
        .unwrap();
    for (alias, target, manifest) in before {
        assert_eq!(fs::read_link(&alias).unwrap(), target);
        assert_eq!(fs::read(alias.join("package.json")).unwrap(), manifest);
        assert_eq!(
            alias.canonicalize().unwrap(),
            target.canonicalize().unwrap()
        );
    }
}

#[test]
fn separate_projects_replace_only_the_selected_leaf_across_alias_directory_alias() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    fs::write(
        root.join("packages/a/src/index.ts"),
        "export * from '@x/b';\nexport { Btn as DirectBtn } from '@x/c';\n",
    )
    .unwrap();
    let shared = load(&root);
    let shared_build_info = shared.cli_build_info_path(&shared.generated_tsconfig_path());
    let selected = shared.virtual_root().join("packages/b/node_modules/@x/c");
    let shared_target = fs::read_link(&selected).unwrap();
    let selected_target = shared
        .workspace_alias_links()
        .into_iter()
        .find(|link| link.virtual_dir == selected)
        .unwrap()
        .real_dir;
    assert_eq!(shared_target, selected_target);
    let variant = root.join("packages/c-variant");
    super::copy_tree(&root.join("packages/c"), &variant);
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
    fs::write(
        variant.join("src/index.ts"),
        "export * from './util';\nimport { authority } from '__vize_pnpm_fixture_provider';\nconst providerMustBeTyped: 'second' = authority;\nvoid providerMustBeTyped;\n",
    )
    .unwrap();
    let sentinels = ["packages/c", "packages/c-variant"]
        .into_iter()
        .flat_map(|directory| {
            ["package.json", "src/index.ts", "src/util.ts", "src/Btn.vue"]
                .map(|relative| root.join(directory).join(relative))
        })
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    let raw_link = root.join("packages/b/node_modules/@x/c");
    fs::remove_file(&raw_link).unwrap();
    std::os::unix::fs::symlink("../../../c-variant", &raw_link).unwrap();
    let mut divergent = load(&root);
    assert_ne!(
        divergent.cli_build_info_path(&divergent.generated_tsconfig_path()),
        shared_build_info,
        "a divergent package selection must not reuse native incremental state"
    );
    assert!(divergent.materialized_package_links.is_empty());
    assert!(fs::symlink_metadata(&selected).unwrap().is_dir());
    assert!(fs::read_link(&selected).is_err());
    assert_eq!(
        fs::read(selected.join("package.json")).unwrap(),
        fs::read(variant.join("package.json")).unwrap()
    );
    assert_eq!(
        divergent
            .package_shadow_manifests
            .get(&selected.join("package.json")),
        Some(&variant.join("package.json"))
    );
    for (path, bytes) in &sentinels {
        assert_eq!(&fs::read(path).unwrap(), bytes);
    }
    super::begin_warm(&mut divergent);
    super::assert_warm_matches_cold(&mut divergent, &root);

    // A raw, unknown ancestor remains ineligible even for a current owned leaf.
    let parent = selected.parent().unwrap();
    fs::remove_dir_all(parent).unwrap();
    let raw_parent = root.join("packages/b/node_modules/@x");
    std::os::unix::fs::symlink(&raw_parent, parent).unwrap();
    divergent.materialized_package_links.clear();
    assert_eq!(
        divergent.materialize().unwrap_err().to_string(),
        "IO error: Workspace package alias requires owned directories"
    );
    assert_eq!(fs::read_link(parent).unwrap(), raw_parent);
    assert_eq!(
        fs::read_link(&raw_link).unwrap(),
        std::path::Path::new("../../../c-variant")
    );
    for (path, bytes) in &sentinels {
        assert_eq!(&fs::read(path).unwrap(), bytes);
    }
    fs::remove_file(parent).unwrap();

    fs::remove_file(&raw_link).unwrap();
    std::os::unix::fs::symlink("../../../c", &raw_link).unwrap();
    let mut repaired = load(&root);
    assert_eq!(
        repaired.cli_build_info_path(&repaired.generated_tsconfig_path()),
        shared_build_info,
        "the repaired package graph may reuse only its matching clean state"
    );
    assert_eq!(fs::read_link(&selected).unwrap(), shared_target);
    let direct = repaired
        .virtual_root()
        .join("packages/a/src/node_modules/@x/c");
    let nested = repaired
        .virtual_root()
        .join("packages/a/src/node_modules/@x/b/node_modules/@x/c");
    assert_eq!(
        direct.canonicalize().unwrap(),
        nested.canonicalize().unwrap(),
        "the repaired dependency routes must share one TypeScript identity"
    );
    let repaired_aliases = repaired.workspace_alias_links();
    assert!(!repaired_aliases.is_empty());
    for alias in repaired_aliases {
        assert!(
            alias.virtual_dir.is_symlink(),
            "repaired alias stayed a copied directory: {}",
            alias.virtual_dir.display()
        );
        assert_eq!(
            alias.virtual_dir.canonicalize().unwrap(),
            alias.real_dir.canonicalize().unwrap(),
            "repaired alias selected another package identity: {}",
            alias.virtual_dir.display()
        );
    }
    for (path, bytes) in &sentinels {
        assert_eq!(&fs::read(path).unwrap(), bytes);
    }
    super::begin_warm(&mut repaired);
    super::assert_warm_matches_cold(&mut repaired, &root);
}

#[test]
fn sequential_package_selections_reclaim_cached_internal_aliases() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();

    let selected_a = load(&root);
    let virtual_root = selected_a.virtual_root().to_path_buf();
    drop(selected_a);

    let selected_b = load_entry(&root, &root.join("packages/b/src/index.ts"));
    assert_eq!(selected_b.virtual_root(), virtual_root.as_path());
    let selected_b_links = selected_b.workspace_alias_links();
    assert!(selected_b_links.is_empty());
    let button = root.join("packages/c/src/Btn.vue");
    assert_eq!(
        selected_b
            .find_by_original(&button)
            .unwrap()
            .original_path
            .as_path(),
        button.as_path()
    );
    drop(selected_b);

    // A separate `vize check` process starts without the first run's in-memory
    // link receipt, but deliberately reuses the project-root cache on disk.
    let selected_a_again = load_entry(&root, &root.join("packages/a/src/index.ts"));
    assert_eq!(selected_a_again.virtual_root(), virtual_root.as_path());
    assert_eq!(
        selected_a_again
            .find_by_original(&button)
            .unwrap()
            .original_path
            .as_path(),
        button.as_path()
    );
    let selected_a_again_links = selected_a_again.workspace_alias_links();
    assert!(!selected_a_again_links.is_empty());
    for link in selected_a_again_links {
        assert!(link.virtual_dir.is_symlink());
        assert_eq!(
            link.virtual_dir.canonicalize().unwrap(),
            link.real_dir.canonicalize().unwrap()
        );
    }
}

#[test]
fn removing_route_owners_restores_surviving_root_bytes_without_stale_shadow_authority() {
    let temp = fixture(true);
    let root = temp.path().canonicalize().unwrap();
    let mut project = load(&root);
    let util = root.join("packages/c/src/util.ts");
    let raw = fs::read(&util).unwrap();
    let path = project
        .find_by_original(&util)
        .unwrap()
        .virtual_path
        .clone();
    assert!(project.package_shadow_files.contains_key(&path));
    assert_ne!(
        raw,
        project.virtual_files.get(&path).unwrap().content.as_bytes()
    );
    assert_eq!(fs::read(&path).unwrap(), raw);
    super::begin_warm(&mut project);
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
    assert!(!project.package_shadow_files.contains_key(&path));
    project.materialize_incremental_delta().unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        project.virtual_files.get(&path).unwrap().content.as_bytes()
    );
    assert_eq!(fs::read(&util).unwrap(), raw);
    let warm = super::disk_receipt(&project);
    let noop = project.materialize_incremental_delta().unwrap();
    assert!(noop.delta.is_empty());
    assert_eq!(noop.considered, 0);
    assert_eq!(super::disk_receipt(&project), warm);
    // The caller retains these registered diagnostic roots after route removal.
    let registered = project.registered_original_paths_sorted();
    let mut cold = VirtualProject::new(&root).unwrap();
    cold.register_paths(&registered).unwrap();
    cold.reconcile_package_routes_for_importers(&registered);
    cold.register_package_route_targets().unwrap();
    cold.finalize_package_routes().unwrap();
    cold.materialize().unwrap();
    cold.capture_materialized_package_links();
    assert_eq!(super::mapping_rows(&project), super::mapping_rows(&cold));
    assert_eq!(super::disk_receipt(&cold), warm);
}
