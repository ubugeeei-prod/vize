use super::{ModuleTargetError, PhysicalTargets, TargetOperation, TargetPolicy};
use std::{os::unix::fs::symlink, path::PathBuf};
use tower_lsp::lsp_types::Url;
#[cfg(feature = "native")]
mod events;
mod replacement;
#[cfg(feature = "native")]
mod transport;

fn root() -> (tempfile::TempDir, PathBuf, Url) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let source = Url::from_file_path(root.join("unsaved.ts")).unwrap();
    (dir, root, source)
}

#[test]
fn module_link_target_physical_complete_unicode_space_and_explicit_extension_uris() {
    let (_dir, root, source) = root();
    let names = [
        "child.js",
        "child.jsx",
        "child.mjs",
        "child.cjs",
        "child.ts",
        "child.tsx",
        "child.mts",
        "child.cts",
        "child.vue",
        "😀 café.ts",
    ];
    for name in names {
        std::fs::write(root.join(name), "original").unwrap();
    }
    let requests: Vec<vize_l0::String> = names
        .iter()
        .map(|name| vize_l0::cstr!("./{name}"))
        .collect();
    let requests: Vec<&str> = requests.iter().map(|name| name.as_str()).collect();
    let observed = PhysicalTargets::observe(&root, &source, &requests).unwrap();
    let expected: Vec<Url> = names
        .iter()
        .map(|name| Url::from_file_path(root.join(name)).unwrap())
        .collect();
    assert_eq!(observed.uris(), expected);
    assert_eq!(observed.recheck().unwrap().uris(), expected);
    assert!(!root.join("unsaved.ts").exists());
}

#[test]
fn module_link_target_actual_policy_refuses_guesses_encoded_and_parent_paths() {
    let (_dir, root, source) = root();
    std::fs::write(root.join("child.ts"), "original").unwrap();
    for request in [
        "child.ts",
        "../child.ts",
        "./../child.ts",
        "/child.ts",
        "C:/child.ts",
        "file:child.ts",
        "@/child.ts",
        "#child.ts",
        "./child",
        "./child.TS",
        "./child.ts?raw",
        "./child.ts#part",
        "./child%20name.ts",
        "./child\\name.ts",
        "./child\0.ts",
        "./child\r.ts",
        "./child\n.ts",
        "./",
        "././child.ts",
        "./a//child.ts",
        "./child.ts/",
    ] {
        assert!(
            matches!(
                PhysicalTargets::observe(&root, &source, &[request]),
                Err(ModuleTargetError::Policy(TargetPolicy::Candidate))
            ),
            "{request:?}"
        );
    }
    let error = PhysicalTargets::observe(&root, &source, &["./missing.ts"])
        .err()
        .unwrap();
    match error {
        ModuleTargetError::Io {
            operation,
            path,
            error,
        } => {
            assert_eq!(operation, TargetOperation::Open);
            assert_eq!(path, root.join("missing.ts"));
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
        error => panic!("actual missing-file IO retained: {error:?}"),
    }
}

#[test]
fn module_link_target_source_uri_root_extent_and_nonregular_targets_refuse() {
    let (_dir, root, source) = root();
    std::fs::write(root.join("child.ts"), "original").unwrap();
    for uri in [
        "untitled:original.ts",
        "https://example.test/source.ts",
        "file://foreign.invalid/source.ts",
        "file:///source.ts",
        "file:///source.ts?query",
        "file:///source.ts#part",
    ] {
        let result = PhysicalTargets::observe(&root, &Url::parse(uri).unwrap(), &["./child.ts"]);
        assert!(
            matches!(
                result,
                Err(ModuleTargetError::Policy(
                    TargetPolicy::SourceUri | TargetPolicy::SourceParent
                ))
            ),
            "{uri}"
        );
    }
    assert!(matches!(
        PhysicalTargets::observe(std::path::Path::new("relative"), &source, &["./child.ts"]),
        Err(ModuleTargetError::Policy(TargetPolicy::Root))
    ));
    std::fs::create_dir(root.join("directory.ts")).unwrap();
    assert!(matches!(
        PhysicalTargets::observe(&root, &source, &["./directory.ts"]),
        Err(ModuleTargetError::Policy(TargetPolicy::NonRegular))
    ));
    let socket = std::os::unix::net::UnixListener::bind(root.join("socket.ts")).unwrap();
    assert!(matches!(
        PhysicalTargets::observe(&root, &source, &["./socket.ts"]),
        Err(ModuleTargetError::Io { .. })
            | Err(ModuleTargetError::Policy(TargetPolicy::NonRegular))
    ));
    drop(socket);
}

#[test]
fn module_link_target_original_root_parent_and_final_symlinks_never_follow() {
    let (_dir, root, source) = root();
    std::fs::create_dir(root.join("real")).unwrap();
    std::fs::write(root.join("real/child.ts"), "original").unwrap();
    symlink(root.join("real"), root.join("linked")).unwrap();
    symlink(root.join("real/child.ts"), root.join("child.ts")).unwrap();
    for request in ["./linked/child.ts", "./child.ts"] {
        assert!(matches!(
            PhysicalTargets::observe(&root, &source, &[request]),
            Err(ModuleTargetError::Io {
                operation: TargetOperation::Open,
                ..
            })
        ));
    }
    let nested_source = Url::from_file_path(root.join("linked/source.ts")).unwrap();
    assert!(matches!(
        PhysicalTargets::observe(&root, &nested_source, &["./child.ts"]),
        Err(ModuleTargetError::Io {
            operation: TargetOperation::Open,
            ..
        })
    ));
    let link_root = root.join("root-link");
    symlink(root.join("real"), &link_root).unwrap();
    let link_source = Url::from_file_path(link_root.join("source.ts")).unwrap();
    assert!(matches!(
        PhysicalTargets::observe(&link_root, &link_source, &["./child.ts"]),
        Err(ModuleTargetError::Io {
            operation: TargetOperation::Open,
            ..
        })
    ));
}
