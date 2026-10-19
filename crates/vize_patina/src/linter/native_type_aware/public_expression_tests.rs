//! Complete scoped semantic oracles for public reports #8752–#8755.
//! The original Opinionated/router project has a separate acceptance boundary.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_l0::ToCompactString;

use super::{lint_sfc_with_corsa, tests::corsa_available};
use crate::{
    LintPreset, LintResult, Linter,
    output::{OutputFormat, format_results},
    rules::opinionated::type_aware::NoUnsafeTemplateBinding,
};

const INITIAL: &str = include_str!(
    "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/scoped-semantic-oracles.json"
);
const SUPPLEMENT: &str = include_str!(
    "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/owner-type-supplement.json"
);
const ORIGINAL_CONFIG: &str = include_str!(
    "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/original-config-oracles.json"
);

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/reports-8752-8755")
}

fn authored_cases() -> Vec<Value> {
    [INITIAL, SUPPLEMENT]
        .into_iter()
        .flat_map(|raw| {
            let packet: Value = serde_json::from_str(raw).expect("authored whole packet");
            packet["cases"].as_array().expect("authored cases").clone()
        })
        .collect()
}

fn runtime_available() -> bool {
    let required = std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some();
    let disabled = std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some();
    assert!(
        !(required && disabled),
        "required Corsa execution cannot be disabled"
    );
    if disabled {
        return false;
    }
    let available = corsa_available();
    assert!(
        !required || available,
        "the public expression corpus requires Corsa"
    );
    available
}

fn scoped_linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental).with_rule(Box::new(NoUnsafeTemplateBinding::new()))
}

/// Serialize every diagnostic field and both stats without target filtering.
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
    let result = lint_sfc_with_corsa(linter, source, execution_filename);
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

#[test]
fn frozen_sources_and_complete_oracles_are_preserved() {
    let cases = authored_cases();
    assert_eq!(cases.len(), 25);
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["id"] == "RouterViewSlot")
            .count(),
        1
    );
    for case in &cases {
        let filename = case["filename"].as_str().expect("filename");
        let bytes =
            std::fs::read(fixture_root().join("cases").join(filename)).expect("full source");
        assert_eq!(bytes, case["source"].as_str().expect("source").as_bytes());
        assert_eq!(
            case["config"],
            json!({"linter": {"preset": "incremental", "typeAware": true,
                "rules": {"type/no-unsafe-template-binding": "warn"}}})
        );
    }
    for filename in [
        "DirectiveObjectArg.vue",
        "MultilineAttr.vue",
        "MultilineTernary.vue",
        "RouterViewSlot.vue",
    ] {
        let root = fixture_root();
        assert_eq!(
            std::fs::read(root.join("original-reports").join(filename)).expect("original report"),
            std::fs::read(root.join("cases").join(filename)).expect("scoped source")
        );
    }
}

#[test]
fn complete_scoped_public_expression_packets_use_real_corsa() {
    if !runtime_available() {
        return;
    }
    let linter = scoped_linter();
    let cases = authored_cases();
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for case in cases.iter().filter(|case| case["id"] != "RouterViewSlot") {
        let filename = case["filename"].as_str().expect("filename");
        actual.push(observe_whole_case(&linter, case, filename));
        expected.push(expected_whole_case(case, filename));
    }
    eprintln!(
        "VIZE_PUBLIC_EXPRESSION_SCOPED_PACKETS={}",
        serde_json::to_string(&actual).expect("complete actual packets")
    );
    assert_eq!(actual.len(), 24);
    assert_eq!(actual, expected);
}

#[test]
fn inert_script_markers_remain_complete_in_the_checker_document() {
    let case = authored_cases()
        .into_iter()
        .find(|case| case["id"] == "SafeInertMarkers")
        .expect("frozen inert marker control");
    let source = case["source"].as_str().expect("source");
    let descriptor =
        vize_atelier_sfc::parse_sfc(source, vize_atelier_sfc::SfcParseOptions::default())
            .expect("full SFC");
    let template = descriptor.template.as_ref().expect("template");
    let script = descriptor.script_setup.as_ref().expect("setup");
    let allocator = vize_l0::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template.content.as_ref());
    assert_eq!(
        super::slot_binding_tests::whole_parser_errors(&errors),
        super::slot_binding_tests::expected_parser_notices(
            "SafeInertMarkers",
            template.content.as_ref(),
        ),
    );
    let document = super::document::project_type_aware(
        &descriptor,
        script.content.as_ref(),
        Some(&root),
        script.loc.start as u32,
        template.loc.start as u32,
        "SafeInertMarkers.vue",
    );
    let emitted =
        "  const text = \"void (fake); // VBind\";\n  const cb = \"__vize_cb)((fake))\";\n";
    assert!(document.content.contains(emitted));
}

#[test]
fn original_router_requires_reported_dependency() {
    if !runtime_available() {
        return;
    }
    let Some(project) = ReportedProject::provision() else {
        return;
    };
    assert_reported_router_dependency(project.root());
    let case = authored_cases()
        .into_iter()
        .find(|case| case["id"] == "RouterViewSlot")
        .expect("unaltered conditional router oracle");
    let filename = project.root().join("original-reports/RouterViewSlot.vue");
    let filename = filename.to_str().expect("project path");
    let actual = observe_whole_case(&scoped_linter(), &case, filename);
    eprintln!(
        "VIZE_PUBLIC_EXPRESSION_ROUTER_PACKET={}",
        serde_json::to_string(&actual).expect("complete actual router packet")
    );
    assert_eq!(actual, expected_whole_case(&case, filename));
}

/// The reporter's project: the original sources and compiler config with
/// vue-router 5.3.1 and Vue 3.5.42 installed beside them.
///
/// A `pnpm install` provisions the fixture workspace itself. Without it, the
/// same project is assembled in a temporary directory: vue-router from the
/// authenticated registry members in `reference/`, Vue from the workspace's
/// pnpm store. Only Vue has no frozen copy, so a checkout without any install
/// skips the test, unless `VIZE_TEST_REQUIRE_TSGO` makes that fatal.
enum ReportedProject {
    Installed(PathBuf),
    Assembled(tempfile::TempDir),
}

impl ReportedProject {
    fn provision() -> Option<Self> {
        let fixture = fixture_root();
        let modules = fixture.join("node_modules");
        if ["vue", "vue-router"]
            .iter()
            .all(|name| modules.join(name).join("package.json").is_file())
        {
            return Some(Self::Installed(fixture));
        }
        let Some(vue) = workspace_vue() else {
            assert!(
                std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
                "the reported router project requires Vue 3.5.42; run `pnpm install`"
            );
            return None;
        };
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
        std::fs::create_dir_all(&workspace).expect("test workspace");
        let project = tempfile::Builder::new()
            .prefix("public-expression-8752-")
            .tempdir_in(workspace)
            .expect("reported project");
        let root = project.path();
        let reports = root.join("original-reports");
        std::fs::create_dir_all(&reports).expect("original reports");
        for entry in std::fs::read_dir(fixture.join("original-reports")).expect("reports") {
            let entry = entry.expect("report entry");
            std::fs::copy(entry.path(), reports.join(entry.file_name())).expect("report file");
        }
        let router = root.join("node_modules/vue-router");
        let reference = fixture.join("reference");
        let custody: Value = serde_json::from_slice(
            &std::fs::read(reference.join("custody.json")).expect("router custody"),
        )
        .expect("router custody JSON");
        for (member, entry) in custody["declarationsAndLicense"]
            .as_object()
            .expect("router members")
        {
            let target = router.join(member.strip_prefix("package/").expect("package member"));
            std::fs::create_dir_all(target.parent().expect("member directory")).expect("dir");
            std::fs::copy(
                reference.join(entry["file"].as_str().expect("reference file")),
                target,
            )
            .expect("router member");
        }
        let link = root.join("node_modules/vue");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, &link).expect("Vue link");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, &link).expect("Vue link");
        Some(Self::Assembled(project))
    }

    fn root(&self) -> &Path {
        match self {
            Self::Installed(root) => root,
            Self::Assembled(project) => project.path(),
        }
    }
}

/// Vue 3.5.42 from the workspace's pnpm store, if installed.
fn workspace_vue() -> Option<PathBuf> {
    let store = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../node_modules/.pnpm");
    std::fs::read_dir(store)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name == "vue@3.5.42" || name.starts_with("vue@3.5.42_")
        })
        .map(|entry| entry.path().join("node_modules/vue"))
        .find(|vue| vue.join("package.json").is_file())
}

fn assert_reported_router_dependency(project: &Path) {
    // The original source and compiler config remain below its package root.
    let package_root = project.join("node_modules/vue-router");
    let reference = fixture_root().join("reference");
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

#[test]
fn complete_original_opinionated_config_packets_use_real_corsa() {
    if !runtime_available() {
        return;
    }
    let Some(project) = ReportedProject::provision() else {
        return;
    };
    assert_reported_router_dependency(project.root());
    let packet: Value = serde_json::from_str(ORIGINAL_CONFIG).expect("original config oracles");
    let cases = packet["cases"].as_array().expect("four original cases");
    assert_eq!(cases.len(), 4);
    let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for case in cases {
        assert_eq!(
            case["reportedConfig"]["value"],
            json!({"linter": {"preset": "opinionated", "typeAware": true}})
        );
        let filename = case["filename"]
            .as_str()
            .expect("unchanged original filename");
        let path = project.root().join("original-reports").join(filename);
        assert_eq!(
            std::fs::read(fixture_root().join("original-reports").join(filename))
                .expect("original full source"),
            case["source"].as_str().expect("source").as_bytes()
        );
        for (name, field) in [
            ("vize.config.json", "reportedConfig"),
            ("tsconfig.json", "reportedTsconfig"),
        ] {
            assert_eq!(
                std::fs::read(path.parent().expect("original project").join(name))
                    .expect("literal reporter configuration"),
                case[field]["source"]
                    .as_str()
                    .expect("config bytes")
                    .as_bytes()
            );
        }
        let filename = path.to_str().expect("original project path");
        actual.push(observe_whole_case(&linter, case, filename));
        expected.push(expected_whole_case(case, filename));
    }
    eprintln!(
        "VIZE_PUBLIC_EXPRESSION_ORIGINAL_CONFIG_PACKETS={}",
        serde_json::to_string(&actual).expect("complete actual original-config packets")
    );
    assert_eq!(actual, expected);
}
