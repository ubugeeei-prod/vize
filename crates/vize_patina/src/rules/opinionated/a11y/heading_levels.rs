//! a11y/heading-levels
//!
//! Detect heading level skipping (e.g., jumping from `<h1>` to `<h3>`).
//! Skipping heading levels can confuse screen reader users who navigate
//! by headings. Based on markuplint's `heading-levels` rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <h1>Title</h1>
//!   <h3>Subsection</h3>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <h1>Title</h1>
//!   <h2>Section</h2>
//!   <h3>Subsection</h3>
//! </template>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::ir::{ByteRange, TemplateSyntax};
use crate::markup::{MarkupContext, MarkupDocument, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/heading-levels",
    description: "Disallow skipping heading levels",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct HeadingLevels;

struct HeadingInfo {
    level: u8,
    range: ByteRange,
}

fn heading_level(element: &MarkupElement<'_>) -> Option<u8> {
    if element.is_unqualified_tag_exact("h1") {
        Some(1)
    } else if element.is_unqualified_tag_exact("h2") {
        Some(2)
    } else if element.is_unqualified_tag_exact("h3") {
        Some(3)
    } else if element.is_unqualified_tag_exact("h4") {
        Some(4)
    } else if element.is_unqualified_tag_exact("h5") {
        Some(5)
    } else if element.is_unqualified_tag_exact("h6") {
        Some(6)
    } else {
        None
    }
}

impl HeadingLevels {
    fn check_document(ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        let mut headings: Vec<HeadingInfo> = Vec::new();

        document.walk_elements(&mut |element| {
            if let Some(level) = heading_level(&element) {
                headings.push(HeadingInfo {
                    level,
                    range: element.range(),
                });
            }
        });

        // Sort by document order (source offset)
        headings.sort_by_key(|h| h.range.start);

        let mut prev_level: u8 = 0;
        for heading in &headings {
            if prev_level > 0 && heading.level > prev_level + 1 {
                let message = ctx.t_fmt(
                    "a11y/heading-levels.message",
                    &[
                        ("from", &format!("h{prev_level}")),
                        ("to", &format!("h{}", heading.level)),
                    ],
                );
                let help = ctx.t("a11y/heading-levels.help");
                let diag = LintDiagnostic::warn(
                    META.name,
                    message,
                    heading.range.start,
                    heading.range.end,
                )
                .with_help(help.into_owned());
                ctx.report(diag);
            }
            prev_level = heading.level;
        }
    }
}

impl MarkupRule for HeadingLevels {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        Self::check_document(ctx.lint(), document);
    }
}

impl Rule for HeadingLevels {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // Heading state is scoped to each lowered render root, matching the
        // legacy fallback. The direct OXC document covers the whole program.
        true
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let document = MarkupDocument::new(root, TemplateSyntax::Vue);
        Self::check_document(ctx, &document);
    }
}

#[cfg(test)]
mod tests {
    use super::HeadingLevels;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HeadingLevels));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_sequential() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<h1>Title</h1><h2>Section</h2><h3>Sub</h3>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_same_level() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h2>Section A</h2><h2>Section B</h2>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_decrease() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<h1>Title</h1><h2>Section</h2><h3>Sub</h3><h2>Back</h2>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_heading() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h3>Only heading</h3>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_headings() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_skip_h1_to_h3() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1>Title</h1><h3>Subsection</h3>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_skip_h2_to_h4() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1>T</h1><h2>S</h2><h4>Sub</h4>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_skip_h1_to_h4() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1>Title</h1><h4>Deep</h4>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_multiple_skips() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1>T</h1><h3>S</h3><h6>D</h6>"#, "test.vue");
        // h1→h3 skip, h3→h6 skip
        assert_eq!(result.warning_count, 2);
    }
}
