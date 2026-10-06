//! Complete native API and source-built CLI contracts for issue #7995.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "whole subprocess fixture vectors")]
#![expect(clippy::disallowed_methods, reason = "fixture strings")]
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "lint_unchecked_indexed_access_cli/support.rs"]
mod support;

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
use vize_patina::{HelpLevel, Linter, Severity};

const RULE: &str = "type/strict-boolean-expressions";
const CORPUS: &str = "tests/_fixtures/differential/linter/unchecked-index-access";

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn read(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join(name)).unwrap()
}

fn json_file(root: &Path, name: &str) -> Value {
    serde_json::from_str(&read(root, name)).unwrap()
}

fn complete_result(result: &vize_patina::LintResult) -> Value {
    json!({
        "filename":result.filename,
        "diagnostics":result.diagnostics.iter().map(|diagnostic| json!({
            "ruleName":diagnostic.rule_name, "severity":diagnostic.severity,
            "message":diagnostic.message, "start":diagnostic.start, "end":diagnostic.end,
            "help":diagnostic.help,
            "labels":diagnostic.labels.iter().map(|label| json!({
                "message":label.message,"start":label.start,"end":label.end
            })).collect::<Vec<_>>(),
            "fix":diagnostic.fix
        })).collect::<Vec<_>>(),
        "errorCount":result.error_count, "warningCount":result.warning_count
    })
}

#[test]
fn authored_index_access_options_reach_native_boolean_types_and_complete_cli_reports() {
    let root = workspace();
    let corpus = root.join(CORPUS);
    let manifest = json_file(&corpus, "corpus.json");
    let catalog_bytes = fs::read(corpus.join("inputs.json")).unwrap();
    assert_eq!(
        support::hash(&catalog_bytes),
        manifest["inputCatalog"]["sha256"]
    );
    let catalog: Value = serde_json::from_slice(&catalog_bytes).unwrap();
    for file in catalog.as_array().unwrap() {
        let bytes = fs::read(corpus.join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(support::hash(&bytes), file["sha256"]);
        assert_eq!(json!(bytes.len()), file["bytes"]);
    }
    let runtime: Option<String> = corsa_requirement::required_or_skip(corsa_path::resolve(root));
    let Some(runtime) = runtime else { return };
    let parent = root.join("target/vize-tests/unchecked-index-access");
    fs::create_dir_all(&parent).unwrap();
    let suite = tempfile::tempdir_in(&parent).unwrap();
    let capture = std::env::var_os("VIZE_UNCHECKED_INDEX_CAPTURE").map_or_else(
        || {
            let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
            root.join("target/nextest")
                .join(profile)
                .join("unchecked-index-access")
        },
        Into::into,
    );
    fs::create_dir_all(&capture).unwrap();
    let version = support::process(
        &capture,
        "native-version",
        Command::new(&runtime).arg("--version"),
    );
    support::save(
        &capture,
        "runtime.json",
        &json!({
            "sourceSha":std::env::var("SOURCE_SHA").ok(),
            "cliBinary":env!("CARGO_BIN_EXE_vize"),
            "cliSha256":support::hash(&fs::read(env!("CARGO_BIN_EXE_vize")).unwrap()),
            "nativeBinary":runtime, "nativeSha256":support::hash(&fs::read(&runtime).unwrap()),
            "catalogSha256":support::hash(&catalog_bytes),
            "corpusSha256":support::hash(&fs::read(corpus.join("corpus.json")).unwrap())
        }),
    );
    assert!(version.status.success());
    assert_eq!(version.stderr, b"");
    let mut completed = Vec::new();
    for case in manifest["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let project = suite.path().join(name);
        fs::create_dir(&project).unwrap();
        let output = capture.join(name);
        let filename = case["filename"].as_str().unwrap();
        let source = read(&corpus, case["source"].as_str().unwrap());
        let config = read(&corpus, case["vizeConfig"].as_str().unwrap());
        support::write(&project, filename, source.as_bytes());
        support::write(&project, "vize.config.json", config.as_bytes());
        support::write(&project, "package.json", b"{\"type\":\"module\"}\n");
        let mut inputs = vec![filename, "vize.config.json", "package.json"];
        for file in case["configFiles"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            support::write(
                &project,
                path,
                read(&corpus, file["input"].as_str().unwrap()).as_bytes(),
            );
            inputs.push(path);
        }
        support::symlink_vue(root, &project);
        for path in &inputs {
            support::write(
                &output.join("inputs"),
                path,
                &fs::read(project.join(path)).unwrap(),
            );
        }
        let authored = project.join(filename);
        let linter = Linter::new()
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_rule_severity_overrides(vec![(RULE.into(), Severity::Error)])
            .with_help_level(HelpLevel::None)
            .with_corsa_path(Some(Path::new(&runtime).to_path_buf()))
            .with_strict_boolean_expressions_options(
                serde_json::from_value(case["options"].clone()).unwrap(),
            );
        let result = linter.lint_sfc(&source, &authored.to_string_lossy());
        let actual = complete_result(&result);
        support::save(&output, "api.json", &actual);
        support::inputs(&project, &output.join("inputs-after-api"), &inputs);
        let generated = support::native_session(&project, &output);
        let mut expected = json_file(&corpus, case["api"].as_str().unwrap());
        // Only the independently owned physical project path substitutes the
        // fixture-relative filename; no diagnostic field comes from output.
        expected["filename"] = json!(authored.to_string_lossy());
        assert_eq!(actual, expected, "{name} complete native API result");
        assert_eq!(
            generated,
            support::expected_session(&case["expectedFlag"]),
            "{name} whole session config"
        );
        Linter::finish_type_aware_lint(
            std::slice::from_ref(&linter),
            std::slice::from_ref(&authored),
        );
        for format in ["json", "plain"] {
            let process = support::process(
                &output,
                format,
                Command::new(env!("CARGO_BIN_EXE_vize"))
                    .current_dir(&project)
                    .env("CORSA_PATH", &runtime)
                    .env("NO_COLOR", "1")
                    .args(["lint", "-f", format, "--help-level", "none", filename]),
            );
            support::inputs(&project, &output.join(format).join("inputs-after"), &inputs);
            assert_eq!(
                process.status.code(),
                case["status"].as_i64().map(|code| code as i32),
                "{name}/{format}"
            );
            assert_eq!(process.stderr, b"", "{name}/{format} complete stderr");
            let stdout = String::from_utf8(process.stdout).unwrap();
            if format == "json" {
                let expected = json_file(&corpus, case["json"].as_str().unwrap());
                assert_eq!(
                    stdout,
                    read(&corpus, case["stdout"].as_str().unwrap()),
                    "{name} complete stdout bytes"
                );
                assert_eq!(serde_json::from_str::<Value>(&stdout).unwrap(), expected);
            } else {
                assert_eq!(
                    stdout,
                    read(&corpus, case["plain"].as_str().unwrap()),
                    "{name} complete plain stdout"
                );
            }
            for path in &inputs {
                assert_eq!(
                    fs::read(project.join(path)).unwrap(),
                    fs::read(output.join("inputs").join(path)).unwrap(),
                    "{name}/{format}/{path} unchanged authored bytes"
                );
            }
        }
        completed.push(name);
        support::save(
            &capture,
            "completed.json",
            &json!({"cases":completed,"apiCount":completed.len(),"cliCount":completed.len()*2,"complete":completed.len()==11}),
        );
    }
    assert_eq!(completed.len(), 11);
}
