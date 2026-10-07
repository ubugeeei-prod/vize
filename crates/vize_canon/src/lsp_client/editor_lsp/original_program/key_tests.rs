//! Complete native-image bytes and inherited configuration govern reuse.

use super::diagnosing_key;

#[test]
fn launcher_scripts_preserve_fresh_diagnosing_process_behavior() {
    let root = tempfile::TempDir::new().unwrap();
    let launcher = root.path().join("corsa");
    std::fs::write(&launcher, "#!/bin/sh\nexec \"$SELECTED_CORSA\" \"$@\"\n").unwrap();
    assert!(
        diagnosing_key(
            launcher.to_str().unwrap(),
            &root.path().join("tsconfig.json")
        )
        .is_none()
    );
}

#[test]
fn same_length_runtime_bytes_and_inherited_config_changes_invalidate_key() {
    let root = tempfile::TempDir::new().unwrap();
    let native = root.path().join("native-image");
    let config = root.path().join("tsconfig.json");
    let base = root.path().join("base.json");
    std::fs::write(&native, b"\x7fELFfirst-native-build").unwrap();
    std::fs::write(
        &config,
        r#"{"extends":"./base.json","include":["source.ts"]}"#,
    )
    .unwrap();
    std::fs::write(&base, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
    let first = diagnosing_key(native.to_str().unwrap(), &config).unwrap();
    assert_eq!(
        diagnosing_key(native.to_str().unwrap(), &config).unwrap(),
        first
    );
    std::fs::write(&native, b"\x7fELFother-native-build").unwrap();
    let replaced = diagnosing_key(native.to_str().unwrap(), &config).unwrap();
    assert_ne!(replaced, first);
    std::fs::write(&native, b"\x7fELFfirst-native-build").unwrap();
    assert_eq!(
        diagnosing_key(native.to_str().unwrap(), &config).unwrap(),
        first
    );
    std::fs::write(&base, r#"{"compilerOptions":{"strict":false}}"#).unwrap();
    assert_ne!(
        diagnosing_key(native.to_str().unwrap(), &config).unwrap(),
        first
    );
}
