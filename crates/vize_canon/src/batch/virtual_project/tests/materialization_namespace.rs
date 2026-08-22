use std::fs;

use super::{VirtualProject, unique_case_dir};

#[test]
fn materialize_stays_out_of_project_node_modules_when_install_is_absent() {
    let case_dir = unique_case_dir("materialize-without-node-modules");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let main_path = src_dir.join("main.ts");
    fs::write(&main_path, "export const answer = 42;\n").unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&main_path).unwrap();
    project.materialize().unwrap();

    assert!(
        !case_dir.join("node_modules").exists(),
        "batch materialization must not create a project-level node_modules"
    );
    assert!(
        project
            .virtual_root()
            .starts_with(case_dir.join(".vize/canon"))
    );
    assert!(
        project
            .virtual_root()
            .join("node_modules/vue/index.d.ts")
            .exists()
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn materialize_uses_git_storage_without_dirtying_the_checkout() {
    let case_dir = unique_case_dir("materialize-git-storage");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".git")).unwrap();
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let main_path = src_dir.join("main.ts");
    fs::write(&main_path, "export const answer = 42;\n").unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&main_path).unwrap();
    project.materialize().unwrap();

    assert!(
        !case_dir.join("node_modules").exists(),
        "batch materialization must not create a project-level node_modules"
    );
    assert!(
        !case_dir.join(".vize").exists(),
        "Git checkouts must stay clean after batch materialization"
    );
    assert!(
        project
            .virtual_root()
            .starts_with(case_dir.join(".git/vize/canon"))
    );
    assert!(
        project
            .virtual_root()
            .join("node_modules/vue/index.d.ts")
            .exists()
    );

    let _ = fs::remove_dir_all(&case_dir);
}
