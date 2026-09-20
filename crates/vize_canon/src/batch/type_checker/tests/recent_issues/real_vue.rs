//! Runtime macro inference must be checked against Vue, not the dependency stub.

use std::path::{Path, PathBuf};

pub(super) fn create_project_case(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let modules = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join("tests/node_modules");
    assert!(
        modules.join("vue/package.json").is_file(),
        "install workspace dependencies before native Vue tests"
    );
    super::super::with_workspace_node_modules_override(modules.to_str(), || {
        super::super::create_project_case(name, files)
    })
}
