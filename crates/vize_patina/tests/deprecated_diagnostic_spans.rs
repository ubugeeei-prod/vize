//! Exact filter/tag ranges from pinned Vue ESLint packets and real projects.

use serde_json::Value;
use vize_patina::rules::vue::{NoDeprecatedFilter, NoDeprecatedRouterLinkTagProp};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry};

const ORACLE: &str = include_str!("fixtures/deprecated-diagnostic-spans/oracle.json");

// ESLint columns count UTF-16 code units, while the Rust API uses byte offsets.
fn byte_offset(source: &str, line: &Value, column: &Value) -> u32 {
    let line_index = line.as_u64().unwrap() as usize - 1;
    let column_index = column.as_u64().unwrap() as usize - 1;
    let prefix: usize = source
        .split_inclusive('\n')
        .take(line_index)
        .map(str::len)
        .sum();
    let mut utf16 = 0;
    for (offset, character) in source[prefix..].char_indices() {
        if utf16 == column_index {
            return (prefix + offset) as u32;
        }
        utf16 += character.len_utf16();
    }
    assert_eq!(utf16, column_index);
    source.len() as u32
}

#[test]
fn pinned_originals_and_minimal_cases_match_complete_native_packets() {
    let fixture: Value = serde_json::from_str(ORACLE).unwrap();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedFilter));
    registry.register(Box::new(NoDeprecatedRouterLinkTagProp));
    let linter = Linter::with_registry(registry)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let cases = fixture["captures"].as_array().unwrap();
    assert_eq!(cases.len(), 20);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let packet = &case["packets"][0];
        let diagnostics = packet["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|message| {
                let rule = message["ruleId"].as_str().unwrap();
                let (text, help) = match rule {
                    "vue/no-deprecated-filter" => (
                        "Filters were removed in Vue 3; replace the '|' filter with a method call or computed property",
                        "Replace the filter with a method call or computed property (e.g. {{ capitalize(message) }} instead of {{ message | capitalize }}).",
                    ),
                    "vue/no-deprecated-router-link-tag-prop" => (
                        "The `tag` prop on `<router-link>` was removed in Vue Router 4; use the v-slot API instead",
                        "Remove the `tag` prop and render the element yourself using `v-slot` (it exposes `href`, `navigate`, and `isActive`).",
                    ),
                    _ => panic!("unexpected provider rule: {rule}"),
                };
                LintDiagnostic::error(
                    rule,
                    text,
                    byte_offset(source, &message["line"], &message["column"]),
                    byte_offset(source, &message["endLine"], &message["endColumn"]),
                )
                .with_help(help)
            })
            .collect();
        let expected = LintResult {
            filename: filename.into(),
            diagnostics,
            error_count: packet["errorCount"].as_u64().unwrap() as usize,
            warning_count: packet["warningCount"].as_u64().unwrap() as usize,
        };
        let actual = linter.lint_sfc(source, filename);
        // Compare every public native field, including help, labels and fixes.
        assert_eq!(
            format!("{actual:#?}"),
            format!("{expected:#?}"),
            "{}",
            case["id"]
        );
    }
}
