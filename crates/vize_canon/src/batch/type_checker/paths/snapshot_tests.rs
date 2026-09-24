use std::fs;

use super::{IncrementalPaths, VirtualProject};

#[test]
fn source_stamps_describe_registered_bytes_not_a_later_disk_revision() {
    for (name, original, changed) in [
        (
            "dep.ts",
            "export const value = 1;",
            "export const value = 2;",
        ),
        (
            "Child.vue",
            "<template>old</template>",
            "<template>new</template>",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let path = root.join(name);
        fs::write(&path, original).unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&path).unwrap();
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        fs::write(&path, changed).unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(modified))
            .unwrap();
        let mut paths = IncrementalPaths::new();
        paths.after_explicit_scan(&project, std::slice::from_ref(&path));
        assert_eq!(
            paths.effective_changes(&root, std::slice::from_ref(&path)),
            [path]
        );
    }
}

#[test]
fn committing_one_source_cannot_acknowledge_an_unobserved_dependency_edit() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    let a = root.join("a.ts");
    let b = root.join("b.ts");
    fs::write(&a, "export const a = 1;").unwrap();
    fs::write(&b, "export const b = 1;").unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_paths(&[a.clone(), b.clone()]).unwrap();
    let mut paths = IncrementalPaths::new();
    paths.after_explicit_scan(&project, &[a.clone(), b.clone()]);

    fs::write(&a, "export const a = 2;").unwrap();
    project.register_path(&a).unwrap();
    fs::write(&b, "export const b = 2;").unwrap();
    paths.commit_project_snapshot(&project);
    assert_eq!(
        paths.effective_changes(&root, &[a.clone(), b.clone()]),
        std::slice::from_ref(&b)
    );

    project.register_path(&b).unwrap();
    paths.commit_project_snapshot(&project);
    assert_eq!(
        paths.effective_changes(&root, &[a, b]),
        Vec::<std::path::PathBuf>::new()
    );
}

#[test]
fn a_deleted_registered_source_stays_changed_until_removed_from_the_graph() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    let path = root.join("dep.ts");
    fs::write(&path, "export const value = 1;").unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_path(&path).unwrap();
    let mut paths = IncrementalPaths::new();
    paths.after_explicit_scan(&project, std::slice::from_ref(&path));
    fs::remove_file(&path).unwrap();
    paths.commit_project_snapshot(&project);
    assert_eq!(
        paths.effective_changes(&root, std::slice::from_ref(&path)),
        [path]
    );
}
