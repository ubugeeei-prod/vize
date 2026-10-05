#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use std::{fs, path::Path, process::Command};

use serde::Serialize;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, compile_sfc, parse_sfc,
};

const SCRIPT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/build-sfc-errors/WithScript.vue.txt"
);
const TEMPLATE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/build-sfc-errors/TemplateOnly.vue.txt"
);
const V_IF: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/build-sfc-errors/VIf.vue.txt");
const OPTIONS: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/build-sfc-errors/Options.vue.txt");
const MEMO: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/build-sfc-errors/Memo.vue.txt");
const VALID_SCRIPT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/build-sfc-errors/ValidWithScript.vue.txt"
);
const VALID_TEMPLATE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/build-sfc-errors/ValidTemplateOnly.vue.txt"
);

#[test]
fn build_reports_every_returned_sfc_error_and_never_counts_partial_code_as_success() {
    for (name, source, vapor, kind) in [
        ("WithScript.vue", SCRIPT, false, "InvalidExpression"),
        ("TemplateOnly.vue", TEMPLATE, false, "InvalidExpression"),
        ("VIf.vue", V_IF, false, "VIfNoExpression"),
        ("Options.vue", OPTIONS, false, "InvalidExpression"),
        ("Memo.vue", MEMO, true, "v-memo with dependencies"),
    ] {
        let errors = compiler_errors(name, source, vapor);
        assert!(
            !errors.is_empty(),
            "{name} must be refused by the actual compiler"
        );
        assert!(errors.join("\n").contains(kind), "{errors:?}");
        for format in ["js", "json", "stats"] {
            for continue_on_error in [false, true] {
                let project = tempfile::tempdir().unwrap();
                fs::write(project.path().join(name), source).unwrap();
                let output = run(project.path(), name, format, vapor, continue_on_error);
                assert_eq!(output.status.code(), Some(1), "{name}/{format}");
                assert_eq!(output.stdout, b"");
                let extension = if format == "json" { "json" } else { "js" };
                let artifact =
                    Path::new("out").join(name.replace(".vue", &format!(".{extension}")));
                let built = if continue_on_error && format != "stats" {
                    format!("Built: {name} -> {}\n", artifact.display())
                } else {
                    String::new()
                };
                let message = errors.join("\n");
                let lines: String = message
                    .lines()
                    .map(|line| format!("      {line}\n"))
                    .collect();
                let expected = format!(
                    "{built}\n\x1b[31m✗ 1 error(s) occurred:\x1b[0m\n\n  \x1b[31mCompile errors (1):\x1b[0m\n    \x1b[1m{name}\x1b[0m\n{lines}\n\x1b[31m✗ 1 file(s) failed\x1b[0m, 0 compiled in <TIME>s\n"
                );
                assert_eq!(
                    normalize_time(output.stderr),
                    expected,
                    "{name}/{format}/{continue_on_error}"
                );
                if continue_on_error && format == "json" {
                    assert_eq!(
                        fs::read(project.path().join(&artifact)).unwrap(),
                        json_output(name, "", &[message.as_str()])
                    );
                } else if continue_on_error && format == "js" {
                    assert_eq!(fs::read(project.path().join(&artifact)).unwrap(), b"");
                } else {
                    assert!(!project.path().join("out").exists());
                }
                assert_eq!(
                    fs::read_to_string(project.path().join(name)).unwrap(),
                    source
                );
            }
        }
    }
}

#[test]
fn valid_script_and_template_only_controls_keep_whole_output_and_success() {
    for (name, source) in [
        ("WithScript.vue", VALID_SCRIPT),
        ("TemplateOnly.vue", VALID_TEMPLATE),
        ("Memo.vue", MEMO),
    ] {
        let compiled = compiler_result(name, source, false).unwrap();
        assert_eq!(
            serde_json::to_value(&compiled.errors).unwrap(),
            serde_json::json!([])
        );
        assert_eq!(
            serde_json::to_value(&compiled.warnings).unwrap(),
            serde_json::json!([])
        );
        for format in ["js", "json", "stats"] {
            let project = tempfile::tempdir().unwrap();
            fs::write(project.path().join(name), &source).unwrap();
            let output = run(project.path(), name, format, false, false);
            assert_eq!(output.status.code(), Some(0));
            assert_eq!(output.stdout, b"");
            let extension = if format == "json" { "json" } else { "js" };
            let artifact = Path::new("out").join(name.replace(".vue", &format!(".{extension}")));
            let built = if format == "stats" {
                String::new()
            } else {
                format!("Built: {name} -> {}\n", artifact.display())
            };
            assert_eq!(
                normalize_time(output.stderr),
                format!("{built}\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n")
            );
            if format == "js" {
                assert_eq!(
                    fs::read(project.path().join(&artifact)).unwrap(),
                    compiled.code.as_bytes()
                );
            } else if format == "json" {
                assert_eq!(
                    fs::read(project.path().join(&artifact)).unwrap(),
                    json_output(name, &compiled.code, &[])
                );
            } else {
                assert!(!project.path().join("out").exists());
            }
        }
    }
}

#[test]
fn stats_repeated_invalid_sources_preserve_every_failure() {
    let project = tempfile::tempdir().unwrap();
    let mut entries = String::new();
    for name in ["SameA.vue", "SameB.vue"] {
        fs::write(project.path().join(name), SCRIPT).unwrap();
        let lines: String = compiler_errors(name, SCRIPT, false)
            .join("\n")
            .lines()
            .map(|line| format!("      {line}\n"))
            .collect();
        entries.push_str(&format!("    \x1b[1m{name}\x1b[0m\n{lines}"));
    }
    let output = run(project.path(), "Same*.vue", "stats", false, false);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        normalize_time(output.stderr),
        format!(
            "\n\x1b[31m✗ 2 error(s) occurred:\x1b[0m\n\n  \x1b[31mCompile errors (2):\x1b[0m\n{entries}\n\x1b[31m✗ 2 file(s) failed\x1b[0m, 0 compiled in <TIME>s\n"
        )
    );
    assert!(!project.path().join("out").exists());
}

fn compiler_result(
    name: &str,
    source: &str,
    vapor: bool,
) -> Result<SfcCompileResult, vize_atelier_sfc::SfcError> {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: name.into(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut options = SfcCompileOptions::default();
    options.parse.filename = name.into();
    options.script.id = Some(name.into());
    options.template.id = Some(name.into());
    options.style.id = name.into();
    options.vapor = vapor;
    compile_sfc(&descriptor, options)
}

fn compiler_errors(name: &str, source: &str, vapor: bool) -> Vec<String> {
    match compiler_result(name, source, vapor) {
        Ok(result) => result
            .errors
            .into_iter()
            .map(|error| String::from(error.message.as_str()))
            .collect(),
        Err(error) => vec![String::from(error.message.as_str())],
    }
}

fn run(
    root: &Path,
    name: &str,
    format: &str,
    vapor: bool,
    continue_on_error: bool,
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--no-config",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            format,
            name,
        ]);
    if vapor {
        command.arg("--vapor");
    }
    if continue_on_error {
        command.arg("--continue-on-error");
    }
    command.output().unwrap()
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    filename: &'a str,
    code: &'a str,
    css: Option<&'a str>,
    errors: &'a [&'a str],
    warnings: Vec<&'a str>,
    script_lang: &'a str,
    macro_artifacts: Vec<serde_json::Value>,
}

fn json_output(name: &str, code: &str, errors: &[&str]) -> Vec<u8> {
    serde_json::to_vec_pretty(&JsonOutput {
        filename: name,
        code,
        css: None,
        errors,
        warnings: Vec::new(),
        script_lang: "js",
        macro_artifacts: Vec::new(),
    })
    .unwrap()
}

fn normalize_time(bytes: Vec<u8>) -> String {
    let text = String::from_utf8(bytes).unwrap();
    let (head, time) = text.rsplit_once(" compiled in ").unwrap();
    let (time, tail) = time.split_once('s').unwrap();
    let (seconds, decimal) = time.split_once('.').unwrap();
    assert!(!seconds.is_empty() && seconds.bytes().all(|byte| byte.is_ascii_digit()));
    assert_eq!(decimal.len(), 4);
    assert!(decimal.bytes().all(|byte| byte.is_ascii_digit()));
    let elapsed = time.parse::<f64>().unwrap();
    assert!(elapsed.is_finite() && elapsed >= 0.0);
    format!("{head} compiled in <TIME>s{tail}")
}
