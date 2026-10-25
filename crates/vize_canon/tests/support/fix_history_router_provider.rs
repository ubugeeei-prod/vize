//! Hydrate only the registered experimental-page corpus with its real provider.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]

use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn prepare(root: &Path, output: &Path, fixtures: &Path) -> PathBuf {
    let root = std::fs::canonicalize(root).expect("the repository must resolve");
    let output = std::fs::canonicalize(output).expect("the owned output must resolve");
    let authority = std::fs::canonicalize(fixtures.join("provider.json.txt"))
        .expect("the registered provider authority must resolve");
    let result = Command::new(std::env::var_os("VIZE_TEST_NODE").unwrap_or_else(|| "node".into()))
        .arg(root.join("tests/tooling/support/router-experimental-provider-cli.ts"))
        .arg(&output)
        .arg(authority)
        .current_dir(&root)
        .output()
        .expect("the real pinned provider hydrator must launch");
    std::fs::write(output.join("provider-cli.stdout"), &result.stdout)
        .expect("provider stdout must persist before assertions");
    std::fs::write(output.join("provider-cli.stderr"), &result.stderr)
        .expect("provider stderr must persist before assertions");
    assert!(
        result.status.success(),
        "the actual provider must install and generate: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&result.stdout).expect("provider custody must return JSON");
    let project = PathBuf::from(value["root"].as_str().expect("the project root must exist"));
    assert!(project.starts_with(&output));
    assert!(
        project
            .join("node_modules/vue-router/dist/experimental/index.d.ts")
            .is_file()
    );
    project
}
