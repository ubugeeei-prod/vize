//! musea/require-component
//!
//! Require a target component for an `<art>` block.
//!
//! The component is normally inferred from an imported tag rendered by the variants.
//! `<art component="...">` and `defineArt(Component)` remain explicit escape hatches.

use super::{MuseaLintResult, MuseaRule, MuseaRuleMeta, art_rule_info};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: MuseaRuleMeta = MuseaRuleMeta {
    name: "musea/require-component",
    description: "Require a resolvable target component for an <art> block",
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

        let tag_content = source.get(art_start..).unwrap_or_default();
        let Some(tag_end) = tag_content.find('>') else {
            return;
        };

        let art_tag = tag_content.get(..tag_end).unwrap_or_default();

        if !art_tag.contains("component=")
            && !art_tag.contains("component =")
            && !art_rule_info(source).has_component
        {
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

    #[test]
    fn test_valid_with_define_art() {
        let source = r#"<script setup>
import Button from "./Button.vue";
defineArt(Button, { title: "Button" });
</script>
<art></art>"#;
        let rule = RequireComponent;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_inferred_component() {
        let source = r#"<script setup>
import Button from "./Button.vue";
</script>
<art><variant name="Default"><Button /></variant></art>"#;
        let rule = RequireComponent;
        let mut result = MuseaLintResult::default();
        rule.check(source, &mut result);
        assert_eq!(result.warning_count, 0);
    }
}
