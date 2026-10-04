use super::{ModuleTargetError, PhysicalTargets, TargetPolicy, root, symlink};

#[test]
fn module_link_target_fresh_final_identity_refuses_equal_bytes_delete_recreate_and_symlink() {
    for link in [false, true] {
        let (_dir, root, source) = root();
        std::fs::write(root.join("child.ts"), "same bytes").unwrap();
        let observed = PhysicalTargets::observe(&root, &source, &["./child.ts"]).unwrap();
        std::fs::rename(root.join("child.ts"), root.join("held.ts")).unwrap();
        if link {
            symlink(root.join("held.ts"), root.join("child.ts")).unwrap();
        } else {
            std::fs::write(root.join("child.ts"), "same bytes").unwrap();
        }
        assert!(matches!(
            observed.recheck(),
            Err(ModuleTargetError::Changed(_)) | Err(ModuleTargetError::Io { .. })
        ));
    }
}

#[test]
fn module_link_target_parent_replacement_refuses_equal_hardlink_to_original_final_object() {
    let (_dir, root, source) = root();
    std::fs::create_dir(root.join("parent")).unwrap();
    std::fs::write(root.join("parent/child.ts"), "original").unwrap();
    let observed = PhysicalTargets::observe(&root, &source, &["./parent/child.ts"]).unwrap();
    std::fs::rename(root.join("parent"), root.join("held-parent")).unwrap();
    std::fs::create_dir(root.join("parent")).unwrap();
    std::fs::hard_link(
        root.join("held-parent/child.ts"),
        root.join("parent/child.ts"),
    )
    .unwrap();
    assert!(
        matches!(observed.recheck(), Err(ModuleTargetError::Changed(path)) if path == root.join("parent"))
    );
}

#[test]
fn module_link_target_root_pathname_reopen_detects_replacement_with_original_file_hardlink() {
    let (_dir, top, _) = root();
    let root = top.join("root");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("child.ts"), "original").unwrap();
    let source = tower_lsp::lsp_types::Url::from_file_path(root.join("unsaved.ts")).unwrap();
    let observed = PhysicalTargets::observe(&root, &source, &["./child.ts"]).unwrap();
    std::fs::rename(&root, top.join("held-root")).unwrap();
    std::fs::create_dir(&root).unwrap();
    std::fs::hard_link(top.join("held-root/child.ts"), root.join("child.ts")).unwrap();
    assert!(matches!(observed.recheck(), Err(ModuleTargetError::Changed(path)) if path == root));
}

#[test]
fn module_link_target_fifo_is_observed_nonblocking_and_never_regular_file() {
    let (_dir, root, source) = root();
    let path = root.join("fifo.ts");
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: this original temporary path is a valid terminated string.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(matches!(
        PhysicalTargets::observe(&root, &source, &["./fifo.ts"]),
        Err(ModuleTargetError::Policy(TargetPolicy::NonRegular))
    ));
    let observed = {
        std::fs::write(root.join("child.ts"), "original").unwrap();
        PhysicalTargets::observe(&root, &source, &["./child.ts"]).unwrap()
    };
    std::fs::rename(root.join("child.ts"), root.join("held.ts")).unwrap();
    std::fs::rename(&path, root.join("child.ts")).unwrap();
    assert!(matches!(
        observed.recheck(),
        Err(ModuleTargetError::Policy(TargetPolicy::NonRegular))
    ));
}

#[test]
fn module_link_target_point_observation_does_not_claim_future_immutability_or_content() {
    let (_dir, root, source) = root();
    std::fs::write(root.join("child.ts"), "original").unwrap();
    let observed = PhysicalTargets::observe(&root, &source, &["./child.ts"]).unwrap();
    std::fs::write(root.join("child.ts"), "changed original object contents").unwrap();
    let fresh = observed.recheck().unwrap();
    assert_eq!(fresh.uris(), observed.uris());
    std::fs::remove_file(root.join("child.ts")).unwrap();
    assert_eq!(fresh.uris(), observed.uris());
    assert!(observed.recheck().is_err());
}
