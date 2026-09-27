//! Diagnostic contracts for historical generator fixes, using the actual
//! production batch checker. Unavailable runtimes and helper errors fail.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    file: String,
    source: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Diagnostic {
    file: String,
    line: u32,
    column: u32,
    severity: u8,
    code: Option<u32>,
    message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    inputs: Vec<Input>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    #[serde(rename = "requiredTier")]
    required_tier: String,
    #[serde(rename = "positionBase")]
    position_base: u32,
    positions: String,
    comparison: String,
    #[serde(rename = "missingFields")]
    missing_fields: Vec<String>,
    native: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pack {
    version: u32,
    issue: u32,
    regression_commit: String,
    historical_issue: Option<u32>,
    source_revision: String,
    diagnostic_contract: Contract,
    cases: Vec<Case>,
}

#[test]
fn inline_event_assignments_preserve_exact_diagnostics() {
    check_pack(
        "event-handler-narrowing",
        "613ce3a5a31d22ec0dfd42feab25772e63fc54bf",
        Some(4996),
        &[
            "component-sibling-valid",
            "native-sibling-valid",
            "handler-assignment-invalid",
        ],
    );
}

#[test]
fn deferred_template_reads_preserve_script_diagnostics() {
    check_pack(
        "template-definite-assignment",
        "39bf60c0614c888ba82f962b944160ce839cf035",
        Some(4239),
        &["vue3-complete-project", "tsx-deferred-template"],
    );
}

#[test]
fn required_props_keep_exact_unicode_diagnostics() {
    check_pack(
        "required-props-edges",
        "f3a26b0e30c98b135a9e897c3584526eb0a2b96d",
        Some(3581),
        &["complete-attribute-boundaries"],
    );
}

#[test]
fn unicode_reserved_props_preserve_value_diagnostics() {
    check_pack(
        "unicode-reserved-props",
        "13e5ec2a9752c5d8d41a9510f54e7096a583bb78",
        None,
        &["literal-values-valid", "literal-values-invalid"],
    );
}

fn check_pack(name: &str, regression: &str, historical_issue: Option<u32>, case_ids: &[&str]) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = root
        .join("tests/_fixtures/differential/typechecker")
        .join(name);
    let pack: Pack = serde_json::from_slice(
        &std::fs::read(fixtures.join("cases.json")).expect("the fixture pack must exist"),
    )
    .expect("the fixture pack must have the expected schema");
    assert_eq!(pack.version, 1);
    assert_eq!(pack.issue, 6879);
    assert_eq!(pack.historical_issue, historical_issue);
    assert_eq!(pack.regression_commit, regression);
    assert_eq!(
        pack.source_revision,
        "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943"
    );
    assert_eq!(pack.diagnostic_contract.position_base, 1);
    assert_eq!(pack.diagnostic_contract.positions, "authored UTF16 start");
    assert_eq!(
        pack.diagnostic_contract.comparison,
        "all production diagnostics in returned order"
    );
    assert_eq!(
        pack.diagnostic_contract.missing_fields,
        ["end", "relatedInformation", "raw backend diagnostics"]
    );
    assert_eq!(pack.diagnostic_contract.native, "unsupported");
    assert_eq!(pack.diagnostic_contract.required_tier, "T1");
    assert_eq!(
        pack.cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        case_ids,
        "missing, duplicated or reordered cases must not pass"
    );
    let vue = std::fs::canonicalize(root.join("tests/node_modules/vue"))
        .expect("the pinned real Vue package must be installed");
    for case in pack.cases {
        let project = tempfile::tempdir().expect("a project must be created");
        let project_root =
            std::fs::canonicalize(project.path()).expect("project identity must resolve");
        std::fs::create_dir(project_root.join("node_modules"))
            .expect("node_modules must be created");
        link_vue(&vue, &project_root.join("node_modules/vue"));
        for input in &case.inputs {
            let file = safe_relative(&input.file);
            let source = safe_relative(&input.source);
            let destination = project_root.join(file);
            std::fs::create_dir_all(destination.parent().expect("an input must have a parent"))
                .expect("input parents must be created");
            std::fs::copy(fixtures.join(source), destination)
                .expect("exact fixture bytes must copy");
        }
        let mut checker = BatchTypeChecker::new(&project_root).expect("the checker must start");
        checker.scan_project().expect("the project must scan");
        let result = checker
            .check_project()
            .expect("the production checker must finish");
        let actual: Vec<Diagnostic> = result
            .diagnostics
            .iter()
            .map(|diagnostic| Diagnostic {
                file: diagnostic
                    .file
                    .strip_prefix(&project_root)
                    .expect("every diagnostic must belong to this exact project")
                    .to_string_lossy()
                    .replace('\\', "/"),
                line: diagnostic.line + 1,
                column: diagnostic.column + 1,
                severity: diagnostic.severity,
                code: diagnostic.code,
                message: String::from(diagnostic.message.as_str()),
            })
            .collect();
        if let Some(capture) = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR") {
            let directory = PathBuf::from(capture).join(name);
            std::fs::create_dir_all(&directory).expect("capture directory must be created");
            std::fs::write(
                directory.join(&case.id).with_extension("json"),
                serde_json::to_vec_pretty(&actual).expect("actual diagnostics must serialize"),
            )
            .expect("actual diagnostic capture must write");
        }
        assert_eq!(
            actual, case.diagnostics,
            "complete diagnostic list for {}",
            case.id
        );
    }
}

fn safe_relative(input: &str) -> PathBuf {
    let path = PathBuf::from(input);
    assert!(
        path.components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
    );
    assert!(!path.as_os_str().is_empty());
    path
}

fn link_vue(source: &Path, destination: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).expect("the real Vue package must link");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, destination).expect("the real Vue package must link");
}
