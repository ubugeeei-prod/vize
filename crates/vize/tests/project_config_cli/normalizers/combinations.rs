use super::{Normalizer, assert_success, run, write};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use vize_l0::{String, cstr};

#[derive(Deserialize)]
struct Combinations {
    source: String,
    cases: Vec<Combination>,
}

#[derive(Deserialize)]
struct Combination {
    id: String,
    enabled: [bool; 3],
    expected: String,
}

fn fixtures() -> Combinations {
    serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/formatter-cli-normalizers-6098/combinations.json"
    ))
    .unwrap()
}

#[test]
fn normalizer_flags_compose_on_one_sfc_and_override_inverse_configuration() {
    let fixtures = fixtures();
    assert_eq!(
        fixtures
            .cases
            .iter()
            .map(|case| case.enabled)
            .collect::<Vec<_>>(),
        [
            [false, false, false],
            [true, false, false],
            [false, true, false],
            [true, true, false],
            [false, false, true],
            [true, false, true],
            [false, true, true],
            [true, true, true],
        ]
    );
    let normalizers = [
        Normalizer::InlineStyles,
        Normalizer::VForParens,
        Normalizer::PreprocessorStyles,
    ];
    for case in fixtures.cases {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        let config = crate::support::vite_config::source(
            serde_json::to_vec(&json!({"formatter": {
                "normalizeInlineStyles": !case.enabled[0],
                "normalizeVForParens": !case.enabled[1],
                "reindentPreprocessorStyles": !case.enabled[2],
            }}))
            .unwrap(),
        );
        write(root, "vite.config.mjs", &config);
        write(root, "Case.vue", &fixtures.source);
        let flags: Vec<_> = normalizers
            .iter()
            .zip(case.enabled)
            .map(|(normalizer, enabled)| {
                if enabled {
                    cstr!("{}", normalizer.flag())
                } else {
                    cstr!("{}=false", normalizer.flag())
                }
            })
            .collect();
        let mut args = vec!["fmt", "--write", "Case.vue"];
        args.extend(flags.iter().map(|flag| flag.as_str()));
        for pass in 1..=3 {
            let output = run(root, &args);
            assert_success(&output);
            assert_eq!(
                fs::read(root.join("Case.vue")).unwrap(),
                case.expected.as_bytes(),
                "{} pass {pass}: {output:?}",
                case.id
            );
            assert_eq!(
                fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
                config
            );
        }
        let mut check = vec!["fmt", "--check", "Case.vue"];
        check.extend(flags.iter().map(|flag| flag.as_str()));
        assert_success(&run(root, &check));
        assert_eq!(
            fs::read(root.join("Case.vue")).unwrap(),
            case.expected.as_bytes()
        );
    }
}

#[test]
fn normalizer_defaults_preserve_authored_style_and_v_for() {
    let fixtures = fixtures();
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(root, "Case.vue", &fixtures.source);
    assert_success(&run(root, &["fmt", "--write", "Case.vue"]));
    assert_eq!(
        fs::read(root.join("Case.vue")).unwrap(),
        fixtures.source.as_bytes()
    );
    assert_success(&run(root, &["fmt", "--check", "Case.vue"]));
    assert_eq!(
        fs::read(root.join("Case.vue")).unwrap(),
        fixtures.source.as_bytes()
    );
}

#[test]
fn normalizer_flags_are_visible_in_current_cli_help() {
    let project = tempfile::tempdir().unwrap();
    let output = run(project.path(), &["fmt", "--help"]);
    assert_success(&output);
    let help = std::str::from_utf8(&output.stdout).unwrap();
    for flag in [
        "--normalize-inline-styles",
        "--normalize-v-for-parens",
        "--reindent-preprocessor-styles",
    ] {
        assert!(help.contains(flag), "missing {flag}: {help}");
    }
}
