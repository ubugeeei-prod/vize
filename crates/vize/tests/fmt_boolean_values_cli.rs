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

fn changed(file: &str) -> vize_l0::String {
    vize_l0::cstr!(
        "Found 1 file(s)\nReformatted: {file}\n\nFormatted 1 file(s)\n  1 file(s) reformatted\n"
    )
}
const UNCHANGED: &str = "Found 1 file(s)\n\nFormatted 1 file(s)\n  1 file(s) unchanged\n";
const CHECKED: &str = "Found 1 file(s)\n\nChecked 1 file(s)\n  1 file(s) already formatted\n";

fn success(output: &Output, expected_stderr: &str) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, expected_stderr.as_bytes());
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
    success(&run(root, &argv), changed("a.vue").as_str());
    let expected = fs::read_to_string(root.join("a.vue")).unwrap();
    success(&run(root, &argv), UNCHANGED);
    assert_eq!(fs::read_to_string(root.join("a.vue")).unwrap(), expected);
    *argv.iter_mut().find(|arg| **arg == "--write").unwrap() = "--check";
    success(&run(root, &argv), CHECKED);
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
    let all_true = round_trip(
        project.path(),
        "fmt",
        &[
            "--semi=true",
            "--bracket-spacing=true",
            "--sort-attributes=true",
        ],
        false,
    );
    assert_ne!(equals, all_true);
    for index in 0..3 {
        let mut flags = vec![
            "--semi=true",
            "--bracket-spacing=true",
            "--sort-attributes=true",
        ];
        flags.remove(index);
        flags.extend([*FLAGS.get(index).unwrap(), "false"]);
        let output = round_trip(project.path(), "fmt", &flags, false);
        let mut equals_flags = vec![
            "--semi=true",
            "--bracket-spacing=true",
            "--sort-attributes=true",
        ];
        *equals_flags.get_mut(index).unwrap() = *[
            "--semi=false",
            "--bracket-spacing=false",
            "--sort-attributes=false",
        ]
        .get(index)
        .unwrap();
        let control = round_trip(project.path(), "fmt", &equals_flags, false);
        assert_eq!(output, control);
        assert_ne!(output, all_true);
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
        let configured_flags = if configured {
            [
                "--semi=true",
                "--bracket-spacing=true",
                "--sort-attributes=true",
            ]
        } else {
            [
                "--semi=false",
                "--bracket-spacing=false",
                "--sort-attributes=false",
            ]
        };
        let configured_control = round_trip(project.path(), "fmt", &configured_flags, true);
        assert_eq!(absent, configured_control);
        for value in ["false", "true"] {
            let spaced: Vec<_> = FLAGS.iter().flat_map(|flag| [*flag, value]).collect();
            let expected = round_trip(project.path(), "fmt", &spaced, true);
            let equals_flags = if value == "true" {
                [
                    "--semi=true",
                    "--bracket-spacing=true",
                    "--sort-attributes=true",
                ]
            } else {
                [
                    "--semi=false",
                    "--bracket-spacing=false",
                    "--sort-attributes=false",
                ]
            };
            let control = round_trip(project.path(), "fmt", &equals_flags, true);
            assert_eq!(expected, control);
        }
        let bare = round_trip(project.path(), "fmt", &FLAGS, true);
        let true_control = round_trip(
            project.path(),
            "fmt",
            &[
                "--semi=true",
                "--bracket-spacing=true",
                "--sort-attributes=true",
            ],
            true,
        );
        assert_eq!(bare, true_control);
        let no_semi = round_trip(project.path(), "fmt", &["--no-semi"], true);
        let semi_control = round_trip(project.path(), "fmt", &["--semi=false"], true);
        assert_eq!(no_semi, semi_control);
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
            let changed = changed(&case.file);
            for (action, stderr) in [
                ("--write", changed.as_str()),
                ("--write", UNCHANGED),
                ("--check", CHECKED),
            ] {
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
                success(&output, stderr);
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
