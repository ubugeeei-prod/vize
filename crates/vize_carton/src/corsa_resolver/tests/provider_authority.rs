//! Selection graphs only: marker files are not compiler/API qualification.

use super::super::{CORSA_ENV_VARS, discover_in_walk, normalize_corsa_path, platform_suffix};
use super::{resolve, write_file, write_typescript_manifest};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn native(root: &Path, version: &str) -> PathBuf {
    write_typescript_manifest(
        &root.join("node_modules/typescript/package.json"),
        platform_suffix(),
        version,
    );
    let binary = root
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("typescript-{}", platform_suffix()))
        .join("lib/tsc");
    write_file(&binary);
    binary.canonicalize().unwrap()
}

#[cfg(unix)]
#[test]
fn explicit_alias_launcher_owns_config_and_all_environment_overrides() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let canonical = native(root, "7.0.2");
    let alias_root = root.join("installed/beta");
    let beta = native(&alias_root, "7.1.0-beta");
    fs::write(root.join("package.json"), r#"{"devDependencies":{"typescript":"7.0.2","typescript-native":"npm:typescript@7.1.0-beta"}}"#).unwrap();
    std::os::unix::fs::symlink(
        alias_root.join("node_modules/typescript"),
        root.join("node_modules/typescript-native"),
    )
    .unwrap();
    let launcher = alias_root.join("node_modules/typescript/bin/tsc");
    write_file(&launcher);
    let wrapper = root.join("node_modules/.bin/tsgo");
    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&launcher, &wrapper).unwrap();
    assert_eq!(
        discover_in_walk(&[root.to_path_buf()], false),
        Some(canonical)
    );
    assert_eq!(resolve(Some(&wrapper), Some(root), &[]).unwrap(), beta);
    for name in CORSA_ENV_VARS {
        assert_eq!(
            resolve(None, Some(root), &[(name, &wrapper)]).unwrap(),
            beta
        );
    }
    // A bin entry that does not own this selected target cannot redirect it.
    let manifest = alias_root.join("node_modules/typescript/package.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("bin".into(), serde_json::json!({"tsc":"../../foreign"}));
    fs::write(manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        normalize_corsa_path(&wrapper),
        launcher.canonicalize().unwrap()
    );
}

#[test]
fn opaque_explicit_wrapper_is_not_replaced_by_another_provider() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    native(root, "7.0.2");
    let selected = root.join("node_modules/.bin/tsgo");
    write_file(&selected);
    assert_eq!(
        resolve(Some(&selected), Some(root), &[]).unwrap(),
        selected.canonicalize().unwrap()
    );
}

#[test]
fn explicit_shim_follows_the_one_package_declaring_its_bin() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let expected = native(root, "7.1.0-beta");
    write_file(&root.join("node_modules/typescript/bin/tsc"));
    // npm on Windows and pnpm write regular-file shims, not links.
    for shim in ["tsc", "tsc.cmd", "tsc.ps1"] {
        let shim = root.join("node_modules/.bin").join(shim);
        write_file(&shim);
        assert_eq!(resolve(Some(&shim), Some(root), &[]).unwrap(), expected);
    }
    // A second owner of the same bin name makes the shim's provider unknown.
    let other = root.join("node_modules/@scope/other/package.json");
    write_file(&other);
    fs::write(&other, r#"{"name":"@scope/other","bin":{"tsc":"tsc.js"}}"#).unwrap();
    let shim = root.join("node_modules/.bin/tsc");
    assert_eq!(
        resolve(Some(&shim), Some(root), &[]).unwrap(),
        shim.canonicalize().unwrap()
    );
    // A sole foreign owner is never replaced with the project runtime.
    fs::remove_dir_all(root.join("node_modules/typescript")).unwrap();
    assert_eq!(
        resolve(Some(&shim), Some(root), &[]).unwrap(),
        shim.canonicalize().unwrap()
    );
}

#[test]
fn undeclared_canonical_package_does_not_supply_a_runtime() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let stale = native(root, "7.0.2");
    fs::write(root.join("package.json"), r#"{"private":true}"#).unwrap();
    assert_eq!(discover_in_walk(&[root.to_path_buf()], false), None);
    assert_eq!(resolve(Some(&stale), Some(root), &[]).unwrap(), stale);
}

#[cfg(unix)]
#[test]
fn declared_alias_wins_over_an_undeclared_canonical_package() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    native(root, "7.0.2");
    let alias = root.join("installed/beta");
    let expected = native(&alias, "7.1.0-beta");
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"native-alias":"npm:typescript@7.1.0-beta"}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(
        alias.join("node_modules/typescript"),
        root.join("node_modules/native-alias"),
    )
    .unwrap();
    assert_eq!(
        discover_in_walk(&[root.to_path_buf()], false),
        Some(expected)
    );
}

#[test]
fn declared_canonical_and_alias_follow_their_nearest_hoisted_slot() {
    for name in ["typescript", "native-alias"] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let expected = native(root, "7.1.0-beta");
        fs::write(root.join("package.json"), r#"{"private":true}"#).unwrap();
        if name != "typescript" {
            fs::rename(
                root.join("node_modules/typescript"),
                root.join("node_modules/native-alias"),
            )
            .unwrap();
        }
        let project = root.join("packages/app");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("package.json"),
            serde_json::to_vec(
                &serde_json::json!({"devDependencies":{(name):"npm:typescript@7.1.0-beta"}}),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(discover_in_walk(&[root.to_path_buf()], false), None);
        assert_eq!(discover_in_walk(&[project], false), Some(expected));
    }
}

#[test]
fn declared_platform_follows_its_hoisted_slot() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let expected = native(root, "7.0.2");
    fs::remove_dir_all(root.join("node_modules/typescript")).unwrap();
    fs::write(root.join("package.json"), r#"{"private":true}"#).unwrap();
    let project = root.join("packages/app");
    fs::create_dir_all(&project).unwrap();
    let name = crate::cstr!("@typescript/typescript-{}", platform_suffix());
    fs::write(
        project.join("package.json"),
        serde_json::to_vec(&serde_json::json!({"optionalDependencies":{(name.as_str()):"7.0.2"}}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(discover_in_walk(&[project], false), Some(expected));
}

#[test]
fn invalid_nearest_alias_is_not_rescued_by_an_ancestor_runtime() {
    for manifest in [
        "",
        r#"{"name":"typescript"}"#,
        r#"{"name":"typescript","version":"broken"}"#,
    ] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let outer = native(root, "7.0.2");
        let project = root.join("packages/app");
        let broken = project.join("node_modules/native-alias/package.json");
        write_file(&broken);
        fs::write(&broken, manifest).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"devDependencies":{"native-alias":"npm:typescript@7.1.0-beta"}}"#,
        )
        .unwrap();
        assert_eq!(discover_in_walk(&[root.to_path_buf()], false), Some(outer));
        assert_eq!(discover_in_walk(&[project], false), None);
    }
}

#[test]
fn mismatched_nearest_platform_is_not_rescued_by_an_ancestor_runtime() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let outer = native(root, "7.0.2");
    let project = root.join("packages/app");
    let own = native(&project, "7.1.0-beta");
    let manifest = own.parent().unwrap().parent().unwrap().join("package.json");
    fs::write(
        manifest,
        &*crate::cstr!(
            r#"{{"name":"@typescript/typescript-{}","version":"7.0.2"}}"#,
            platform_suffix()
        ),
    )
    .unwrap();
    assert_eq!(discover_in_walk(&[root.to_path_buf()], false), Some(outer));
    assert_eq!(discover_in_walk(&[project], false), None);
}

#[test]
fn every_implicit_cache_requires_the_developer_flag() {
    for relative in [
        ".cache/tsgo",
        "ref/typescript-go/.cache/tsgo",
        "ref/typescript-go/built/local/tsgo",
        "../corsa-bind/.cache/tsgo",
    ] {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("project");
        fs::create_dir_all(&root).unwrap();
        let binary = root.join(relative);
        write_file(&binary);
        assert_eq!(discover_in_walk(&[root.clone()], false), None);
        assert_eq!(
            discover_in_walk(&[root.clone()], true)
                .unwrap()
                .canonicalize()
                .unwrap(),
            binary.canonicalize().unwrap()
        );
        assert_eq!(
            resolve(Some(&binary), Some(&root), &[]).unwrap(),
            binary.canonicalize().unwrap()
        );
    }
}
