//! musea/require-title
//!
//! Require a title for an `<art>` block.
//!
//! A title may be provided by `<art title="...">`, `defineArtMeta({ title: "..." })`, or
//! inferred from the imported component rendered by the variants.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <art>
//!   <!-- missing title -->
//! </art>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! import Button from "./Button.vue";
//! </script>
//!
//! <art>
//!   <variant name="Default"><Button /></variant>
//! </art>
//! ```

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta, art_rule_info};
use crate::diagnostic::{LintDiagnostic, Severity};
use vize_l0::cstr;

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/require-title",
    description: "Require a resolvable display title for an <art> block",
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
        let tag_content = source.get(art_start..).unwrap_or_default();
        let Some(tag_end) = tag_content.find('>') else {
            return;
        };

        let art_tag = tag_content.get(..tag_end).unwrap_or_default();

        // Check for title attribute or defineArt metadata.
        if !has_attribute(art_tag, "title") && !art_rule_info(source).has_title {
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
    let patterns = [cstr!("{}=", attr_name), cstr!("{} =", attr_name)];

    for pattern in patterns {
        if tag.contains(pattern.as_str()) {
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

    #[test]
    fn test_valid_with_define_art() {
        let source = r#"<script setup>
import Button from "./Button.vue";
defineArt(Button, { title: "Button" });
</script>
<art></art>"#;
        let rule = RequireTitle;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_with_inferred_component_title() {
        let source = r#"<script setup>
import Button from "./Button.vue";
</script>
<art><variant name="Default"><Button /></variant></art>"#;
        let rule = RequireTitle;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_with_define_art_meta_title() {
        let source = r#"<script setup>
defineArtMeta({ title: "Button" });
</script>
<art></art>"#;
        let rule = RequireTitle;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.error_count, 0);
    }
}
