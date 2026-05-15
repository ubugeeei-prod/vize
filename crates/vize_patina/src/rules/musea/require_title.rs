//! musea/require-title
//!
//! Require title attribute in `<art>` block.
//!
//! The title attribute is required for the component gallery to display
//! the component properly.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <art component="./Button.vue">
//!   <!-- missing title -->
//! </art>
//! ```
//!
//! ### Valid
//! ```vue
//! <art title="Button" component="./Button.vue">
//! </art>
//! ```

#![allow(clippy::disallowed_macros)]

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/require-title",
    description: "Require title attribute in <art> block",
    default_severity: Severity::Error,
};

/// Require title in art block
pub struct RequireTitle;

impl MuseaRule for RequireTitle {
    fn meta(&self) -> &'static MuseaRuleMeta {
        &META
    }

    fn check(&self, source: &str, result: &mut MuseaLintResult) {
        // Find <art> block
        let Some(art_start) = source.find("<art") else {
            return; // No art block, handled by another rule
        };

        // Find the end of the opening tag
        let tag_content = &source[art_start..];
        let Some(tag_end) = tag_content.find('>') else {
            return;
        };

        let art_tag = &tag_content[..tag_end];

        // Check for title attribute
        if !has_attribute(art_tag, "title") {
            result.add_diagnostic(
                LintDiagnostic::error(
                    META.name,
                    "Missing required 'title' attribute in <art> block",
                    art_start as u32,
                    (art_start + tag_end) as u32,
                )
                .with_help("Add a title attribute: <art title=\"Component Name\">"),
            );
        }
    }
}

/// Check if a tag has an attribute (simple check)
fn has_attribute(tag: &str, attr_name: &str) -> bool {
    let patterns = [format!("{}=", attr_name), format!("{} =", attr_name)];

    for pattern in patterns {
        if tag.contains(&pattern) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{MuseaLintResult, MuseaRule, RequireTitle};

    #[test]
    fn test_valid_with_title() {
        let source = r#"<art title="Button" component="./Button.vue"></art>"#;
        let rule = RequireTitle;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_missing_title() {
        let source = r#"<art component="./Button.vue"></art>"#;
        let rule = RequireTitle;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_title_with_spaces() {
        let source = r#"<art title = "Button" component="./Button.vue"></art>"#;
        let rule = RequireTitle;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }
}
