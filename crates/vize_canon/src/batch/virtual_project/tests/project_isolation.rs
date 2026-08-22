#![cfg(unix)]

use super::{VirtualProject, fs, unique_case_dir};

#[test]
fn virtual_project_uses_a_full_project_key() {
    let case_dir = unique_case_dir("new");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();

    let project = VirtualProject::new(&case_dir).unwrap();
    assert_eq!(project.project_root(), case_dir.as_path());
    assert_eq!(
        project.virtual_root().parent().unwrap().file_name(),
        Some(std::ffi::OsStr::new("projects"))
    );
    assert_eq!(
        project.virtual_root().file_name().unwrap().len(),
        64,
        "the project namespace uses the full SHA-256 identity"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn distinct_projects_sharing_node_modules_have_distinct_virtual_roots() {
    let case_dir = unique_case_dir("shared-node-modules-project-identity");
    let _ = fs::remove_dir_all(&case_dir);
    let first_root = case_dir.join("first");
    let second_root = case_dir.join("second");
    let shared_node_modules = case_dir.join("shared-node-modules");
    fs::create_dir_all(&first_root).unwrap();
    fs::create_dir_all(&second_root).unwrap();
    fs::create_dir_all(&shared_node_modules).unwrap();
    std::os::unix::fs::symlink(&shared_node_modules, first_root.join("node_modules")).unwrap();
    std::os::unix::fs::symlink(&shared_node_modules, second_root.join("node_modules")).unwrap();

    let first_source = first_root.join("src/App.vue");
    let second_source = second_root.join("src/App.vue");
    fs::create_dir_all(first_source.parent().unwrap()).unwrap();
    fs::create_dir_all(second_source.parent().unwrap()).unwrap();
    fs::write(
        &first_source,
        "<script setup lang=\"ts\">const owner = 'first'</script>",
    )
    .unwrap();
    fs::write(
        &second_source,
        "<script setup lang=\"ts\">const owner = 'second'</script>",
    )
    .unwrap();

    let mut first = VirtualProject::new(&first_root).unwrap();
    first.register_path(&first_source).unwrap();
    first.materialize().unwrap();
    let first_virtual_source = first.virtual_root().join("src/App.vue.ts");
    let first_materialized = fs::read_to_string(&first_virtual_source).unwrap();
    let first_crash_residue = first.virtual_root().join("crash-residue.ts");
    fs::write(&first_crash_residue, "stale crashed process state").unwrap();

    let mut second = VirtualProject::new(&second_root).unwrap();
    second.register_path(&second_source).unwrap();
    second.materialize().unwrap();
    let second_virtual_source = second.virtual_root().join("src/App.vue.ts");
    let second_materialized = fs::read_to_string(&second_virtual_source).unwrap();

    let canonical_storage = fs::canonicalize(&shared_node_modules).unwrap();
    assert!(!first.virtual_root().starts_with(&canonical_storage));
    assert!(!second.virtual_root().starts_with(&canonical_storage));
    assert!(
        first
            .virtual_root()
            .starts_with(first_root.join(".vize/canon"))
    );
    assert!(
        second
            .virtual_root()
            .starts_with(second_root.join(".vize/canon"))
    );
    assert_ne!(
        first.virtual_root(),
        second.virtual_root(),
        "physical project identity must not collapse when dependency storage is shared"
    );
    assert_eq!(
        fs::read_to_string(&first_virtual_source).unwrap(),
        first_materialized,
        "materializing the second project must not overwrite the first project's state"
    );
    assert!(
        first_crash_residue.exists(),
        "another project must not prune crash residue from a namespace it does not own"
    );
    assert!(
        !second.virtual_root().join("crash-residue.ts").exists(),
        "another project's crash residue must never appear in this project"
    );
    assert_ne!(
        second_materialized, first_materialized,
        "the projects must retain their incompatible virtual sources"
    );

    first.materialize().unwrap();
    assert!(
        !first_crash_residue.exists(),
        "a project must prune its own stale crash residue on the next materialization"
    );
    assert_eq!(
        fs::read_to_string(&second_virtual_source).unwrap(),
        second_materialized,
        "pruning one project must not mutate another project's generated state"
    );

    let _ = fs::remove_dir_all(&case_dir);
}
