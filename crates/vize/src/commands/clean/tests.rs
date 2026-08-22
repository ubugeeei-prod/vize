use super::{
    CleanArgs, CleanScope, force_vize_artifact_paths, managed_vize_artifact_paths,
    node_modules_vize_artifact_paths, node_modules_vize_dir, project_vize_artifact_paths,
    project_vize_dir, run,
};
use std::path::Path;

#[test]
fn scoped_vize_artifact_dirs_can_target_each_root() {
    assert_eq!(
        project_vize_dir(Path::new("/project")),
        Path::new("/project").join(".vize")
    );
    assert_eq!(
        node_modules_vize_dir(Path::new("/project")),
        Path::new("/project").join("node_modules").join(".vize")
    );
}

#[test]
fn managed_artifact_paths_are_lifecycle_owned_entries() {
    let root = Path::new("/project");
    assert_eq!(
        project_vize_artifact_paths(root),
        vec![
            root.join(".vize/patina"),
            root.join(".vize/reports"),
            root.join(".vize/snapshots"),
            root.join(".vize/tokens"),
            vize_canon::project_virtual_root(root),
            vize_canon::project_virtual_lock_paths(root)[0].clone(),
            vize_canon::project_virtual_lock_paths(root)[1].clone(),
        ]
    );
    assert_eq!(
        node_modules_vize_artifact_paths(root),
        vec![
            root.join("node_modules/.vize/check-profile"),
            root.join("node_modules/.vize/corsa"),
            root.join("node_modules/.vize/corsa-overlay"),
            root.join("node_modules/.vize/lsp.log"),
            root.join("node_modules/.vize/oxc-dumps"),
            root.join("node_modules/.vize/oxlint-plugin-vize"),
            root.join("node_modules/.vize/patina"),
            root.join("node_modules/.vize/vize.config.schema.json"),
            root.join("node_modules/.vize/vize.sock"),
        ]
    );
    assert_eq!(
        managed_vize_artifact_paths(root, CleanScope::All).len(),
        project_vize_artifact_paths(root).len() + node_modules_vize_artifact_paths(root).len()
    );
}

#[test]
fn clean_removes_managed_project_and_node_modules_vize_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let project_artifact = root.join(".vize/patina/session-1-0");
    let node_modules_artifact = vize_canon::project_virtual_root(root);
    std::fs::create_dir_all(&project_artifact).unwrap();
    std::fs::create_dir_all(&node_modules_artifact).unwrap();
    for lock_path in vize_canon::project_virtual_lock_paths(root) {
        std::fs::write(lock_path, "").unwrap();
    }
    std::fs::create_dir_all(root.join("node_modules/.vize")).unwrap();
    std::fs::write(root.join("node_modules/.vize/lsp.log"), "").unwrap();
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    std::fs::write(root.join("node_modules/keep.txt"), "keep").unwrap();

    run(CleanArgs {
        root: root.to_path_buf(),
        scope: CleanScope::All,
        force: false,
        dry_run: false,
        quiet: true,
    });
    assert!(!root.join(".vize").exists());
    assert!(!root.join("node_modules/.vize").exists());
    assert!(root.join("node_modules/keep.txt").exists());
}

#[test]
fn clean_preserves_unrecognized_entries_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let managed_project_artifact = root.join(".vize/reports");
    let unknown_project_artifact = root.join(".vize/custom/keep.txt");
    let managed_node_modules_artifact = vize_canon::project_virtual_root(root);
    let unknown_node_modules_artifact = root.join("node_modules/.vize/custom/keep.txt");
    std::fs::create_dir_all(&managed_project_artifact).unwrap();
    std::fs::create_dir_all(unknown_project_artifact.parent().unwrap()).unwrap();
    std::fs::write(&unknown_project_artifact, "keep").unwrap();
    std::fs::create_dir_all(&managed_node_modules_artifact).unwrap();
    std::fs::create_dir_all(unknown_node_modules_artifact.parent().unwrap()).unwrap();
    std::fs::write(&unknown_node_modules_artifact, "keep").unwrap();

    run(CleanArgs {
        root: root.to_path_buf(),
        scope: CleanScope::All,
        force: false,
        dry_run: false,
        quiet: true,
    });
    assert!(!managed_project_artifact.exists());
    assert!(!managed_node_modules_artifact.exists());
    assert!(unknown_project_artifact.exists());
    assert!(unknown_node_modules_artifact.exists());
}

#[test]
fn project_clean_preserves_a_foreign_canon_namespace_without_force() {
    for scope in [CleanScope::All, CleanScope::Project] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let current = vize_canon::project_virtual_root(root);
        let foreign = current.parent().unwrap().join("foreign-project-key");
        let foreign_lock = foreign.with_extension("lock");
        let foreign_windows_lock = foreign.with_extension("materialize.lock");
        std::fs::create_dir_all(&current).unwrap();
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::write(current.join("current.ts"), "current").unwrap();
        std::fs::write(foreign.join("foreign.ts"), "foreign").unwrap();
        std::fs::write(&foreign_lock, "foreign").unwrap();
        std::fs::write(&foreign_windows_lock, "foreign").unwrap();

        run(CleanArgs {
            root: root.to_path_buf(),
            scope,
            force: false,
            dry_run: false,
            quiet: true,
        });
        assert!(!current.exists(), "current key survived scope={scope:?}");
        assert!(
            foreign.join("foreign.ts").is_file(),
            "scope={scope:?} removed state owned by another project"
        );
        assert!(
            foreign_lock.is_file(),
            "scope={scope:?} removed foreign lock"
        );
        assert!(
            foreign_windows_lock.is_file(),
            "scope={scope:?} removed foreign Windows lock"
        );
    }
}

#[test]
fn force_node_modules_paths_treat_a_missing_vize_directory_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        force_vize_artifact_paths(dir.path(), CleanScope::NodeModules).unwrap(),
        Vec::<std::path::PathBuf>::new()
    );
}

#[test]
fn force_node_modules_paths_reject_a_non_directory_vize_path() {
    let dir = tempfile::tempdir().unwrap();
    let node_modules = dir.path().join("node_modules");
    std::fs::create_dir_all(&node_modules).unwrap();
    std::fs::write(node_modules.join(".vize"), "not a directory").unwrap();

    let error = force_vize_artifact_paths(dir.path(), CleanScope::NodeModules).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotADirectory);
}

#[test]
fn force_clean_removes_selected_artifact_roots() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let unknown_project_artifact = root.join(".vize/custom/keep.txt");
    let unknown_node_modules_artifact = root.join("node_modules/.vize/custom/keep.txt");
    std::fs::create_dir_all(unknown_project_artifact.parent().unwrap()).unwrap();
    std::fs::write(&unknown_project_artifact, "keep").unwrap();
    std::fs::create_dir_all(unknown_node_modules_artifact.parent().unwrap()).unwrap();
    std::fs::write(&unknown_node_modules_artifact, "keep").unwrap();

    run(CleanArgs {
        root: root.to_path_buf(),
        scope: CleanScope::Project,
        force: true,
        dry_run: false,
        quiet: true,
    });
    assert!(!root.join(".vize").exists());
    assert!(unknown_node_modules_artifact.exists());
}

#[test]
fn force_clean_removes_unrecognized_node_modules_entries() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let unknown_node_modules_artifact = root.join("node_modules/.vize/custom/keep.txt");
    std::fs::create_dir_all(unknown_node_modules_artifact.parent().unwrap()).unwrap();
    std::fs::write(&unknown_node_modules_artifact, "keep").unwrap();

    run(CleanArgs {
        root: root.to_path_buf(),
        scope: CleanScope::NodeModules,
        force: true,
        dry_run: false,
        quiet: true,
    });
    assert!(!unknown_node_modules_artifact.exists());
}
