//! musea/require-component
//!
//! Require component attribute in `<art>` block.

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/require-component",
    description: "Require component attribute in <art> block",
    default_severity: Severity::Warning,
};

/// Require component in art block
pub struct RequireComponent;

impl MuseaRule for RequireComponent {
    fn meta(&self) -> &'static MuseaRuleMeta {
        &META
    }

    fn check(&self, source: &str, result: &mut MuseaLintResult) {
        let Some(art_start) = source.find("<art") else {
            return;
        };

        let tag_content = &source[art_start..];
        let Some(tag_end) = tag_content.find('>') else {
            return;
        };

        let art_tag = &tag_content[..tag_end];

        if !art_tag.contains("component=") && !art_tag.contains("component =") {
            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Missing 'component' attribute in <art> block",
                    art_start as u32,
                    (art_start + tag_end) as u32,
                )
                .with_help("Add component=\"./Component.vue\""),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MuseaLintResult, MuseaRule, RequireComponent};

    #[test]
    fn test_valid() {
        let source = r#"<art title="Button" component="./Button.vue"></art>"#;
        let rule = RequireComponent;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_missing_component() {
        let source = r#"<art title="Button"></art>"#;
        let rule = RequireComponent;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 1);
    }
}
