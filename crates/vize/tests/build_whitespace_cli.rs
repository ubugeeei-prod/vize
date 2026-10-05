#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "whole CLI fixtures use std strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "whole CLI fixtures use std strings"
)]

use serde::Serialize;
use std::{fs, path::Path, process::Command};
use vize_atelier_core::{WhitespaceStrategy, parser::with_whitespace_mode};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

const SOURCE: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/App.vue.txt");
const JSON: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/vize.config.json.txt"
);
const TYPESCRIPT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/vize.config.ts.txt"
);

#[test]
fn build_config_whitespace_reaches_client_ssr_and_vapor_whole_outputs() {
    for (config_name, explicit, no_config, config, preserve) in [
        ("vize.config.json", false, false, JSON, true),
        ("vize.config.ts", false, false, TYPESCRIPT, true),
        ("explicit.json", true, false, JSON, true),
        ("explicit.ts", true, false, TYPESCRIPT, true),
        ("vize.config.json", false, true, JSON, false),
        (
            "vize.config.json",
            false,
            false,
            r#"{"compiler":{"whitespace":"condense"}}"#,
            false,
        ),
        (
            "vize.config.json",
            false,
            false,
            r#"{"compiler":{"whitespace":"unknown"}}"#,
            false,
        ),
    ] {
        for (ssr, vapor) in [(false, false), (true, false), (false, true)] {
            let expected = compiler_code(SOURCE, "App.vue", preserve, false, ssr, vapor);
            let condensed = compiler_code(SOURCE, "App.vue", false, false, ssr, vapor);
            if preserve {
                assert_ne!(expected, condensed, "ssr={ssr}, vapor={vapor}");
            } else {
                assert_eq!(expected, condensed);
            }
            for format in ["js", "json", "stats"] {
                let project = tempfile::tempdir().unwrap();
                fs::write(project.path().join("App.vue"), SOURCE).unwrap();
                fs::write(project.path().join(config_name), config).unwrap();
                let output = run(
                    project.path(),
                    config_name,
                    explicit,
                    no_config,
                    format,
                    ssr,
                    vapor,
                );
                assert_eq!(
                    output.status.code(),
                    Some(0),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(output.stdout, b"");
                let built = if format == "stats" {
                    String::new()
                } else {
                    let extension = if format == "json" { "json" } else { "js" };
                    format!(
                        "Built: App.vue -> {}\n",
                        Path::new("out").join(format!("App.{extension}")).display()
                    )
                };
                assert_eq!(
                    normalize_time(output.stderr),
                    format!("{built}\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n")
                );
                if format == "stats" {
                    assert!(!project.path().join("out").exists());
                } else if format == "js" {
                    assert_eq!(
                        fs::read(project.path().join("out/App.js")).unwrap(),
                        expected.as_bytes()
                    );
                } else {
                    let json = serde_json::to_vec_pretty(&JsonOutput {
                        filename: "App.vue",
                        code: &expected,
                        css: None,
                        errors: Vec::new(),
                        warnings: Vec::new(),
                        script_lang: "js",
                        macro_artifacts: Vec::new(),
                    })
                    .unwrap();
                    assert_eq!(fs::read(project.path().join("out/App.json")).unwrap(), json);
                }
                assert_eq!(
                    fs::read_to_string(project.path().join("App.vue")).unwrap(),
                    SOURCE
                );
                assert_eq!(
                    fs::read_to_string(project.path().join(config_name)).unwrap(),
                    config
                );
            }
        }
    }
}

#[test]
fn configured_vue2_line_breaks_keep_the_existing_scoped_api_behavior() {
    let source = "<template><p>\n  Label\n  <i />\n</p></template>";
    for (ssr, vapor) in [(false, false), (true, false), (false, true)] {
        let expected = compiler_code(source, "App.vue", false, true, ssr, vapor);
        assert_ne!(
            expected,
            compiler_code(source, "App.vue", false, false, ssr, vapor)
        );
        let project = tempfile::tempdir().unwrap();
        fs::write(project.path().join("App.vue"), source).unwrap();
        fs::write(
            project.path().join("vize.config.json"),
            r#"{"compiler":{"whitespace":"vue2-line-breaks"}}"#,
        )
        .unwrap();
        let output = run(
            project.path(),
            "vize.config.json",
            false,
            false,
            "js",
            ssr,
            vapor,
        );
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, b"");
        assert_eq!(
            fs::read(project.path().join("out/App.js")).unwrap(),
            expected.as_bytes()
        );
    }
}

#[test]
fn whitespace_and_vapor_from_the_same_config_both_apply() {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("App.vue"), SOURCE).unwrap();
    fs::write(
        project.path().join("vize.config.json"),
        r#"{"compiler":{"whitespace":"preserve","vapor":true}}"#,
    )
    .unwrap();
    let output = run(
        project.path(),
        "vize.config.json",
        false,
        false,
        "js",
        false,
        false,
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        normalize_time(output.stderr),
        format!(
            "Built: App.vue -> {}\n\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n",
            Path::new("out").join("App.js").display()
        )
    );
    assert_eq!(
        fs::read(project.path().join("out/App.js")).unwrap(),
        compiler_code(SOURCE, "App.vue", true, false, false, true).as_bytes()
    );
}

fn compiler_code(
    source: &str,
    name: &str,
    preserve: bool,
    legacy: bool,
    ssr: bool,
    vapor: bool,
) -> String {
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
    options.template.ssr = ssr;
    options.style.id = name.into();
    options.vapor = vapor;
    let strategy = if preserve {
        WhitespaceStrategy::Preserve
    } else {
        WhitespaceStrategy::Condense
    };
    let result =
        with_whitespace_mode(strategy, legacy, || compile_sfc(&descriptor, options)).unwrap();
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
    result.code.as_str().into()
}

fn run(
    root: &Path,
    config: &str,
    explicit: bool,
    no_config: bool,
    format: &str,
    ssr: bool,
    vapor: bool,
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            format,
            "App.vue",
        ]);
    if explicit {
        command.args(["--config", config]);
    }
    if no_config {
        command.arg("--no-config");
    }
    if ssr {
        command.arg("--ssr");
    }
    if vapor {
        command.arg("--vapor");
    }
    command.output().unwrap()
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    filename: &'a str,
    code: &'a str,
    css: Option<&'a str>,
    errors: Vec<&'a str>,
    warnings: Vec<&'a str>,
    script_lang: &'a str,
    macro_artifacts: Vec<serde_json::Value>,
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
