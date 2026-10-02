//! Real native checker contracts for opt-in boolean conditions and config scope.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "subprocess fixture reports")]
#![expect(clippy::disallowed_methods, reason = "subprocess fixture strings")]
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "lint_strict_boolean_cli/lsp.rs"]
mod lsp;
#[path = "support/lsp_process.rs"]
mod lsp_process;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const RULE: &str = "type/strict-boolean-expressions";
const STRICT: &str = r#"{"allowString":false,"allowNumber":false,"allowNullableObject":false}"#;
const CASES: &[(&str, &str, &str)] = &[
    (
        "member expressions",
        include_str!("fixtures/strict-boolean/MemberConditions.vue"),
        include_str!("fixtures/strict-boolean/MemberConditions.json"),
    ),
    (
        "both script blocks",
        include_str!("fixtures/strict-boolean/DualScript.vue"),
        include_str!("fixtures/strict-boolean/DualScript.json"),
    ),
    (
        "script",
        include_str!("fixtures/strict-boolean/ScriptConditions.vue"),
        include_str!("fixtures/strict-boolean/ScriptConditions.json"),
    ),
    (
        "types",
        include_str!("fixtures/strict-boolean/TypeClassification.vue"),
        include_str!("fixtures/strict-boolean/TypeClassification.json"),
    ),
    (
        "template",
        include_str!("fixtures/strict-boolean/TemplateConditions.vue"),
        include_str!("fixtures/strict-boolean/TemplateConditions.json"),
    ),
    (
        "DOM element",
        include_str!("fixtures/strict-boolean/DomElement.vue"),
        include_str!("fixtures/strict-boolean/DomElement.json"),
    ),
    (
        "template only",
        include_str!("fixtures/strict-boolean/TemplateOnly.vue"),
        include_str!("fixtures/strict-boolean/TemplateOnly.json"),
    ),
];

fn project(source: &str, options: Value) -> Option<tempfile::TempDir> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.parent()?;
    let runtime: String = corsa_requirement::required_or_skip(corsa_path::resolve(workspace))?;
    let cases = workspace.join("target/vize-tests/strict-boolean");
    fs::create_dir_all(&cases).expect("fixture directory");
    let project = tempfile::Builder::new()
        .prefix("boolean-")
        .tempdir_in(cases)
        .expect("project");
    fs::write(project.path().join("App.vue"), source).expect("authored source");
    fs::write(project.path().join("package.json"), r#"{"type":"module"}"#).expect("package");
    fs::write(project.path().join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.vue"]}"#).expect("tsconfig");
    let vue = workspace
        .join("playground/node_modules/vue")
        .canonicalize()
        .expect("frozen Vue");
    fs::create_dir(project.path().join("node_modules")).expect("modules");
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, project.path().join("node_modules/vue")).expect("Vue symlink");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue, project.path().join("node_modules/vue"))
        .expect("Vue symlink");
    fs::write(project.path().join("vize.config.json"), serde_json::to_vec(&json!({
        "typeChecker": { "corsaPath": runtime },
        "linter": { "preset": "incremental", "typeAware": true, "rules": { (RULE): "error" }, "ruleOptions": { (RULE): options } }
    })).expect("config")).expect("write config");
    Some(project)
}

fn lint(root: &Path) -> (Option<i32>, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "--format", "json", "App.vue"])
        .output()
        .expect("lint command");
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{error}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code(), value)
}

#[test]
fn native_boolean_conditions_match_complete_authored_reports() {
    for &(name, source, expected) in CASES {
        let Some(project) = project(source, serde_json::from_str(STRICT).expect("options")) else {
            return;
        };
        let (status, actual) = lint(project.path());
        assert_eq!(
            actual,
            serde_json::from_str::<Value>(expected).expect("oracle"),
            "{name}"
        );
        assert_eq!(status, Some(1), "{name}");
        assert_eq!(
            fs::read_to_string(project.path().join("App.vue")).expect("source"),
            source
        );
    }
}

#[test]
fn defaults_allow_truthy_nullable_literals_and_nullable_objects() {
    let source = include_str!("fixtures/strict-boolean/TruthyLiterals.vue");
    let Some(project) = project(source, json!({})) else {
        return;
    };
    assert_eq!(
        lint(project.path()),
        (
            Some(0),
            serde_json::from_str::<Value>(include_str!(
                "fixtures/strict-boolean/TruthyLiterals.json"
            ))
            .expect("oracle")
        )
    );
}

#[test]
fn ordered_entries_can_override_options_and_keep_explicit_off() {
    let source = include_str!("fixtures/strict-boolean/DomElement.vue");
    let Some(project) = project(source, serde_json::from_str(STRICT).expect("options")) else {
        return;
    };
    let path = project.path().join("vize.config.json");
    let mut config: Value =
        serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
    config["entries"] = json!([{ "files": ["App.vue"], "linter": { "ruleOptions": { (RULE): { "allowNullableObject": true } } } }]);
    fs::write(&path, serde_json::to_vec(&config).expect("overlay")).expect("config");
    assert_eq!(
        lint(project.path()),
        (
            Some(0),
            json!([{ "file":"App.vue", "messages":[], "errorCount":0, "warningCount":0 }])
        )
    );
    config["entries"] = json!([{ "files": ["App.vue"], "linter": { "rules": { (RULE): "off" }, "ruleOptions": { (RULE): { "allowNullableObject": false } } } }]);
    fs::write(
        &path,
        serde_json::to_vec(&config).expect("disabled overlay"),
    )
    .expect("config");
    assert_eq!(
        lint(project.path()),
        (
            Some(0),
            json!([{ "file":"App.vue", "messages":[], "errorCount":0, "warningCount":0 }])
        )
    );
}

#[test]
fn warning_severity_is_preserved_and_options_alone_do_not_enable_the_rule() {
    let source = include_str!("fixtures/strict-boolean/DomElement.vue");
    let Some(project) = project(source, serde_json::from_str(STRICT).expect("options")) else {
        return;
    };
    let path = project.path().join("vize.config.json");
    let mut config: Value =
        serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
    config["linter"]["rules"][RULE] = json!("warn");
    fs::write(&path, serde_json::to_vec(&config).expect("warning config")).expect("config");
    let mut expected: Value =
        serde_json::from_str(include_str!("fixtures/strict-boolean/DomElement.json"))
            .expect("oracle");
    expected[0]["messages"][0]["severity"] = json!(1);
    expected[0]["errorCount"] = json!(0);
    expected[0]["warningCount"] = json!(1);
    assert_eq!(lint(project.path()), (Some(0), expected));
    config["linter"]["rules"] = json!({});
    fs::write(&path, serde_json::to_vec(&config).expect("opt-in config")).expect("config");
    assert_eq!(
        lint(project.path()),
        (
            Some(0),
            json!([{ "file":"App.vue", "messages":[], "errorCount":0, "warningCount":0 }])
        )
    );
}

#[test]
fn invalid_boolean_allowance_cannot_write_authored_sources() {
    let source = include_str!("fixtures/strict-boolean/DomElement.vue");
    let Some(project) = project(source, json!({ "allowNullableObject": "yes" })) else {
        return;
    };
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["lint", "--fix", "App.vue"])
        .output()
        .expect("lint command");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        fs::read_to_string(project.path().join("App.vue")).expect("protected source"),
        source
    );
}

#[test]
fn every_nullable_allowance_changes_only_its_authored_conditions() {
    let source = include_str!("fixtures/strict-boolean/TypeClassification.vue");
    for (option, allowed_lines) in [
        ("allowNullableObject", &[6u64][..]),
        ("allowNullableString", &[7][..]),
        ("allowNullableNumber", &[8][..]),
        ("allowNullableBoolean", &[9][..]),
        ("allowAny", &[10, 11][..]),
        ("allowNullableEnum", &[13][..]),
        ("allowString", &[17][..]),
    ] {
        let mut options: Value = serde_json::from_str(STRICT).expect("strict options");
        options[option] = json!(true);
        let Some(project) = project(source, options) else {
            return;
        };
        let mut expected: Value = serde_json::from_str(include_str!(
            "fixtures/strict-boolean/TypeClassification.json"
        ))
        .expect("authored oracle");
        let messages = expected[0]["messages"].as_array_mut().expect("messages");
        let original_count = messages.len();
        messages.retain(|message| !allowed_lines.iter().any(|line| message["line"] == *line));
        let count = messages.len();
        assert_eq!(
            original_count - count,
            allowed_lines.len(),
            "{option} fixture lines"
        );
        expected[0]["errorCount"] = json!(count);
        assert_eq!(lint(project.path()), (Some(1), expected), "{option}");
    }
    let source = include_str!("fixtures/strict-boolean/ScriptConditions.vue");
    let Some(project) = project(source, json!({})) else {
        return;
    };
    assert_eq!(
        lint(project.path()),
        (
            Some(0),
            json!([{ "file":"App.vue", "messages":[], "errorCount":0, "warningCount":0 }])
        )
    );
}
