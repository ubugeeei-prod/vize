//! Component contracts keep every strict prop finding through a real Vue.
//!
//! Required, unknown, wrong-value, model, aria/fallthrough, generic, slot,
//! dynamic and repeated usages. Contract emission changes must leave this list
//! byte-identical: code, message and position.
#![expect(clippy::unwrap_used, reason = "fixture setup fails by panicking")]
use serde_json::{Value, json};
use std::path::Path;
use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

const SOURCES: [&str; 9] = [
    "ChildA",
    "ChildB",
    "ChildC",
    "GenChild",
    "SlotChild",
    "Opts",
    "Parent1",
    "Parent2",
    "Parent3",
];

#[test]
fn strict_contract_battery_keeps_every_finding() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures =
        repo.join("tests/_fixtures/differential/typechecker/component-contract-strict-battery");
    let project = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(project.path()).unwrap();
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    let vue = std::fs::canonicalize(repo.join("tests/node_modules/vue")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, root.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    junction::create(vue, root.join("node_modules/vue")).unwrap();
    std::fs::copy(
        fixtures.join("tsconfig.json.txt"),
        root.join("tsconfig.json"),
    )
    .unwrap();
    for name in SOURCES {
        std::fs::copy(
            fixtures.join(vize_l0::cstr!("src/{name}.vue.txt").as_str()),
            root.join(vize_l0::cstr!("src/{name}.vue").as_str()),
        )
        .unwrap();
    }
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut actual: Vec<Value> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "file": diagnostic.file.strip_prefix(&root).unwrap().to_str().unwrap().replace('\\', "/"),
                "line": diagnostic.line + 1,
                "column": diagnostic.column + 1,
                "severity": diagnostic.severity,
                "code": diagnostic.code,
                "message": diagnostic.message.as_str()
            })
        })
        .collect();
    actual.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    let expected: Value =
        serde_json::from_slice(&std::fs::read(fixtures.join("diagnostics.json")).unwrap()).unwrap();
    assert_eq!(
        json!(actual),
        expected,
        "{}",
        serde_json::to_string_pretty(&actual).unwrap()
    );
}
