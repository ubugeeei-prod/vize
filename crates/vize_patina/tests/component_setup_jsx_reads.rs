//! JSX script references preserve the imported component's lexical owner.

use serde_json::Value;
use std::path::Path;
use vize_patina::Linter;

const RULE: &str = "vue/no-unused-components";
const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-setup-jsx-reads/",
);
const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-setup-jsx-reads/cases.json",
));

#[test]
fn jsx_reads_and_shadowed_or_unused_controls_keep_their_lexical_owner() {
    let manifest: Value = serde_json::from_str(CASES).unwrap();
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    for case in manifest["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let source =
            std::fs::read_to_string(Path::new(ROOT).join(case["source"].as_str().unwrap()))
                .unwrap();
        let result = linter.lint_sfc(&source, "JsxRead.vue");
        assert_eq!(result.error_count, 0, "{id}: {:?}", result.diagnostics);
        assert_eq!(
            result.warning_count as u64,
            case["expectedWarnings"].as_u64().unwrap(),
            "{id}: {:?}",
            result.diagnostics,
        );
        for diagnostic in result.diagnostics {
            assert_eq!(diagnostic.rule_name, RULE, "{id}");
            assert_eq!(
                source.get(diagnostic.start as usize..diagnostic.end as usize),
                Some("Child"),
                "{id}: unused warnings must retain the original import range",
            );
        }
    }
}
