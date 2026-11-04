//! A later sibling branch must not disconnect an actual listener value read.

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, workspace_root,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

fn sha256(bytes: &[u8]) -> vize_l0::String {
    let mut value = vize_l0::String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        value.push_str(vize_l0::cstr!("{byte:02x}").as_str());
    }
    value
}

#[test]
#[ignore = "requires the authenticated beta host and actual source CLI in native-canon Actions"]
fn standard_tsgo_nested_function_guards_keep_listener_reads_and_real_errors() {
    let host = std::env::var_os(TSGO_ENV)
        .map(PathBuf::from)
        .expect("this explicitly selected regression requires the real beta content-mapper host");
    assert!(host.is_file(), "the required host must be a physical image");
    let version = Command::new(&host).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.1.0-beta\n");
    assert_eq!(version.stderr, b"");
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/nested-function-guard-11853");
    let contract: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("cases.json")).unwrap()).unwrap();
    let expected = contract["expectedStdout"].as_str().unwrap();
    let parent = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&parent).unwrap();
    let project = tempfile::Builder::new()
        .prefix("nested-function-guard-11853-")
        .tempdir_in(parent)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    let inputs: Vec<_> = contract["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| {
            let file = input.as_str().unwrap();
            let source = std::fs::read_to_string(fixture.join(format!("{file}.txt"))).unwrap();
            std::fs::write(project.path().join(file), &source).unwrap();
            (file, source)
        })
        .collect();
    let result = check_project(&host, project.path(), "tsconfig.json");
    let capture = std::env::var_os("VIZE_NESTED_FUNCTION_GUARD_CAPTURE_DIR")
        .map(PathBuf::from)
        .expect("the complete native packet must be retained");
    std::fs::create_dir_all(&capture).unwrap();
    let packet = json!({
        "route":"actual-source-cli-content-mapper-protocol", "host":host,
        "hostVersion":version.stdout,
        "hostSha256":sha256(&std::fs::read(&host).unwrap()),
        "sourceCli":env!("CARGO_BIN_EXE_vize"),
        "sourceCliSha256":sha256(&std::fs::read(env!("CARGO_BIN_EXE_vize")).unwrap()),
        "inputs":inputs, "contract":contract, "expectedStdout":expected,
        "status":result.status.code(),"stdout":result.stdout,"stderr":result.stderr,
    });
    std::fs::write(
        capture.join("native-nested-function-guard-11853.json"),
        serde_json::to_vec_pretty(&packet).unwrap(),
    )
    .unwrap();
    println!("nested function guard source protocol: {packet}");
    assert_eq!(result.status.code(), Some(2), "{packet}");
    assert_eq!(result.stderr, b"", "{packet}");
    assert_eq!(result.stdout, expected.as_bytes(), "{packet}");
}
