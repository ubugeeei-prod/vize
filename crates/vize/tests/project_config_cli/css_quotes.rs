use super::{assert_success, run, write};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use vize_l0::String;

#[derive(Deserialize)]
struct QuoteFixture {
    id: String,
    source: String,
    normalized_double: String,
    normalized_single: String,
}

#[test]
fn css_quote_cli_overrides_config_and_presets_with_exact_fixed_points() {
    let fixtures: Vec<QuoteFixture> = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/formatter-css-quotes-cli-6098/cases.json"
    ))
    .unwrap();
    assert_eq!(
        fixtures
            .iter()
            .map(|fixture| fixture.id.as_str())
            .collect::<Vec<_>>(),
        ["css", "scss", "less"]
    );
    for fixture in fixtures {
        let controls: [(&str, bool, &[&str], &str); 4] = [
            (
                "enabled-double",
                false,
                &["--preset", "vize", "--normalize-css-quotes"],
                &fixture.normalized_double,
            ),
            (
                "enabled-single",
                false,
                &[
                    "--preset",
                    "vize",
                    "--normalize-css-quotes=true",
                    "--single-quote",
                ],
                &fixture.normalized_single,
            ),
            (
                "disabled-prettier",
                true,
                &["--preset", "prettier", "--normalize-css-quotes=false"],
                &fixture.source,
            ),
            (
                "disabled-config",
                true,
                &["--normalize-css-quotes=false"],
                &fixture.source,
            ),
        ];
        for (id, configured, extra, expected) in controls {
            let project = tempfile::tempdir().unwrap();
            let root = project.path();
            let config = crate::support::vite_config::source(
                serde_json::to_vec(&json!({"formatter":{
                    "normalizeCssQuotes": configured,
                    "singleQuote": false,
                }}))
                .unwrap(),
            );
            write(root, "vite.config.mjs", &config);
            write(root, "Case.vue", &fixture.source);
            let mut args = vec!["fmt", "--write", "Case.vue"];
            args.extend_from_slice(extra);
            for pass in 1..=3 {
                let output = run(root, &args);
                assert_success(&output);
                assert_eq!(
                    fs::read_to_string(root.join("Case.vue")).unwrap(),
                    expected,
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
            assert_eq!(fs::read_to_string(root.join("Case.vue")).unwrap(), expected);
        }
    }
}
