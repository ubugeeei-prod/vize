//! musea/no-empty-variant
//!
//! Disallow empty variant blocks that have no template content.

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/no-empty-variant",
    description: "Disallow empty <variant> blocks",
    default_severity: Severity::Warning,
};

/// Disallow empty variant blocks
pub struct NoEmptyVariant;

impl MuseaRule for NoEmptyVariant {
    fn meta(&self) -> &'static MuseaRuleMeta {
        &META
    }

    fn check(&self, source: &str, result: &mut MuseaLintResult) {
        let mut search_start = 0;

        while let Some(variant_pos) = source[search_start..].find("<variant") {
            let abs_pos = search_start + variant_pos;
            let remaining = &source[abs_pos..];

            // Find the opening tag end
            let Some(tag_end) = remaining.find('>') else {
                break;
            };

            // Check for self-closing tag
            if remaining[..tag_end].ends_with('/') {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Empty self-closing <variant /> block",
                        abs_pos as u32,
                        (abs_pos + tag_end + 1) as u32,
                    )
                    .with_help("Add template content inside the variant"),
                );
                search_start = abs_pos + tag_end + 1;
                continue;
            }

            // Find the closing tag
            let after_open = &remaining[tag_end + 1..];
            if let Some(close_pos) = after_open.find("</variant>") {
                let content = &after_open[..close_pos];
                let trimmed = content.trim();

                if trimmed.is_empty() {
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            "Empty <variant> block with no content",
                            abs_pos as u32,
                            (abs_pos + tag_end + 1 + close_pos + 10) as u32,
                        )
                        .with_help("Add template content inside the variant"),
                    );
                }

                search_start = abs_pos + tag_end + 1 + close_pos + 10;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MuseaLintResult, MuseaRule, NoEmptyVariant};

    #[test]
    fn test_valid_variant() {
        let source = r#"<variant name="default"><Button /></variant>"#;
        let rule = NoEmptyVariant;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_empty_variant() {
        let source = r#"<variant name="empty"></variant>"#;
        let rule = NoEmptyVariant;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_self_closing_variant() {
        let source = r#"<variant name="empty" />"#;
        let rule = NoEmptyVariant;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 1);
    }
}
