use std::fs;

use tempfile::TempDir;

use super::normalize_corsa_path_with_discovery;

#[test]
fn normalize_keeps_wrapper_when_nothing_better_exists() {
    let temp_dir = TempDir::new().unwrap();
    let wrapper = temp_dir
        .path()
        .join("node_modules")
        .join(".bin")
        .join("tsgo");
    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    fs::write(&wrapper, "binary").unwrap();

    let normalized = normalize_corsa_path_with_discovery(&wrapper, |project_root| {
        assert_eq!(project_root, temp_dir.path());
        None
    });

    assert_eq!(normalized, wrapper);
}
