//! `vue/v-on-event-hyphenation` reports eslint-plugin-vue's ranges and suggests
//! its autofix name (#10822). `tests/tooling/patina-v-on-hyphenation-oracle.test.ts`
//! re-derives the recorded upstream output from the pinned plugin.

use serde_json::Value;
use vize_patina::Linter;

const SOURCE: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/linter/v-on-hyphenation-suggestion-10822/",
    "LocaleMenu.vue.txt"
));
const ORACLE: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/linter/v-on-hyphenation-suggestion-10822/",
    "eslint-plugin-vue.json"
));

/// ESLint's 1-based line and UTF-16 column of a byte offset.
fn position(offset: u32) -> (u64, u64) {
    let before = SOURCE.get(..offset as usize).unwrap_or_default();
    let line = before.matches('\n').count() as u64 + 1;
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    let column = before
        .get(line_start..)
        .unwrap_or_default()
        .encode_utf16()
        .count() as u64
        + 1;
    (line, column)
}

#[test]
fn ranges_and_suggestions_match_eslint_plugin_vue() {
    let oracle: Value = serde_json::from_str(ORACLE).unwrap();
    assert_eq!(oracle["upstream"]["version"], "10.9.2");
    let expected = oracle["files"]["LocaleMenu.vue.txt"].as_array().unwrap();
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/v-on-event-hyphenation".into()]))
        .lint_sfc(SOURCE, "LocaleMenu.vue");
    assert_eq!(result.diagnostics.len(), expected.len());
    for (diagnostic, finding) in result.diagnostics.iter().zip(expected) {
        let at = |key: &str| finding[key].as_u64().unwrap();
        assert_eq!(
            (position(diagnostic.start), position(diagnostic.end)),
            ((at("line"), at("column")), (at("endLine"), at("endColumn"))),
            "{}",
            finding["message"]
        );
        // Upstream offers no fix for a name with `_`; the suggestion still applies.
        if let Some(fix) = finding["fix"].as_str() {
            let suggestion = diagnostic.message.rsplit('\'').nth(1);
            assert_eq!(suggestion, Some(fix), "{}", diagnostic.message);
        }
    }
}
