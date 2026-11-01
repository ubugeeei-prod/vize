//! Original #11264 argv, complete consumer outputs, and configured overrides.
#![cfg(feature = "glyph")]
#![expect(clippy::disallowed_types, reason = "whole CLI fixture output custody")]

use serde::Deserialize;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter-regressions/space-separated-booleans-11264/Original.vue.txt"
);
const FLAGS: [&str; 3] = ["--semi", "--bracket-spacing", "--sort-attributes"];

fn run(root: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    // nextest retains successful test streams in JUnit: keep every process whole.
    eprintln!(
        "formatter-separated-bools-process:{}",
        serde_json::json!({
            "argv":args,"status":output.status.code(),"success":output.status.success(),
            "stdout":String::from_utf8(output.stdout.clone()).unwrap(),
            "stderr":String::from_utf8(output.stderr.clone()).unwrap(),
            "files":fs::read_dir(root).unwrap().map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file()).map(|path| {
                (path.file_name().unwrap().to_str().unwrap().to_owned(),fs::read_to_string(&path).unwrap())
            }).collect::<std::collections::BTreeMap<_,_>>()        })
    );
    output
}

fn success(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Found 1 file(s)"),
        "{output:?}"
    );
}

fn assert_preferences(output: &str, semi: bool, spacing: bool, sorting: bool) {
    assert!(
        output.contains(if semi {
            "const name = \"ada\";"
        } else {
            "const name = \"ada\"\n"
        }),
        "{output}"
    );
    assert!(
        output.contains(if spacing {
            "{ name: \"ada\" }"
        } else {
            "{name: \"ada\"}"
        }),
        "{output}"
    );
    assert!(
        output.contains(if sorting {
            "id=\"y\" class=\"x\" type=\"button\""
        } else {
            "type=\"button\" class=\"x\" id=\"y\""
        }),
        "{output}"
    );
}

fn round_trip(root: &Path, command: &str, flags: &[&str], configured: bool) -> String {
    fs::write(root.join("a.vue"), ORIGINAL).unwrap();
    let mut argv = vec![command, "a.vue"];
    argv.extend(if configured {
        vec!["--config", "vite.config.mjs"]
    } else {
        vec!["--no-config"]
    });
    argv.push("--write");
    argv.extend_from_slice(flags);
    success(&run(root, &argv));
    let expected = fs::read_to_string(root.join("a.vue")).unwrap();
    success(&run(root, &argv));
    assert_eq!(fs::read_to_string(root.join("a.vue")).unwrap(), expected);
    *argv.iter_mut().find(|arg| **arg == "--write").unwrap() = "--check";
    success(&run(root, &argv));
    assert_eq!(fs::read_to_string(root.join("a.vue")).unwrap(), expected);
    expected
}

#[test]
fn reported_original_spaced_false_matches_the_complete_existing_equals_route() {
    let project = tempfile::tempdir().unwrap();
    // The original report's SFC and the exact two argv sequences remain intact.
    let equals = round_trip(
        project.path(),
        "fmt",
        &[
            "--semi=false",
            "--bracket-spacing=false",
            "--sort-attributes=false",
        ],
        false,
    );
    assert_preferences(&equals, false, false, false);
    let spaced = round_trip(
        project.path(),
        "fmt",
        &[
            "--semi",
            "false",
            "--bracket-spacing",
            "false",
            "--sort-attributes",
            "false",
        ],
        false,
    );
    assert_eq!(spaced, equals);
    let mixed = round_trip(
        project.path(),
        "glyph",
        &[
            "--semi=false",
            "--bracket-spacing",
            "false",
            "--sort-attributes=false",
        ],
        false,
    );
    assert_eq!(mixed, equals);
    for index in 0..3 {
        let mut flags = vec![
            "--semi=true",
            "--bracket-spacing=true",
            "--sort-attributes=true",
        ];
        flags.remove(index);
        flags.extend([*FLAGS.get(index).unwrap(), "false"]);
        let output = round_trip(project.path(), "fmt", &flags, false);
        assert_preferences(&output, index != 0, index != 1, index != 2);
    }
}

#[test]
fn spaced_true_false_and_bare_options_override_config_without_changing_absent_or_no_semi() {
    let project = tempfile::tempdir().unwrap();
    for configured in [false, true] {
        crate::support::vite_config::write(project.path(), serde_json::to_vec(&serde_json::json!({
            "formatter":{"semi":configured,"bracketSpacing":configured,"sortAttributes":configured}
        })).unwrap());
        let config = fs::read(project.path().join("vite.config.mjs")).unwrap();
        let absent = round_trip(project.path(), "fmt", &[], true);
        assert_preferences(&absent, configured, configured, configured);
        for value in ["false", "true"] {
            let spaced: Vec<_> = FLAGS.iter().flat_map(|flag| [*flag, value]).collect();
            let expected = round_trip(project.path(), "fmt", &spaced, true);
            assert_preferences(&expected, value == "true", value == "true", value == "true");
        }
        let bare = round_trip(project.path(), "fmt", &FLAGS, true);
        assert_preferences(&bare, true, true, true);
        let no_semi = round_trip(project.path(), "fmt", &["--no-semi"], true);
        assert_preferences(&no_semi, false, configured, configured);
        assert_eq!(
            fs::read(project.path().join("vite.config.mjs")).unwrap(),
            config
        );
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemicolonCase {
    file: String,
    source: String,
    with_semicolons: String,
    without_semicolons: String,
}
#[derive(Deserialize)]
struct SemicolonCorpus {
    cases: Vec<SemicolonCase>,
}

#[test]
fn spaced_semicolons_preserve_the_prior_complete_independent_js_ts_reference_outputs() {
    let corpus: SemicolonCorpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/semicolon-cli-6098/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 2);
    for case in corpus.cases {
        let project = tempfile::tempdir().unwrap();
        crate::support::vite_config::write(
            project.path(),
            br#"{"formatter":{"singleQuote":true,"printWidth":73}}"#,
        );
        for (value, expected) in [
            ("true", &case.with_semicolons),
            ("false", &case.without_semicolons),
        ] {
            fs::write(project.path().join(&case.file), &case.source).unwrap();
            for action in ["--write", "--write", "--check"] {
                let output = run(
                    project.path(),
                    &[
                        "fmt",
                        &case.file,
                        "--config",
                        "vite.config.mjs",
                        action,
                        "--semi",
                        value,
                    ],
                );
                success(&output);
                assert_eq!(
                    &fs::read_to_string(project.path().join(&case.file)).unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn parser_conflicts_invalid_values_and_a_false_named_file_preserve_all_bytes() {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("a.vue"), ORIGINAL).unwrap();
    for flags in [
        vec!["--semi=invalid"],
        vec!["--bracket-spacing=invalid"],
        vec!["--sort-attributes=invalid"],
        vec!["--semi", "false", "--no-semi"],
        vec!["--no-semi", "--semi", "false"],
        vec!["--semi=true", "--semi", "false"],
        vec!["--semi", "false", "--semi=true"],
    ] {
        let mut argv = vec!["fmt", "a.vue", "--no-config", "--write"];
        argv.extend(flags);
        let output = run(project.path(), &argv);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        assert_eq!(
            fs::read_to_string(project.path().join("a.vue")).unwrap(),
            ORIGINAL
        );
    }
    fs::write(project.path().join("false"), "literal false-named file\n").unwrap();
    let output = run(
        project.path(),
        &["fmt", "--no-config", "--write", "--", "false"],
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "the existing unsupported extension remains refused: {output:?}"
    );
    assert_eq!(
        fs::read_to_string(project.path().join("false")).unwrap(),
        "literal false-named file\n"
    );
    assert_eq!(
        fs::read_to_string(project.path().join("a.vue")).unwrap(),
        ORIGINAL
    );
}
