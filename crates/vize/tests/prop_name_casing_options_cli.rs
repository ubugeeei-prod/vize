//! The public config path consumes the same authored bytes as the pinned provider.
#![expect(clippy::disallowed_types, reason = "complete fixture packets")]
#![expect(clippy::disallowed_macros, reason = "complete fixture packets")]

use crate::support::{config_refusal, vite_config};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const RULE: &str = "vue/prop-name-casing";
const CAMEL_HELP: &str = "Declare props in camelCase; a template may still write them in kebab-case:\n```vue\n<!-- Bad -->\ndefineProps({ 'my-prop': String })\n\n<!-- Good -->\ndefineProps({ myProp: String })\n```";
const SNAKE_HELP: &str = "Declare props in snake_case; a template may still write them in kebab-case:\n```vue\n<!-- Bad -->\ndefineProps({ myProp: String })\n\n<!-- Good -->\ndefineProps({ my_prop: String })\n```";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Finding {
    rule_id: String,
    severity: u32,
    message: String,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    casing: Option<String>,
    findings: Vec<Finding>,
}
#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/prop-name-casing-options-6097")
}
fn corpus() -> Corpus {
    serde_json::from_str(&fs::read_to_string(fixture_root().join("cases.json")).unwrap()).unwrap()
}
fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let file = root.join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
}
fn settings(casing: Option<&str>) -> Value {
    let mut config = json!({ "linter": {
        "preset": "incremental", "rules": { "vue/prop-name-casing": "error" },
        "ruleOptions": {}
    }});
    if let Some(casing) = casing {
        config["linter"]["ruleOptions"][RULE] = json!({ "casing": casing });
    }
    config
}
fn run(root: &Path, files: &[&str], fix: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .args(["lint", "--config", "vite.config.mjs", "--format", "json"])
        .args(files);
    if fix {
        command.arg("--fix");
    }
    command.output().unwrap()
}
fn expected_file(filename: &str, case: &Case) -> Value {
    let help = if case.casing.as_deref() == Some("snake_case") {
        SNAKE_HELP
    } else {
        CAMEL_HELP
    };
    json!({ "file": filename, "messages": case.findings.iter().map(|f| json!({
        "ruleId":f.rule_id,"ruleDocsPath":"docs/content/rules/vue.md","severity":f.severity,
        "message":format!("[vize:{RULE}] {}", f.message.strip_suffix('.').unwrap()),
        "line":f.line,"column":f.column,"endLine":f.end_line,"endColumn":f.end_column,"help":help
    })).collect::<Vec<_>>(),"errorCount":case.findings.len(),"warningCount":0 })
}

#[test]
fn pinned_prop_casing_inputs_follow_public_config_and_have_no_automatic_rename() {
    for case in corpus().cases {
        let project = tempfile::tempdir().unwrap();
        let input = fs::read(fixture_root().join(&case.source)).unwrap();
        let filename = "src/Probe.vue";
        write(project.path(), filename, &input);
        let config = vite_config::source(settings(case.casing.as_deref()).to_string());
        write(project.path(), "vite.config.mjs", &config);
        for fix in [false, true, false] {
            let output = run(project.path(), &[filename], fix);
            assert_eq!(
                output.status.code(),
                Some(i32::from(!case.findings.is_empty())),
                "{}: {output:?}",
                case.id
            );
            assert_eq!(output.stderr, b"", "{}: {output:?}", case.id);
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                json!([expected_file(filename, &case)]),
                "{}",
                case.id
            );
            assert_eq!(fs::read(project.path().join(filename)).unwrap(), input);
            assert_eq!(
                fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
                config
            );
        }
    }
}

#[test]
fn scoped_prop_casing_replaces_the_whole_option_and_empty_object_resets_default() {
    let project = tempfile::tempdir().unwrap();
    let input = fs::read(fixture_root().join("Probe.vue.txt")).unwrap();
    let files = [
        "src/base/Probe.vue",
        "src/override/Probe.vue",
        "src/reset/Probe.vue",
    ];
    for file in files {
        write(project.path(), file, &input);
    }
    let mut config = settings(Some("snake_case"));
    config["entries"] = json!([
        {"files":["src/override/**/*.vue"],"linter":{"ruleOptions":{"vue/prop-name-casing":{"casing":"camelCase"}}}},
        {"files":["src/reset/**/*.vue"],"linter":{"ruleOptions":{"vue/prop-name-casing":{"casing":"snake_case"}}}},
        {"files":["src/reset/**/*.vue"],"linter":{"ruleOptions":{"vue/prop-name-casing":{}}}}
    ]);
    let carrier = vite_config::source(config.to_string());
    write(project.path(), "vite.config.mjs", &carrier);
    let cases = corpus().cases;
    let output = run(project.path(), &files, false);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(output.stderr, b"", "{output:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([
            expected_file(files[0], &cases[2]),
            expected_file(files[1], &cases[1]),
            expected_file(files[2], &cases[1])
        ])
    );
    for file in files {
        assert_eq!(fs::read(project.path().join(file)).unwrap(), input);
    }
    assert_eq!(
        fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
        carrier
    );
}

#[test]
fn prop_casing_options_do_not_enable_an_unselected_or_disabled_rule() {
    let project = tempfile::tempdir().unwrap();
    let input = fs::read(fixture_root().join("Probe.vue.txt")).unwrap();
    write(project.path(), "Probe.vue", &input);
    for rules in [json!({}), json!({"vue/prop-name-casing":"off"})] {
        let mut config = settings(Some("snake_case"));
        config["linter"]["rules"] = rules;
        write(
            project.path(),
            "vite.config.mjs",
            vite_config::source(config.to_string()),
        );
        let output = run(project.path(), &["Probe.vue"], false);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stderr, b"", "{output:?}");
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            json!([
                {"file":"Probe.vue","messages":[],"errorCount":0,"warningCount":0}
            ])
        );
        assert_eq!(fs::read(project.path().join("Probe.vue")).unwrap(), input);
    }
}

#[test]
fn public_prop_casing_config_rejects_invalid_values_without_modifying_inputs() {
    for (invalid, reason) in [
        (json!("snake_case"), "must be object"),
        (
            json!({"casing":"snake-case"}),
            "casing: unsupported native option value",
        ),
        (
            json!({"casing":false}),
            "casing: unsupported native option value",
        ),
        (
            json!({"casing":null}),
            "casing: unsupported native option value",
        ),
        (
            json!({"ignoreProps":["authoredName"]}),
            "unknown option \"ignoreProps\"",
        ),
    ] {
        let project = tempfile::tempdir().unwrap();
        let input = fs::read(fixture_root().join("Probe.vue.txt")).unwrap();
        write(project.path(), "Probe.vue", &input);
        let mut config = settings(None);
        config["linter"]["ruleOptions"][RULE] = invalid;
        let carrier = vite_config::source(config.to_string());
        write(project.path(), "vite.config.mjs", &carrier);
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("RAYON_NUM_THREADS", "1")
            .env("NODE_OPTIONS", config_refusal::NODE_OPTIONS)
            .args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "--fix",
                "Probe.vue",
            ])
            .output()
            .unwrap();
        let directory = project.path().canonicalize().unwrap();
        let exception = if reason.starts_with("casing:") {
            format!("TypeError: vue.linter.ruleOptions.{RULE}.{reason}")
        } else {
            format!("TypeError: vue.linter.ruleOptions.{RULE}: {reason}")
        };
        config_refusal::assert_error(
            &output,
            &directory.join("vite.config.mjs"),
            &directory,
            &exception,
            1,
        );
        assert_eq!(fs::read(project.path().join("Probe.vue")).unwrap(), input);
        assert_eq!(
            fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
            carrier
        );
    }
}
