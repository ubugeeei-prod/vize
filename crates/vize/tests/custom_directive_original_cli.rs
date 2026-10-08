//! #8226 retains the two complete pinned ant-design-vue files and CLI packets.
#![expect(clippy::disallowed_types, reason = "CLI corpus uses std strings")]
#![expect(clippy::disallowed_macros, reason = "CLI corpus hashes use format")]

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    path: String,
    source: String,
    bytes: usize,
    sha256: String,
    blob: String,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn complete_original_ant_design_files_preserve_every_other_cli_finding() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/custom-directive-identity-8226");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["issue"], 8226);
    assert_eq!(manifest["project"], "ant-design-vue");
    assert_eq!(
        manifest["revision"],
        "7483836f0adac76516e527df893c3d84a7cd4005"
    );
    assert_eq!(
        manifest["matrixSource"],
        "fd6241bf8ea5466794cc955138a59a9b75a8aac2"
    );
    assert_eq!(manifest["matrixRun"], 37700140534u64);
    let inputs: Vec<Input> = serde_json::from_value(manifest["files"].clone()).unwrap();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0].blob, "90b503fda9a0da3a679784692c3c61cca4d38578");
    assert_eq!(inputs[1].blob, "a6124ae25f5688812b4a2c871acaa2dc3f3fadba");
    let expected: Vec<Value> = serde_json::from_slice(
        &fs::read(fixtures.join("expected-original-cli-packets.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(expected.len(), 2);
    for (input, expected) in inputs.iter().zip(&expected) {
        let bytes = fs::read(fixtures.join(&input.source)).unwrap();
        assert_eq!(bytes.len(), input.bytes);
        assert_eq!(digest(&bytes), input.sha256);
        for _ in 0..2 {
            let project = tempfile::tempdir().unwrap();
            let target = project.path().join(&input.path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, &bytes).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .args([
                    "lint",
                    &input.path,
                    "--format",
                    "json",
                    "--no-config",
                    "--preset",
                    "ecosystem",
                ])
                .current_dir(project.path())
                .env("LANG", "C")
                .env("LC_ALL", "C")
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(0),
                "{}: {}",
                input.path,
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "{}", input.path);
            let observed: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(observed, serde_json::json!([expected]), "{}", input.path);
            assert_eq!(fs::read(&target).unwrap(), bytes, "{}", input.path);
            assert!(!project.path().join("vize.config.json").exists());
            assert!(!project.path().join("node_modules").exists());
        }
    }
}
