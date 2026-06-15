#[cfg(windows)]
use super::{VirtualProject, unique_case_dir};
#[cfg(windows)]
use std::fs;

#[cfg(windows)]
#[test]
fn virtual_project_new_strips_windows_verbatim_project_root() {
    let case_dir = unique_case_dir("windows-verbatim-root");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();

    let verbatim_root = case_dir.canonicalize().unwrap();
    let project = VirtualProject::new(&verbatim_root).unwrap();

    assert_eq!(
        project.project_root(),
        vize_carton::path::canonicalize_non_verbatim(&case_dir).as_path()
    );
    assert!(
        !project
            .virtual_root()
            .to_string_lossy()
            .starts_with("\\\\?\\")
    );

    let _ = fs::remove_dir_all(&case_dir);
}
