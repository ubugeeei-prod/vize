//! musea/valid-variant
//!
//! Validate that variant blocks have required name attribute.

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/valid-variant",
    description: "Require name attribute in <variant> blocks",
    default_severity: Severity::Error,
};

/// Validate variant blocks have name attribute
pub struct ValidVariant;

impl MuseaRule for ValidVariant {
    fn meta(&self) -> &'static MuseaRuleMeta {
        &META
    }

    fn check(&self, source: &str, result: &mut MuseaLintResult) {
        let mut search_start = 0;

        while let Some(variant_pos) = source[search_start..].find("<variant") {
            let abs_pos = search_start + variant_pos;
            let tag_content = &source[abs_pos..];

            if let Some(tag_end) = tag_content.find('>') {
                let variant_tag = &tag_content[..tag_end];

                // Check for name attribute
                if !variant_tag.contains("name=") && !variant_tag.contains("name =") {
                    result.add_diagnostic(
                        LintDiagnostic::error(
                            META.name,
                            "Missing required 'name' attribute in <variant> block",
                            abs_pos as u32,
                            (abs_pos + tag_end) as u32,
                        )
                        .with_help("Add name=\"variant-name\" to the variant"),
                    );
                }

                search_start = abs_pos + tag_end;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MuseaLintResult, MuseaRule, ValidVariant};

    #[test]
    fn test_valid_variant() {
        let source = r#"<variant name="default"><Button /></variant>"#;
        let rule = ValidVariant;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_missing_name() {
        let source = r#"<variant><Button /></variant>"#;
        let rule = ValidVariant;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_multiple_variants() {
        let source = r#"<variant name="a"></variant><variant></variant>"#;
        let rule = ValidVariant;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 1); // Only second is invalid
    }
}
