//! A real Vue runtime PropType must preserve authored errors and callback types.
#![expect(clippy::unwrap_used, reason = "fixture setup fails by panicking")]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};
use vize_l0::{String, cstr};

#[test]
fn runtime_proptype_keeps_typed_callbacks_and_wrong_assignment() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures =
        repo.join("tests/_fixtures/differential/typechecker/runtime-proptype-call-result-11734");
    let provenance: Value =
        serde_json::from_slice(&std::fs::read(fixtures.join("contract-provenance.json")).unwrap())
            .unwrap();
    assert_eq!(provenance["version"], "3.5.44");
    let contract =
        std::fs::read(fixtures.join(provenance["contractPath"].as_str().unwrap())).unwrap();
    assert_eq!(
        Sha256::digest(contract)
            .iter()
            .map(|byte| cstr!("{byte:02x}"))
            .collect::<String>(),
        provenance["contractSha256"].as_str().unwrap()
    );
    let cases: Value =
        serde_json::from_slice(&std::fs::read(fixtures.join("cases.json")).unwrap()).unwrap();
    assert_eq!(cases["issue"], 11734);
    assert_eq!(cases["cases"].as_array().unwrap().len(), 2);
    for case in cases["cases"].as_array().unwrap() {
        check_case(&repo, &fixtures, case);
    }
}

fn check_case(repo: &Path, fixtures: &Path, case: &Value) {
    let project = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(project.path()).unwrap();
    std::fs::create_dir(root.join("node_modules")).unwrap();
    let vue = std::fs::canonicalize(repo.join("tests/node_modules/vue")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, root.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    junction::create(vue, root.join("node_modules/vue")).unwrap();
    for input in case["inputs"].as_array().unwrap() {
        std::fs::copy(
            fixtures.join(input["source"].as_str().unwrap()),
            root.join(input["file"].as_str().unwrap()),
        )
        .unwrap();
    }
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let actual: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "file": diagnostic.file.strip_prefix(&root).unwrap().to_str().unwrap(),
                "line": diagnostic.line + 1,
                "column": diagnostic.column + 1,
                "severity": diagnostic.severity,
                "code": diagnostic.code,
                "message": diagnostic.message.as_str()
            })
        })
        .collect();
    assert_eq!(json!(actual), case["diagnostics"], "{}", case["id"]);
}
