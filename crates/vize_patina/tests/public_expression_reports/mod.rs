//! Complete public expression packets in the already selected required-Corsa lane.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_l0::ToCompactString;
use vize_patina::{
    LintPreset, LintResult, Linter,
    output::{OutputFormat, format_results},
    rules::type_aware::NoUnsafeTemplateBinding,
};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/reports-8752-8755")
}

fn whole_api(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({"message": label.message, "start": label.start, "end": label.end})
                })
                .collect();
            json!({
                "rule_name": diagnostic.rule_name,
                "severity": diagnostic.severity,
                "message": diagnostic.message,
                "start": diagnostic.start,
                "end": diagnostic.end,
                "help": diagnostic.help,
                "fix": diagnostic.fix,
                "labels": labels,
            })
        })
        .collect();
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": diagnostics,
    })
}

fn observe_whole_case(linter: &Linter, case: &Value, execution_filename: &str) -> Value {
    let source = case["source"].as_str().expect("source");
    let result = linter.lint_sfc(source, execution_filename);
    let actual_api = whole_api(&result);
    let actual_status = i32::from(result.has_errors());
    let sources = vec![(
        execution_filename.to_compact_string(),
        source.to_compact_string(),
    )];
    let formatted = format_results(&[result], &sources, OutputFormat::Json);
    let actual_cli: Value = serde_json::from_str(&formatted).expect("whole formatter JSON");
    json!({"id": case["id"], "actualApi": actual_api,
        "actualCli": actual_cli, "actualStatus": actual_status})
}

fn expected_whole_case(case: &Value, execution_filename: &str) -> Value {
    let filename = case["filename"].as_str().expect("filename");
    // Adapt only the filename supplied by the caller's real-project route.
    let mut expected_api = case["afterApi"].clone();
    assert_eq!(expected_api["filename"], filename);
    expected_api["filename"] = execution_filename.into();
    let mut expected_cli = case
        .get("afterCli")
        .or_else(|| case.get("afterFormatter"))
        .expect("complete formatter contract")
        .clone();
    assert_eq!(expected_cli[0]["file"], filename);
    expected_cli[0]["file"] = execution_filename.into();
    json!({"id": case["id"], "actualApi": expected_api,
        "actualCli": expected_cli, "actualStatus": case["afterStatus"]})
}

fn assert_reported_router_dependency() {
    // Ordinary Actions installs this isolated workspace's genuine dependencies.
    // The original source and compiler config remain below its package root.
    let project = fixture_root();
    let package_root = project.join("node_modules/vue-router");
    let reference = project.join("reference");
    let custody: Value = serde_json::from_slice(
        &std::fs::read(reference.join("custody.json")).expect("official router source custody"),
    )
    .expect("router custody JSON");
    let members = custody["declarationsAndLicense"]
        .as_object()
        .expect("complete router declarations and license");
    assert_eq!(members.len(), 18);
    for (member, entry) in members {
        let relative = member
            .strip_prefix("package/")
            .expect("registry package member");
        let frozen = std::fs::read(reference.join(entry["file"].as_str().expect("reference file")))
            .expect("complete official package member");
        assert_eq!(
            frozen.len() as u64,
            entry["bytes"].as_u64().expect("member length")
        );
        assert_eq!(
            std::fs::read(package_root.join(relative)).expect("installed genuine package member"),
            frozen,
            "router member {member} differs from the authenticated package"
        );
    }
    let package: Value = serde_json::from_slice(
        &std::fs::read(package_root.join("package.json")).expect("resolved router package"),
    )
    .expect("router package JSON");
    assert_eq!(package["name"], "vue-router");
    assert_eq!(package["version"], "5.3.1");
    let types = package["types"]
        .as_str()
        .or_else(|| package["exports"]["."]["types"].as_str())
        .or_else(|| package["exports"]["."]["types"]["default"].as_str())
        .expect("genuine router declarations");
    assert!(package_root.join(types).is_file(), "missing router types");
    let vue: Value = serde_json::from_slice(
        &std::fs::read(project.join("node_modules/vue/package.json"))
            .expect("fixture's real Vue package"),
    )
    .expect("Vue package JSON");
    assert_eq!(vue["name"], "vue");
    assert_eq!(vue["version"], "3.5.42");
}

pub(super) fn verify_required_reports() {
    assert_eq!(std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref(), Ok("1"));
    assert!(std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none());
    assert_reported_router_dependency();
    let root = fixture_root();
    let scoped = Linter::with_preset(LintPreset::Incremental)
        .with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let original = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    let mut scoped_count = 0;
    let mut original_count = 0;
    for (packet_name, original_config) in [
        ("scoped-semantic-oracles.json", false),
        ("owner-type-supplement.json", false),
        ("original-config-oracles.json", true),
    ] {
        let packet: Value = serde_json::from_slice(
            &std::fs::read(root.join(packet_name)).expect("immutable whole oracle"),
        )
        .expect("complete oracle JSON");
        for case in packet["cases"].as_array().expect("all declared cases") {
            let filename = case["filename"].as_str().expect("filename");
            let use_original_path = original_config || case["id"] == "RouterViewSlot";
            let path = root
                .join(if use_original_path {
                    "original-reports"
                } else {
                    "cases"
                })
                .join(filename);
            assert_eq!(
                std::fs::read(&path).expect("frozen physical source"),
                case["source"].as_str().expect("source").as_bytes()
            );
            if original_config {
                assert_eq!(
                    case["reportedConfig"]["value"],
                    json!({"linter": {"preset":"opinionated", "typeAware":true}})
                );
                for (name, field) in [
                    ("vize.config.json", "reportedConfig"),
                    ("tsconfig.json", "reportedTsconfig"),
                ] {
                    assert_eq!(
                        std::fs::read(root.join("original-reports").join(name))
                            .expect("literal original config"),
                        case[field]["source"]
                            .as_str()
                            .expect("config bytes")
                            .as_bytes()
                    );
                }
                original_count += 1;
            } else {
                assert_eq!(
                    case["config"],
                    json!({"linter": {"preset":"incremental", "typeAware":true, "rules":{"type/no-unsafe-template-binding":"warn"}}})
                );
                scoped_count += 1;
            }
            // Only the caller-owned physical filename adapts. Every diagnostic,
            // count, ordered formatter message and status stays independently frozen.
            let filename = path.to_str().expect("whole physical project path");
            let linter = if original_config { &original } else { &scoped };
            actual.push(
                json!({"role":packet_name,"packet":observe_whole_case(linter,case,filename)}),
            );
            expected.push(json!({"role":packet_name,"packet":expected_whole_case(case,filename)}));
        }
    }
    assert_eq!((scoped_count, original_count), (25, 4));
    eprintln!(
        "VIZE_REQUIRED_PUBLIC_EXPRESSION_PACKETS={}",
        serde_json::to_string(&actual).expect("whole 29 native packets")
    );
    assert_eq!(
        actual, expected,
        "required public linter expression packets"
    );
}
