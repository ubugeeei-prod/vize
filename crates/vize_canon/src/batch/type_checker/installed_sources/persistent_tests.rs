use super::{BatchTypeChecker, BatchTypeCheckerOptions};
use crate::batch::{
    persistent_batch_storage_root, persistent_batch_virtual_root, project_virtual_root,
};

#[test]
fn explicit_persistent_owner_reuses_installed_files_after_drop_without_changing_originals() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let original = root.join("node_modules/package/index.ts");
    std::fs::create_dir_all(original.parent().unwrap()).unwrap();
    std::fs::write(&original, b"export const value: number = 1;\n").unwrap();
    let original_modified = std::fs::metadata(&original).unwrap().modified().unwrap();
    let binary = root.join("unused-tsgo");
    std::fs::write(&binary, "unused fixture executable").unwrap();
    let expected = persistent_batch_virtual_root(&root, &root, None);
    let file = expected.join("node_modules/package/index.ts");
    let mut previous_mtime = None;
    for _ in 0..2 {
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            &root,
            BatchTypeCheckerOptions::default(),
            Some(&binary),
        )
        .unwrap();
        checker.set_persistent_storage_owner(&root, None).unwrap();
        checker.scan_paths(std::slice::from_ref(&original)).unwrap();
        assert!(checker.owned_storage.is_none());
        assert_eq!(checker.virtual_root(), expected);
        assert_ne!(checker.virtual_root(), project_virtual_root(&root));
        checker.project.materialize().unwrap();
        let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();
        if let Some(previous) = previous_mtime {
            assert_eq!(mtime, previous);
        }
        previous_mtime = Some(mtime);
        assert_eq!(
            std::fs::read(&file).unwrap(),
            b"export const value: number = 1;\n"
        );
        assert_eq!(
            std::fs::metadata(&original).unwrap().modified().unwrap(),
            original_modified
        );
    }
    assert!(file.is_file(), "persistent files survive the checker owner");
    std::fs::remove_dir_all(persistent_batch_storage_root(&root)).unwrap();
}
