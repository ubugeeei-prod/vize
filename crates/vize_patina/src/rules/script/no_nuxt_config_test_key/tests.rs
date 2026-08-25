use super::{META, NoNuxtConfigTestKey};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule};
use serde_json::Value;

const CORPUS: &str = include_str!(
    "../../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/corpus.json"
);
const RECORDING: &str = include_str!(
    "../../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/nuxt-eslint-output.json"
);

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoNuxtConfigTestKey));
    linter.lint(source, 0)
}

fn line_column(source: &str, offset: usize) -> (u64, u64) {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u64 + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, tail)| tail)
        .len() as u64
        + 1;
    (line, column)
}

#[test]
fn metadata_matches_the_nuxt_rule_contract() {
    let meta = NoNuxtConfigTestKey.meta();
    assert_eq!(meta.name, "nuxt/no-nuxt-config-test-key");
    assert_eq!(
        meta.description,
        "Disallow setting `test` key in Nuxt config"
    );
    assert_eq!(meta.default_severity, Severity::Error);
}

#[test]
fn exact_diagnostic_contract_is_non_fixable() {
    let source = "export default defineNuxtConfig({ test: true })";
    let result = lint(source);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);

    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.message,
        "Do not set `test` key in Nuxt config. The test environment is automatically detected."
    );
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "test: true"
    );
    assert!(diagnostic.help.is_none());
    assert!(diagnostic.fix.is_none());
}

#[test]
fn diagnostic_offsets_include_the_sfc_block_offset() {
    let source = "export default { test: false }";
    let mut result = ScriptLintResult::default();
    NoNuxtConfigTestKey.check(source, 41, &mut result);
    let property_start = source.find("test").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, 41 + property_start);
    assert_eq!(
        result.diagnostics[0].end,
        41 + source.find('}').unwrap() as u32 - 1
    );
}

#[test]
fn matches_the_recorded_nuxt_eslint_plugin_oracle() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let recording: Value = serde_json::from_str(RECORDING).unwrap();
    let cases = corpus["noNuxtConfigTestKeyCases"].as_array().unwrap();
    let recorded = recording["noNuxtConfigTestKeyCases"].as_object().unwrap();

    for case in cases {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let upstream = &recorded[id];
        let expected_messages = upstream["messages"].as_array().unwrap();
        let result = lint(source);

        assert_eq!(result.diagnostics.len(), expected_messages.len(), "{id}");
        for (diagnostic, expected) in result.diagnostics.iter().zip(expected_messages) {
            assert_eq!(
                diagnostic.rule_name,
                expected["ruleId"].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(diagnostic.severity, Severity::Error, "{id}");
            assert_eq!(
                diagnostic.message,
                expected["message"].as_str().unwrap(),
                "{id}"
            );
            assert!(diagnostic.fix.is_none(), "{id}");

            let range = expected["range"].as_array().unwrap();
            let start = range[0].as_u64().unwrap();
            let end = range[1].as_u64().unwrap();
            assert_eq!(
                (u64::from(diagnostic.start), u64::from(diagnostic.end)),
                (start, end),
                "diagnostic range for {id}"
            );
            assert_eq!(
                line_column(source, start as usize),
                (
                    expected["line"].as_u64().unwrap(),
                    expected["column"].as_u64().unwrap()
                ),
                "diagnostic start location for {id}"
            );
            assert_eq!(
                line_column(source, end as usize),
                (
                    expected["endLine"].as_u64().unwrap(),
                    expected["endColumn"].as_u64().unwrap()
                ),
                "diagnostic end location for {id}"
            );
        }

        assert_eq!(upstream["fixed"], false, "{id}");
        assert_eq!(upstream["output"], source, "{id}");
        assert_eq!(upstream["secondPassFixed"], false, "{id}");
        assert_eq!(upstream["secondPassOutput"], source, "{id}");
        assert_eq!(
            upstream["secondPassMessageCount"],
            expected_messages.len(),
            "{id}"
        );
        assert_eq!(upstream["secondPassMessagesMatch"], true, "{id}");
    }
}

#[test]
fn uses_utf8_byte_offsets() {
    let source = "const cafe = 'é'\nexport default { test: true }";
    let result = lint(source);
    let start = source.find("test").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(
        &source[result.diagnostics[0].start as usize..result.diagnostics[0].end as usize],
        "test: true"
    );
}
