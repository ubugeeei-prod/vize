use super::supports_shared_editor_project;
use std::path::Path;

#[test]
fn shared_editor_attachment_keeps_sync_fake_and_absent_config_fallbacks() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    assert!(!supports_shared_editor_project(
        "/workspace/.bin/tsgo",
        &config
    ));
    std::fs::write(&config, "{}").unwrap();
    for executable in ["corsa", "/tmp/fake-api.sh", "/tmp/fake-api.py"] {
        assert!(!supports_shared_editor_project(executable, &config));
    }
    assert_eq!(
        supports_shared_editor_project("/workspace/.bin/tsgo", &config),
        cfg!(unix)
    );
    assert!(!supports_shared_editor_project(
        "/workspace/.bin/tsgo",
        root.path()
    ));
    assert!(!supports_shared_editor_project(
        "/workspace/.bin/tsgo",
        Path::new("/absent/config.json")
    ));
}

#[cfg(target_os = "linux")]
mod native;
