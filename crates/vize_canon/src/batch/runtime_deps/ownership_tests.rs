//! Runtime stubs and pruning belong to the mirror, never an installation link.
#![expect(
    clippy::disallowed_types,
    reason = "filesystem snapshots use std strings"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{materialize_runtime_dependencies, resolver::with_test_env_overrides};

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Directory(PathBuf),
    File(PathBuf, Vec<u8>),
    Link(PathBuf, PathBuf),
}

fn snapshot(root: &Path) -> Vec<Entry> {
    fn visit(root: &Path, directory: &Path, entries: &mut Vec<Entry>) {
        let mut children = fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        children.sort();
        for path in children {
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if let Ok(target) = fs::read_link(&path) {
                entries.push(Entry::Link(relative, target));
            } else if fs::symlink_metadata(&path).unwrap().is_dir() {
                entries.push(Entry::Directory(relative));
                visit(root, &path, entries);
            } else {
                entries.push(Entry::File(relative, fs::read(&path).unwrap()));
            }
        }
    }
    let mut entries = Vec::new();
    visit(root, root, &mut entries);
    entries
}

fn link_directory(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    junction::create(source, target).unwrap();
}

fn without_runtime_packages(run: impl FnOnce()) {
    with_test_env_overrides(
        &[
            ("VIZE_VUE_PACKAGE", None),
            ("VIZE_VUE_NAMESPACE_PACKAGE", None),
            ("VIZE_VUE_RUNTIME_DOM_PACKAGE", None),
            ("VIZE_VITE_PACKAGE", None),
            ("VIZE_RUNTIME_NODE_MODULES", None),
            (
                "VIZE_TEST_WORKSPACE_NODE_MODULES",
                Some(Path::new("__none__")),
            ),
        ],
        run,
    );
}

#[test]
fn whole_install_link_is_replaced_before_runtime_writes_and_pruning() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let modules = project.join("node_modules");
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/editor-vite-stub-11145/installation.json"
    )).unwrap();
    for section in ["files", "sentinels"] {
        for (relative, source) in fixture[section].as_object().unwrap() {
            let path = project.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source.as_str().unwrap()).unwrap();
        }
    }
    let link = &fixture["packageLink"];
    link_directory(
        &project.join(link["target"].as_str().unwrap()),
        &project.join(link["alias"].as_str().unwrap()),
    );
    let original = snapshot(&project);
    let mirror = temp.path().join("editor/projects/fixture");
    fs::create_dir_all(&mirror).unwrap();
    link_directory(&modules, &mirror.join("node_modules"));

    without_runtime_packages(|| {
        for _ in 0..2 {
            materialize_runtime_dependencies(&project, &mirror, &[]).unwrap();
            assert_eq!(snapshot(&project), original);
            assert!(fs::read_link(mirror.join("node_modules")).is_err());
            assert_eq!(
                fs::read(mirror.join("node_modules/vite/package.json")).unwrap(),
                super::stubs::VITE_STUB_PACKAGE_JSON.as_bytes()
            );
            assert_eq!(
                fs::read(mirror.join("node_modules/vite/client.d.ts")).unwrap(),
                super::stubs::VITE_CLIENT_STUB.as_bytes()
            );
        }
    });
}

#[cfg(unix)]
#[test]
fn dangling_install_link_does_not_create_its_external_target() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let mirror = temp.path().join("editor");
    let missing = project.join("absent-node_modules");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&mirror).unwrap();
    std::os::unix::fs::symlink(&missing, mirror.join("node_modules")).unwrap();
    without_runtime_packages(|| {
        materialize_runtime_dependencies(&project, &mirror, &[]).unwrap();
    });
    assert!(!missing.exists());
    assert!(fs::read_link(mirror.join("node_modules")).is_err());
    assert!(mirror.join("node_modules/vite/client.d.ts").is_file());
}
