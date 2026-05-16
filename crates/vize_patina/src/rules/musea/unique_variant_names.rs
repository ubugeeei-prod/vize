//! musea/unique-variant-names
//!
//! Require unique variant names within an art file.

use vize_carton::FxHashSet;

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/unique-variant-names",
    description: "Require unique variant names",
    default_severity: Severity::Error,
};

/// Require unique variant names
pub struct UniqueVariantNames;

impl MuseaRule for UniqueVariantNames {
    fn meta(&self) -> &'static MuseaRuleMeta {
        &META
    }

    fn check(&self, source: &str, result: &mut MuseaLintResult) {
        let mut seen_names: FxHashSet<&str> = FxHashSet::default();
        let mut search_start = 0;

        while let Some(variant_pos) = source[search_start..].find("<variant") {
            let abs_pos = search_start + variant_pos;
            let remaining = &source[abs_pos..];

            let Some(tag_end) = remaining.find('>') else {
                break;
            };

            let variant_tag = &remaining[..tag_end];

            // Extract name attribute value
            if let Some(name) = extract_name_attr(variant_tag) {
                if seen_names.contains(name) {
                    result.add_diagnostic(
                        LintDiagnostic::error(
                            META.name,
                            "Duplicate variant name",
                            abs_pos as u32,
                            (abs_pos + tag_end) as u32,
                        )
                        .with_help("Use a unique name for each variant"),
                    );
                } else {
                    seen_names.insert(name);
                }
            }

            search_start = abs_pos + tag_end;
        }
    }
}

/// Extract the value of the name attribute from a tag
fn extract_name_attr(tag: &str) -> Option<&str> {
    // Find name=" or name='
    let name_pos = tag.find("name=")?;
    let after_eq = &tag[name_pos + 5..];
    let trimmed = after_eq.trim_start();

    if trimmed.is_empty() {
        return None;
    }

    let quote = trimmed.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let after_quote = &trimmed[1..];
    let end_quote = after_quote.find(quote)?;

    Some(&after_quote[..end_quote])
}

#[cfg(test)]
mod tests {
    use super::{extract_name_attr, MuseaLintResult, MuseaRule, UniqueVariantNames};

    #[test]
    fn test_unique_names() {
        let source = r#"<variant name="a"></variant><variant name="b"></variant>"#;
        let rule = UniqueVariantNames;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_duplicate_names() {
        let source = r#"<variant name="same"></variant><variant name="same"></variant>"#;
        let rule = UniqueVariantNames;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_extract_name() {
        assert_eq!(extract_name_attr(r#"<variant name="test""#), Some("test"));
        assert_eq!(extract_name_attr(r#"<variant name='test'"#), Some("test"));
        assert_eq!(extract_name_attr(r#"<variant "#), None);
    }
}
