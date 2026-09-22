use std::path::Path;

use vize_carton::String;
use vize_davinci::key::manifest::{AmbientInput, CachedArtifact};

use super::{CorsaSessionKey, SessionInputs, corsa_build_identity, tsconfig_chain_digest};

fn base_inputs() -> SessionInputs {
    SessionInputs {
        project: "/work/app/tsconfig.json".into(),
        tsconfig: String::from("0123456789abcdef0123456789abcdef"),
        toolchain: String::from("0.425.1"),
        corsa: String::from("/work/app/node_modules/.bin/tsgo len=1 mtime=2"),
        flags: String::from("options-api=1 vue=3.5"),
        platform: String::from("aarch64-macos-unix"),
    }
}

/// One manifest input's name and a mutation of just that input.
type Flip = (&'static str, fn(&mut SessionInputs));

fn write(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
}

#[test]
fn equal_keys_compare_equal() {
    let a = CorsaSessionKey::from_inputs(&base_inputs());
    let b = CorsaSessionKey::from_inputs(&base_inputs());
    assert_eq!(a, b);
    assert_eq!(a.fingerprint(), b.fingerprint());
}

#[test]
fn different_paths_yield_different_keys() {
    let a = CorsaSessionKey::new("./tsconfig.json", "tsgo", "");
    let b = CorsaSessionKey::new("./tsconfig.app.json", "tsgo", "");
    assert_ne!(a, b);
}

/// P5-8's acceptance: flipping any one manifest input yields another key,
/// and the seven keys are pairwise distinct.
#[test]
fn every_manifest_input_separates_sessions() {
    let base = base_inputs();
    let flips: [Flip; 6] = [
        ("project-identity", |inputs| {
            inputs.project = "/work/other/tsconfig.json".into();
        }),
        ("tsconfig-content", |inputs| {
            inputs.tsconfig = String::from("fedcba9876543210fedcba9876543210");
        }),
        ("toolchain-version", |inputs| {
            inputs.toolchain = String::from("0.426.0");
        }),
        ("corsa-version", |inputs| {
            inputs.corsa = String::from("/work/app/node_modules/.bin/tsgo len=1 mtime=3");
        }),
        ("feature-flags", |inputs| {
            inputs.flags = String::from("options-api=0 vue=3.5");
        }),
        ("platform", |inputs| {
            inputs.platform = String::from("x86_64-linux-unix");
        }),
    ];
    let mut keys = vec![CorsaSessionKey::from_inputs(&base)];
    for (input, flip) in flips {
        let mut flipped = base.clone();
        flip(&mut flipped);
        let key = CorsaSessionKey::from_inputs(&flipped);
        assert_ne!(key, keys[0], "flipping {input} must change the key");
        keys.push(key);
    }
    for (i, a) in keys.iter().enumerate() {
        for b in &keys[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn the_key_folds_exactly_the_corsa_session_row() {
    assert_eq!(
        CachedArtifact::CorsaSession.inputs(),
        &[
            AmbientInput::ProjectIdentity,
            AmbientInput::TsconfigContent,
            AmbientInput::ToolchainVersion,
            AmbientInput::CorsaVersion,
            AmbientInput::FeatureFlags,
            AmbientInput::Platform,
        ],
    );
    base_inputs()
        .manifest()
        .check(CachedArtifact::CorsaSession)
        .unwrap();
}

#[test]
fn tsconfig_content_follows_the_extends_chain() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("tsconfig.json");
    let base = dir.path().join("tsconfig.base.json");
    write(&config, r#"{ "extends": "./tsconfig.base.json" }"#);
    write(&base, r#"{ "compilerOptions": { "strict": true } }"#);
    let key = |dir: &Path| CorsaSessionKey::new(dir.join("tsconfig.json"), "tsgo", "");
    let first = key(dir.path());
    assert_eq!(first, key(dir.path()), "an unchanged chain keeps the key");

    write(&base, r#"{ "compilerOptions": { "strict": false } }"#);
    let edited_base = key(dir.path());
    assert_ne!(edited_base, first, "an edit to an extended config");

    write(
        &config,
        r#"{ "extends": "./tsconfig.base.json", "include": [] }"#,
    );
    assert_ne!(key(dir.path()), edited_base, "an edit to the config itself");
}

#[test]
fn tsconfig_content_covers_extends_arrays_and_unresolved_targets() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("tsconfig.json");
    write(&config, r#"{ "extends": ["./a.json", "./b.json"] }"#);
    write(&dir.path().join("a.json"), "{}");
    let before = tsconfig_chain_digest(&config);
    write(&dir.path().join("b.json"), "{}");
    let created = tsconfig_chain_digest(&config);
    assert_ne!(before, created, "creating a missing extended config");

    write(&config, r#"{ "extends": ["./b.json", "./a.json"] }"#);
    assert_ne!(tsconfig_chain_digest(&config), created, "extends order");

    write(&config, r#"{ "extends": "@vize-demo/tsconfig" }"#);
    let unresolved = tsconfig_chain_digest(&config);
    assert_eq!(unresolved.len(), 32);
    let package = dir.path().join("node_modules/@vize-demo/tsconfig");
    std::fs::create_dir_all(&package).unwrap();
    write(&package.join("tsconfig.json"), "{}");
    assert_ne!(
        tsconfig_chain_digest(&config),
        unresolved,
        "installing the extended package, the config's bytes unchanged"
    );
}

#[test]
fn equal_content_at_another_path_is_another_project() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    for dir in [&a, &b] {
        write(&dir.path().join("tsconfig.json"), "{}");
    }
    let key = |dir: &Path| CorsaSessionKey::new(dir.join("tsconfig.json"), "tsgo", "");
    assert_ne!(key(a.path()), key(b.path()));
}

#[test]
#[cfg(unix)]
fn a_symlinked_tsconfig_is_the_same_project() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("tsconfig.json");
    let link = dir.path().join("linked.json");
    write(&config, "{}");
    std::os::unix::fs::symlink(&config, &link).unwrap();
    assert_eq!(
        CorsaSessionKey::new(&config, "tsgo", ""),
        CorsaSessionKey::new(&link, "tsgo", ""),
    );
}

#[test]
fn the_corsa_build_identity_changes_with_the_binary() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("tsgo");
    write(&binary, "build one");
    let first = corsa_build_identity(&binary);
    assert_eq!(first, corsa_build_identity(&binary));
    write(&binary, "build number two");
    assert_ne!(corsa_build_identity(&binary), first);
    assert!(corsa_build_identity(dir.path().join("absent")).ends_with(" missing"));
}
