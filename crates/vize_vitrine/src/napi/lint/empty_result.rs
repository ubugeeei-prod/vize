//! Empty lint result formatting for native callers.

use vize_patina::{OutputFormat, format_results};

/// Render a valid response when no files match the requested patterns.
pub(super) fn format_empty_lint_output(patterns: &[String], format: OutputFormat) -> String {
    if format == OutputFormat::Text {
        return format!("No .vue or .html files found matching patterns: {patterns:?}");
    }

    format_results(&[], &[], format).into()
}

#[cfg(test)]
mod tests {
    use super::format_empty_lint_output;
    use serde_json::Value;
    use vize_patina::OutputFormat;

    #[test]
    fn structured_empty_output_remains_valid_json() {
        let output = format_empty_lint_output(&["missing".to_owned()], OutputFormat::Json);
        let value: Value = serde_json::from_str(&output).expect("valid JSON output");

        assert_eq!(value, Value::Array(Vec::new()));
    }

    #[test]
    fn text_empty_output_explains_the_unmatched_patterns() {
        let output = format_empty_lint_output(&["missing".to_owned()], OutputFormat::Text);

        assert!(output.contains("No .vue or .html files found"));
        assert!(output.contains("missing"));
    }
}
