use crate::{LintDiagnostic, LintResult};
use crate::{OutputFormat, format_results};

/// ESLint drops a leading byte order mark before reporting positions.
#[test]
fn json_columns_skip_a_leading_byte_order_mark() {
    let source = "\u{feff}<script setup lang=\"ts\">\nconst a = 1\n</script>\n";
    let filename = vize_l0::String::from("Component.vue");
    let start = source.find("lang").unwrap_or_default() as u32;
    let second = source.find("const").unwrap_or_default() as u32;
    let result = LintResult {
        filename: filename.clone(),
        diagnostics: vec![
            LintDiagnostic::warn("vue/block-lang", "lang", start, start + 4),
            LintDiagnostic::warn("vue/no-unused-vars", "a", second, second + 5),
        ],
        error_count: 0,
        warning_count: 2,
    };
    let output = format_results(
        &[result],
        &[(filename, vize_l0::String::from(source))],
        OutputFormat::Json,
    );
    let json: serde_json::Value = serde_json::from_str(&output).unwrap_or_default();
    let messages = &json[0]["messages"];
    assert_eq!(
        [
            (
                &messages[0]["line"],
                &messages[0]["column"],
                &messages[0]["endColumn"]
            ),
            (
                &messages[1]["line"],
                &messages[1]["column"],
                &messages[1]["endColumn"]
            ),
        ],
        [
            (
                &serde_json::json!(1),
                &serde_json::json!(15),
                &serde_json::json!(19)
            ),
            (
                &serde_json::json!(2),
                &serde_json::json!(1),
                &serde_json::json!(6)
            ),
        ]
    );
}
