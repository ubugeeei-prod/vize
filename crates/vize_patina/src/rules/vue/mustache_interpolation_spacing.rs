//! vue/mustache-interpolation-spacing
//!
//! Enforce consistent spacing inside mustache interpolations.
//!
//! ## Examples
//!
//! ### Invalid (default: always)
//! ```vue
//! <div>{{text}}</div>
//! <div>{{ text}}</div>
//! <div>{{text }}</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>{{ text }}</div>
//! <div>{{ foo.bar }}</div>
//! <div>{{ foo + bar }}</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/mustache-interpolation-spacing",
    description: "Enforce consistent spacing inside mustache interpolations",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Spacing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpacingStyle {
    /// Require spaces: {{ foo }}
    #[default]
    Always,
    /// No spaces: {{foo}}
    Never,
}

/// Mustache interpolation spacing rule
pub struct MustacheInterpolationSpacing {
    pub style: SpacingStyle,
}

impl Default for MustacheInterpolationSpacing {
    fn default() -> Self {
        Self {
            style: SpacingStyle::Always,
        }
    }
}

impl Rule for MustacheInterpolationSpacing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        let _content = match &interpolation.content {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(_) => return,
        };

        // Get the raw source to check spacing
        // Note: end.offset is exclusive (points to the character AFTER the last one)
        let start = interpolation.loc.start.offset as usize;
        let end = interpolation.loc.end.offset as usize;

        if end <= start || end > ctx.source.len() {
            return;
        }

        let raw = &ctx.source[start..end];

        // Check if it starts with {{ and ends with }}
        if !raw.starts_with("{{") || !raw.ends_with("}}") {
            return;
        }

        // Extract the inner content (between {{ and }})
        let inner = &raw[2..raw.len() - 2];

        match self.style {
            SpacingStyle::Always => {
                let has_leading_space = inner.starts_with(' ') || inner.starts_with('\n');
                let has_trailing_space = inner.ends_with(' ') || inner.ends_with('\n');

                if !has_leading_space || !has_trailing_space {
                    ctx.warn_with_help(
                        ctx.t("vue/mustache-interpolation-spacing.expected"),
                        &interpolation.loc,
                        ctx.t("vue/mustache-interpolation-spacing.help_expected"),
                    );
                }
            }
            SpacingStyle::Never => {
                let trimmed = inner.trim();
                if inner != trimmed {
                    ctx.warn_with_help(
                        ctx.t("vue/mustache-interpolation-spacing.unexpected"),
                        &interpolation.loc,
                        ctx.t("vue/mustache-interpolation-spacing.help_unexpected"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MustacheInterpolationSpacing;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MustacheInterpolationSpacing::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_spaces() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ text }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_no_spaces() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{text}}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_missing_leading_space() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{text }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_missing_trailing_space() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ text}}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
