//! SDK admission uses complete lowering packets and closed lookup oracles.
use super::helper_abi_fixture::{InstalledSdk, Mode};
use serde_json::{Value, json};

#[test]
fn intact_sdk_keeps_the_shared_lowering() {
    let sdk = InstalledSdk::new();
    sdk.assert_projection(Mode::Shared);
}

#[test]
fn every_conditional_type_entry_must_supply_the_consumed_helper() {
    for entry in ["virtual-ts.d.ts", "virtual-ts.d.mts", "virtual-ts.d.cts"] {
        let sdk = InstalledSdk::new();
        sdk.rename(
            entry,
            "type $ComponentRawProps<",
            "type $MissingComponentRawProps<",
        );
        sdk.assert_projection(Mode::Core);
    }
}

#[test]
fn an_unavailable_private_dependency_cannot_enable_shared_lowering() {
    let sdk = InstalledSdk::new();
    std::fs::remove_file(sdk.package.join("virtual-ts-slots.d.ts")).unwrap();
    sdk.assert_projection(Mode::Core);
}

#[test]
fn a_missing_private_binding_invalidates_the_same_dependency_closure() {
    let sdk = InstalledSdk::new();
    sdk.rename(
        "virtual-ts-slots.d.ts",
        "type $NativeSlotsFactory<",
        "type $MissingNativeSlotsFactory<",
    );
    sdk.assert_projection(Mode::Core);
}

#[test]
fn a_conditional_export_cannot_redirect_the_qualified_sdk() {
    let sdk = InstalledSdk::new();
    let manifest = sdk.package.join("package.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    value["exports"]["./virtual-ts"]["require"]["types"] = json!("./virtual-ts.d.ts");
    std::fs::write(manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    sdk.assert_projection(Mode::Core);
}

#[test]
fn user_resolution_overrides_cannot_bypass_the_declaration_contract() {
    for compiler_options in [
        json!({"paths": {"vize/*": ["./custom-sdk/*"]}}),
        json!({"moduleSuffixes": [".custom", ""]}),
    ] {
        let sdk = InstalledSdk::new();
        sdk.assert_projection_with_options(Mode::Inline, compiler_options);
    }
}

#[test]
fn sdk_repairs_invalidate_the_open_project_identity_and_watch_closure() {
    let sdk = InstalledSdk::new();
    let before = sdk.open_result();
    let declaration = sdk.package.join("virtual-ts.d.mts");
    let original = std::fs::read(&declaration).unwrap();
    std::fs::remove_file(&declaration).unwrap();
    let incomplete = sdk.open_result();
    assert_ne!(before["configIdentity"], incomplete["configIdentity"]);
    assert_eq!(
        before["watchedFiles"],
        sdk.expected_watches(true, None, false)
    );
    assert_eq!(incomplete["watchedFiles"], before["watchedFiles"]);
    std::fs::write(declaration, original).unwrap();
    assert_eq!(before, sdk.open_result());
}

#[cfg(unix)]
#[test]
fn pnpm_helpers_use_their_physical_vue_peer_by_default() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    sdk.assert_projection(Mode::Inline);
}

#[cfg(unix)]
#[test]
fn pnpm_helpers_with_the_same_physical_vue_peer_keep_shared_types() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(true);
    sdk.assert_projection(Mode::Shared);
}

#[cfg(unix)]
#[test]
fn preserve_symlinks_uses_the_logical_vue_authority() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    sdk.assert_projection_with_options(Mode::Shared, json!({"preserveSymlinks": true}));
}

#[cfg(unix)]
#[test]
fn pnpm_peer_repairs_are_watched_and_change_the_project_identity() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    let physical = std::fs::canonicalize(&sdk.package).unwrap();
    let peer = physical.parent().unwrap().join("vue");
    let before = sdk.open_result();
    assert_eq!(
        before["watchedFiles"],
        sdk.expected_watches(true, Some(peer.join("package.json").as_path()), false)
    );
    std::fs::remove_dir_all(&peer).unwrap();
    std::os::unix::fs::symlink(sdk._directory.path().join("node_modules/vue"), peer).unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    sdk.assert_projection(Mode::Shared);
}

#[test]
fn a_new_closer_vue_installation_is_watched_before_it_exists() {
    let sdk = InstalledSdk::new();
    let closer = sdk.config.parent().unwrap().join("node_modules/vue");
    let before = sdk.open_result();
    assert_eq!(
        before["watchedFiles"],
        sdk.expected_watches(true, None, false)
    );
    std::fs::create_dir_all(&closer).unwrap();
    std::fs::write(closer.join("package.json"), "{}").unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    sdk.assert_projection(Mode::Inline);
}

#[test]
fn absent_wildcard_targets_keep_sharing_until_a_redirect_can_exist() {
    let sdk = InstalledSdk::new();
    let options = json!({"baseUrl": ".", "paths": {"*": ["types/*"]}});
    std::fs::write(
        &sdk.config,
        serde_json::to_vec(&json!({ "compilerOptions": options })).unwrap(),
    )
    .unwrap();
    let types = sdk.config.parent().unwrap().join("types");
    std::fs::create_dir_all(&types).unwrap();
    let before = sdk.open_result_with_options(options.clone());
    assert_eq!(
        before["watchedFiles"],
        sdk.expected_watches(true, None, true)
    );
    sdk.assert_projection_with_options(Mode::Shared, options.clone());
    std::fs::create_dir_all(types.join("vize")).unwrap();
    std::fs::write(types.join("vize/virtual-ts.d.ts"), "export {};\n").unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result_with_options(options.clone())["configIdentity"]
    );
    sdk.assert_projection_with_options(Mode::Inline, options);
}

#[test]
fn first_sdk_installation_is_in_the_missing_lookup_watch_closure() {
    let sdk = InstalledSdk::new();
    let held = sdk._directory.path().join("held-sdk");
    std::fs::rename(&sdk.package, &held).unwrap();
    let before = sdk.open_result();
    assert_eq!(
        before["watchedFiles"],
        sdk.expected_watches(false, None, false)
    );
    std::fs::rename(held, &sdk.package).unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    sdk.assert_projection(Mode::Shared);
}

#[cfg(unix)]
#[test]
fn closer_sdk_candidates_are_watched_before_an_installed_parent() {
    let sdk = InstalledSdk::new();
    let closer = sdk.config.parent().unwrap().join("node_modules/vize");
    let before = sdk.open_result();
    assert_eq!(
        before["watchedFiles"],
        sdk.expected_watches(true, None, false)
    );
    std::fs::create_dir_all(closer.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&sdk.package, closer).unwrap();
    assert_ne!(
        before["configIdentity"],
        sdk.open_result()["configIdentity"]
    );
    sdk.assert_projection(Mode::Shared);
}

#[cfg(unix)]
#[test]
fn a_manifest_symlink_cannot_move_the_sdk_vue_authority() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    let metadata = sdk._directory.path().join("metadata/package.json");
    std::fs::create_dir_all(metadata.parent().unwrap()).unwrap();
    let manifest = sdk.package.join("package.json");
    std::fs::rename(&manifest, &metadata).unwrap();
    std::os::unix::fs::symlink(metadata, manifest).unwrap();
    sdk.assert_projection(Mode::Inline);
}

#[cfg(unix)]
#[test]
fn shared_manifest_bytes_do_not_merge_distinct_vue_package_directories() {
    let sdk = InstalledSdk::new();
    sdk.move_to_pnpm_store(false);
    let project = sdk._directory.path().join("node_modules/vue/package.json");
    let physical = std::fs::canonicalize(&sdk.package).unwrap();
    let peer = physical.parent().unwrap().join("vue/package.json");
    let metadata = sdk._directory.path().join("metadata/vue.json");
    std::fs::create_dir_all(metadata.parent().unwrap()).unwrap();
    std::fs::rename(&project, &metadata).unwrap();
    std::fs::remove_file(&peer).unwrap();
    std::os::unix::fs::symlink(&metadata, project).unwrap();
    std::os::unix::fs::symlink(metadata, peer).unwrap();
    sdk.assert_projection(Mode::Inline);
}
