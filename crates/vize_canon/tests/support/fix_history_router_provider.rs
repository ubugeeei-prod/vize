//! Hydrate only the registered experimental-page corpus with its real provider.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Install the unchanged source-build members bound by their recorded digests.
pub(crate) fn install_source_build(root: &Path, project: &Path) {
    let fixture = root.join("tests/fixtures/typechecker/experimental-router-project-capability");
    let authority: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixture.join("artifacts.json")).expect("provider authority must exist"),
    )
    .expect("provider authority must be JSON");
    let provider = authority["providers"]
        .as_array()
        .and_then(|providers| providers.iter().find(|entry| entry["kind"] == "preview"))
        .expect("the source-built provider must be recorded");
    for entry in provider["files"]
        .as_array()
        .expect("provider files must exist")
    {
        let path = entry["path"]
            .as_str()
            .expect("a provider member has a path");
        let mut member = fixture.join("preview").join(path).into_os_string();
        member.push(".txt");
        let bytes = std::fs::read(member).expect("a recorded provider member must exist");
        assert_eq!(
            super::fix_history_observation::digest(&bytes),
            entry["sha256"]
                .as_str()
                .expect("a provider member has a digest")
        );
        let target = project.join("node_modules/vue-router").join(path);
        std::fs::create_dir_all(target.parent().expect("a member has a parent"))
            .expect("provider directories must be created");
        std::fs::write(target, bytes).expect("provider members must be installed");
    }
}

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
    assert_eq!(project.parent(), Some(output.as_path()));
    assert!(
        project
            .join("node_modules/vue-router/dist/experimental/index.d.ts")
            .is_file()
    );
    project
}
