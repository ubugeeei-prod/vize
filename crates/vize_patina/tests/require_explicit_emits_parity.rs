//! `script/require-explicit-emits` reports exactly what eslint-plugin-vue's
//! `vue/require-explicit-emits` reports on the #10822 reductions.
//! `tests/tooling/patina-require-explicit-emits-oracle.test.ts` re-derives the
//! recorded upstream output from the pinned plugin.

use serde_json::Value;
use vize_patina::Linter;

macro_rules! fixture {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                "../../../tests/_fixtures/differential/linter/require-explicit-emits-parity-10822/",
                $name
            )),
        )
    };
}

const ORACLE: &str = fixture!("eslint-plugin-vue.json").1;
const SOURCES: [(&str, &str); 4] = [
    fixture!("NegativeControls.vue.txt"),
    fixture!("OptionsWithoutEmits.vue.txt"),
    fixture!("SetupWithoutMacro.vue.txt"),
    fixture!("SiblingBlockTypes.vue.txt"),
];

/// ESLint's 1-based line and UTF-16 column of a byte offset.
fn position(source: &str, offset: usize) -> (u64, u64) {
    let before = source.get(..offset).unwrap_or_default();
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
fn findings_match_the_recorded_eslint_plugin_vue_ranges() {
    let oracle: Value = serde_json::from_str(ORACLE).unwrap();
    assert_eq!(oracle["upstream"]["version"], "10.9.2");
    let files = oracle["files"].as_object().unwrap();
    assert_eq!(files.len(), SOURCES.len());
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["script/require-explicit-emits".into()]));
    for (name, source) in SOURCES {
        let result = linter.lint_sfc(source, name.trim_end_matches(".txt"));
        let mut actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let (line, column) = position(source, diagnostic.start as usize);
                let (end_line, end_column) = position(source, diagnostic.end as usize);
                (line, column, end_line, end_column)
            })
            .collect();
        let expected: Vec<_> = files[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                let at = |key: &str| finding[key].as_u64().unwrap();
                (at("line"), at("column"), at("endLine"), at("endColumn"))
            })
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, expected, "{name}");
    }
}
