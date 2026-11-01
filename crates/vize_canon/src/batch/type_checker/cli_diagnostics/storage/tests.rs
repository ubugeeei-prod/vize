use super::*;
use std::fs;
use tempfile::TempDir;

fn project(owner: &TempDir) -> VirtualProject {
    let root = owner.path().join("source");
    fs::create_dir_all(&root).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.scope_batch_namespace(&owner.path().join("ordinary-storage"));
    project
}

#[test]
fn fresh_cli_storage_does_not_alias_an_independent_default_checker() {
    let owner = TempDir::new().unwrap();
    let mut cli = project(&owner);
    let default = project(&owner);
    let ordinary = default.virtual_root().to_path_buf();
    let cli_key = purpose_key(&cli, None);
    assert_ne!(cli_key, cli.installed_storage_key());
    assert_eq!(cli_key.len(), 64);
    let storage = claim(&mut cli, None).unwrap().unwrap();
    assert_ne!(cli.virtual_root(), ordinary);
    assert!(cli.virtual_root().starts_with(storage.path()));
    assert_eq!(default.virtual_root(), ordinary);
    assert_eq!(default.virtual_files_sorted().len(), 0);
}

#[test]
fn concurrent_cli_owners_keep_distinct_roots_and_live_files() {
    let owner = TempDir::new().unwrap();
    let mut first = project(&owner);
    let mut second = project(&owner);
    let first_storage = claim(&mut first, None).unwrap().unwrap();
    fs::create_dir_all(first.virtual_root()).unwrap();
    let live = first.virtual_root().join("live.d.ts");
    fs::write(&live, b"export {};\n").unwrap();
    let second_storage = claim(&mut second, None).unwrap().unwrap();
    assert_ne!(first_storage.path(), second_storage.path());
    assert_ne!(first.virtual_root(), second.virtual_root());
    drop(second_storage);
    assert_eq!(fs::read(&live).unwrap(), b"export {};\n");
    drop(first_storage);
    assert!(!live.exists());
}

#[test]
fn terminal_owner_retires_its_files_but_preserves_regular_build_info() {
    let owner = TempDir::new().unwrap();
    let mut cli = project(&owner);
    let storage = claim(&mut cli, None).unwrap().unwrap();
    let info_dir = cli.batch_build_info_dir(storage.path());
    fs::create_dir_all(&info_dir).unwrap();
    let build_info = info_dir.join("program.tsbuildinfo");
    fs::write(&build_info, b"retained incremental state").unwrap();
    let sidecar = cli.virtual_root().join("retired-helper.d.ts");
    fs::write(&sidecar, b"export {};\n").unwrap();
    drop(storage);
    assert!(!sidecar.exists());
    assert_eq!(fs::read(build_info).unwrap(), b"retained incremental state");
}

#[test]
fn registration_and_editor_owners_never_move_into_the_cli_namespace() {
    let owner = TempDir::new().unwrap();
    let mut registered = project(&owner);
    let source = registered.project_root().join("input.ts");
    fs::write(&source, b"export const value: number = 1;\n").unwrap();
    registered.register_ts_file(&source).unwrap();
    let original_root = registered.virtual_root().to_path_buf();
    let original = registered.virtual_files_sorted()[0].content.clone();
    assert!(claim(&mut registered, None).unwrap().is_none());
    assert_eq!(registered.virtual_root(), original_root);
    assert_eq!(registered.virtual_files_sorted()[0].content, original);

    let mut editor = project(&owner);
    editor.set_editor_document_options(Default::default());
    let original_root = editor.virtual_root().to_path_buf();
    assert!(claim(&mut editor, None).unwrap().is_none());
    assert_eq!(editor.virtual_root(), original_root);
}

#[test]
fn sequential_expanded_owners_keep_separate_incremental_state() {
    let owner = TempDir::new().unwrap();
    let first_root = owner.path().join("owner-a");
    let second_root = owner.path().join("owner-b");
    fs::create_dir_all(&first_root).unwrap();
    fs::create_dir_all(&second_root).unwrap();
    let first_config = first_root.join("tsconfig.json");
    let second_config = second_root.join("tsconfig.json");
    fs::write(&first_config, b"{}").unwrap();
    fs::write(&second_config, b"{}").unwrap();
    let a = PersistentStorageOwner::capture(&first_root, Some(&first_config));
    let b = PersistentStorageOwner::capture(&second_root, Some(&second_config));
    let changed_config = PersistentStorageOwner::capture(&first_root, Some(&second_config));
    let mut first = project(&owner);
    assert_ne!(purpose_key(&first, Some(&a)), purpose_key(&first, Some(&b)));
    assert_ne!(
        purpose_key(&first, Some(&a)),
        purpose_key(&first, Some(&changed_config))
    );
    let storage = claim(&mut first, Some(&a)).unwrap().unwrap();
    let first_virtual_root = first.virtual_root().to_path_buf();
    let info = first
        .batch_build_info_dir(storage.path())
        .join("owner.tsbuildinfo");
    fs::create_dir_all(info.parent().unwrap()).unwrap();
    fs::write(&info, b"owner-a-only").unwrap();
    drop(storage);

    let mut second = project(&owner);
    let storage = claim(&mut second, Some(&b)).unwrap().unwrap();
    assert_ne!(second.virtual_root(), first_virtual_root);
    assert!(
        !second
            .batch_build_info_dir(storage.path())
            .join("owner.tsbuildinfo")
            .exists()
    );
    assert_eq!(fs::read(&info).unwrap(), b"owner-a-only");
    drop(storage);

    let mut first_again = project(&owner);
    let _storage = claim(&mut first_again, Some(&a)).unwrap().unwrap();
    assert_eq!(first_again.virtual_root(), first_virtual_root);
    assert_eq!(fs::read(info).unwrap(), b"owner-a-only");
}
