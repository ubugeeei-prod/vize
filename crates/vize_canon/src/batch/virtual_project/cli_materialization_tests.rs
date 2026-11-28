//! CLI topology authority stays local to one completed, locked transaction.

use std::fs;
#[cfg(unix)]
use std::path::PathBuf;

use crate::batch::materialize_lock::MaterializeLock;

use super::VirtualProject;

fn fixture() -> (tempfile::TempDir, VirtualProject) {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.ts"), "export const value = 42;\n").unwrap();
    let package = root.path().join("node_modules/pkg/lib");
    fs::create_dir_all(&package).unwrap();
    fs::write(
        package.parent().unwrap().join("package.json"),
        r#"{"name":"pkg","types":"./lib/index.d.ts"}"#,
    )
    .unwrap();
    fs::write(package.join("index.d.ts"), "export declare const a: 1;\n").unwrap();
    fs::write(package.join("extra.d.ts"), "export declare const b: 2;\n").unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.register_path(&root.path().join("main.ts")).unwrap();
    let declaration = project
        .project_root()
        .join("node_modules/pkg/lib/extra.d.ts");
    project
        .register_declaration_file(&declaration, "export declare const b: 2;\n")
        .unwrap();
    (root, project)
}

#[test]
fn committed_topology_matches_fresh_single_and_shard_identities_without_reprobing() {
    let (_root, project) = fixture();
    let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
    let materialized = project
        .materialize_for_cli(&lock)
        .unwrap()
        .into_build_info();
    let config = project.generated_tsconfig_path();
    let configs = [
        config.clone(),
        config.with_file_name("tsconfig.shard0.json"),
        config.with_file_name("tsconfig.shard1.json"),
        project.virtual_root().join("nested/tsconfig.json"),
    ];
    let fresh = configs
        .iter()
        .map(|config| project.cli_build_info_path(config))
        .collect::<Vec<_>>();
    super::canonical_paths::reset_prep_counters();
    let reused = configs
        .iter()
        .map(|config| materialized.build_info_path(&project, config).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(reused, fresh);
    assert_eq!(super::canonical_paths::prep_counters(), (0, 0));
    let distinct = reused.iter().collect::<vize_carton::FxHashSet<_>>();
    assert_eq!(distinct.len(), configs.len());
    assert_eq!(
        fs::read_to_string(
            project
                .virtual_root()
                .join("node_modules/pkg/lib/index.d.ts")
        )
        .unwrap(),
        "export declare const a: 1;\n"
    );
}

#[test]
fn even_another_project_with_the_same_cache_root_cannot_consume_the_proof() {
    let (root, project) = fixture();
    let other = VirtualProject::new(root.path()).unwrap();
    assert_eq!(project.virtual_root(), other.virtual_root());
    let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
    let materialized = project
        .materialize_for_cli(&lock)
        .unwrap()
        .into_build_info();
    assert_eq!(
        materialized
            .build_info_path(&other, &other.generated_tsconfig_path())
            .unwrap_err()
            .to_string(),
        "IO error: CLI materialization belongs to a different virtual project"
    );
}

#[test]
fn failed_materialization_has_no_reusable_authority_and_repair_can_retry() {
    let (root, mut project) = fixture();
    let original = root.path().join("not-yet-present.css");
    project.passthrough_files.insert(
        project.virtual_root().join("not-yet-present.css"),
        original.clone(),
    );
    let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
    assert!(project.materialize_for_cli(&lock).is_err());
    fs::write(&original, "body { color: red; }\n").unwrap();
    let materialized = project
        .materialize_for_cli(&lock)
        .unwrap()
        .into_build_info();
    let config = project.generated_tsconfig_path();
    assert_eq!(
        materialized.build_info_path(&project, &config).unwrap(),
        project.cli_build_info_path(&config)
    );
}

#[test]
fn shadow_manifest_source_identity_still_participates_without_any_link_change() {
    let (root, mut project) = fixture();
    fs::write(root.path().join("package.json"), r#"{"name":"app"}"#).unwrap();
    let config = project.generated_tsconfig_path();
    let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
    let before = project
        .materialize_for_cli(&lock)
        .unwrap()
        .into_build_info()
        .build_info_path(&project, &config)
        .unwrap();
    project.package_shadow_manifests.insert(
        project.virtual_root().join("shadow/package.json"),
        project.project_root().join("package.json"),
    );
    let materialized = project
        .materialize_for_cli(&lock)
        .unwrap()
        .into_build_info();
    let after = materialized.build_info_path(&project, &config).unwrap();
    assert_eq!(after, project.cli_build_info_path(&config));
    assert_ne!(after, before);
}

#[cfg(unix)]
#[test]
fn an_unchanged_source_symlink_retarget_gets_a_new_transaction_identity() {
    let (_holder, project) = fixture();
    let root = project.project_root();
    let targets: [PathBuf; 2] = [root.join("first"), root.join("second")];
    for target in &targets {
        fs::create_dir_all(target).unwrap();
        fs::write(
            target.join("index.d.ts"),
            "export declare const value: 42;\n",
        )
        .unwrap();
    }
    let targets = targets.map(|target| fs::canonicalize(target).unwrap());
    let dependency = root.join("node_modules/dependency");
    let mirrored = project.virtual_root().join("node_modules/dependency");
    std::os::unix::fs::symlink(&targets[0], &dependency).unwrap();
    // The fixture owns an installed extra.d.ts leaf, forcing a per-entry
    // merge. Assert the actual map rather than assuming a child retarget is
    // visible through an otherwise whole-directory link.
    let original_links = project.desired_package_links(&project.workspace_alias_plan());
    assert_eq!(original_links.get(&mirrored), Some(&targets[0]));
    assert_eq!(
        original_links.get(&project.virtual_root().join("node_modules")),
        None
    );
    let config = project.generated_tsconfig_path();
    let first = {
        let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
        let materialized = project
            .materialize_for_cli(&lock)
            .unwrap()
            .into_build_info();
        let first = materialized.build_info_path(&project, &config).unwrap();
        assert_eq!(first, project.cli_build_info_path(&config));
        first
    };
    fs::remove_file(&dependency).unwrap();
    std::os::unix::fs::symlink(&targets[1], &dependency).unwrap();
    let mut retargeted_links = original_links.clone();
    retargeted_links.insert(mirrored.clone(), targets[1].clone());
    assert_eq!(
        project.desired_package_links(&project.workspace_alias_plan()),
        retargeted_links
    );
    let second = {
        let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
        let materialized = project
            .materialize_for_cli(&lock)
            .unwrap()
            .into_build_info();
        let second = materialized.build_info_path(&project, &config).unwrap();
        assert_eq!(second, project.cli_build_info_path(&config));
        second
    };
    assert_ne!(first, second);
    assert_eq!(
        fs::read(targets[0].join("index.d.ts")).unwrap(),
        fs::read(targets[1].join("index.d.ts")).unwrap()
    );
    fs::remove_file(&dependency).unwrap();
    std::os::unix::fs::symlink(&targets[0], &dependency).unwrap();
    assert_eq!(
        project.desired_package_links(&project.workspace_alias_plan()),
        original_links
    );
    let lock = MaterializeLock::acquire(project.virtual_root()).unwrap();
    let materialized = project
        .materialize_for_cli(&lock)
        .unwrap()
        .into_build_info();
    assert_eq!(
        materialized.build_info_path(&project, &config).unwrap(),
        first
    );
}
