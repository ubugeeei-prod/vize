use super::{assert_success, run, write};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use vize_l0::{String, cstr};

#[derive(Clone, Copy, Deserialize)]
enum Normalizer {
    #[serde(rename = "normalizeInlineStyles")]
    InlineStyles,
    #[serde(rename = "normalizeVForParens")]
    VForParens,
    #[serde(rename = "reindentPreprocessorStyles")]
    PreprocessorStyles,
}

impl Normalizer {
    fn config_key(self) -> &'static str {
        match self {
            Self::InlineStyles => "normalizeInlineStyles",
            Self::VForParens => "normalizeVForParens",
            Self::PreprocessorStyles => "reindentPreprocessorStyles",
        }
    }

    fn flag(self) -> &'static str {
        match self {
            Self::InlineStyles => "--normalize-inline-styles",
            Self::VForParens => "--normalize-v-for-parens",
            Self::PreprocessorStyles => "--reindent-preprocessor-styles",
        }
    }
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    option: Normalizer,
    source: String,
    normalized: String,
}

#[test]
fn direct_normalizers_keep_exact_outputs_and_config_preset_precedence() {
    let fixtures: Vec<Fixture> = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/formatter-cli-normalizers-6098/cases.json"
    ))
    .unwrap();
    assert_eq!(
        fixtures
            .iter()
            .map(|fixture| (fixture.id.as_str(), fixture.option.config_key()))
            .collect::<Vec<_>>(),
        [
            ("inline-static-and-dynamic", "normalizeInlineStyles"),
            ("inline-quoted-semicolon", "normalizeInlineStyles"),
            ("inline-comment-control", "normalizeInlineStyles"),
            ("v-for-alias-and-destructuring", "normalizeVForParens"),
            ("v-for-of-and-parenthesized", "normalizeVForParens"),
            (
                "scss-nested-comments-and-strings",
                "reindentPreprocessorStyles"
            ),
            (
                "less-nested-comments-and-strings",
                "reindentPreprocessorStyles"
            ),
            ("sass-preservation-control", "reindentPreprocessorStyles"),
            ("unbalanced-scss-control", "reindentPreprocessorStyles"),
        ]
    );
    for fixture in fixtures {
        let flag = fixture.option.flag();
        let enabled_flag = cstr!("{flag}=true");
        let disabled_flag = cstr!("{flag}=false");
        let controls: [(&str, bool, &[&str], bool); 6] = [
            ("override-disabled-config", false, &[flag], true),
            (
                "override-vize-preset",
                true,
                &["--preset", "vize", enabled_flag.as_str()],
                true,
            ),
            (
                "override-prettier-preset",
                false,
                &["--preset", "prettier", disabled_flag.as_str()],
                false,
            ),
            (
                "override-enabled-config",
                true,
                &[disabled_flag.as_str()],
                false,
            ),
            ("retain-enabled-config", true, &[], true),
            ("retain-disabled-config", false, &[], false),
        ];
        for (id, configured, extra, enabled) in controls {
            let project = tempfile::tempdir().unwrap();
            let root = project.path();
            let config = crate::support::vite_config::source(
                serde_json::to_vec(&json!({"formatter":{
                    (fixture.option.config_key()): configured,
                }}))
                .unwrap(),
            );
            write(root, "vite.config.mjs", &config);
            write(root, "Case.vue", &fixture.source);
            let expected = if enabled {
                &fixture.normalized
            } else {
                &fixture.source
            };
            let mut args = vec!["fmt", "--write", "Case.vue"];
            args.extend_from_slice(extra);
            for pass in 1..=3 {
                let output = run(root, &args);
                assert_success(&output);
                assert_eq!(
                    fs::read_to_string(root.join("Case.vue"))
                        .unwrap()
                        .as_bytes(),
                    expected.as_bytes(),
                    "{} {id} pass {pass}: {output:?}",
                    fixture.id
                );
                assert_eq!(
                    fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
                    config
                );
            }
            let mut check = vec!["fmt", "--check", "Case.vue"];
            check.extend_from_slice(extra);
            assert_success(&run(root, &check));
            assert_eq!(
                fs::read(root.join("Case.vue")).unwrap(),
                expected.as_bytes()
            );
        }
    }
}
