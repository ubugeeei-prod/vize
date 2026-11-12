//! #12923: typescript-eslint `no-unused-vars` test cases as SFC scripts.

use crate::{LintPreset, Linter};
use vize_l0::config::ConfigLintRuleOptions;

const FIXTURE: &str =
    include_str!("../../../../../tests/fixtures/typescript-eslint-no-unused-vars-12923.txt");

#[derive(Default)]
struct Case {
    label: String,
    sfc: String,
    typescript_eslint: Vec<String>,
    vize: Vec<String>,
    kept: Option<String>,
}

fn cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let mut section = "";
    for line in FIXTURE.lines() {
        if line.starts_with('#') && cases.is_empty() {
            continue;
        }
        if let Some(label) = line.strip_prefix("=== ") {
            cases.push(Case {
                label: label.to_string(),
                ..Case::default()
            });
            continue;
        }
        if let Some(name) = line.strip_prefix("--- ") {
            section = match name {
                "sfc" => "sfc",
                "typescript-eslint" => "typescript-eslint",
                "vize" => "vize",
                "kept" => "kept",
                other => panic!("unknown section {other}"),
            };
            continue;
        }
        let case = cases.last_mut().expect("a case header comes first");
        match section {
            "sfc" => {
                case.sfc.push_str(line);
                case.sfc.push('\n');
            }
            "typescript-eslint" => case.typescript_eslint.push(line.to_string()),
            "vize" => case.vize.push(line.to_string()),
            _ => case.kept = Some(line.to_string()),
        }
    }
    cases
}

fn reports(source: &str, options: Option<&ConfigLintRuleOptions>) -> Vec<String> {
    let mut linter = Linter::with_preset(LintPreset::Opinionated);
    if let Some(options) = options {
        linter = linter.with_config_rule_options(options);
    }
    let result = linter.lint_sfc(source, "Case.vue");
    let mut found: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-script-vars")
        .map(|diagnostic| {
            let line = source
                .bytes()
                .take(diagnostic.start as usize)
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            (line, format!("line {line}: {}", diagnostic.message))
        })
        .collect();
    found.sort();
    found.into_iter().map(|(_, text)| text).collect()
}

#[test]
fn all_114_cases_are_present() {
    let cases = cases();
    assert_eq!(cases.len(), 114);
    assert_eq!(cases.iter().filter(|case| case.kept.is_some()).count(), 3);
}

#[test]
fn vize_defaults_keep_the_documented_underscore_patterns() {
    for case in cases() {
        assert_eq!(reports(&case.sfc, None), case.vize, "{}", case.label);
    }
}

#[test]
fn empty_patterns_match_typescript_eslint_defaults() {
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-unused-script-vars": { "varsIgnorePattern": "", "argsIgnorePattern": "" }
    }))
    .unwrap();
    for case in cases() {
        if case.kept.is_some() {
            continue;
        }
        let mut expected = case.typescript_eslint.clone();
        expected.sort();
        let mut found = reports(&case.sfc, Some(&options));
        found.sort();
        assert_eq!(found, expected, "{}", case.label);
    }
}
