//! vue/no-multi-spaces
//!
//! Disallow multiple consecutive spaces in template.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div  class="foo"></div>
//! <div class="foo"  id="bar"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div class="foo"></div>
//! <div class="foo" id="bar"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::ir::ByteRange;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-multi-spaces",
    description: "Disallow multiple consecutive spaces",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Disallow multiple spaces
pub struct NoMultiSpaces {
    /// Ignore properties (v-if, v-for expressions)
    pub ignore_properties: bool,
}

impl Default for NoMultiSpaces {
    fn default() -> Self {
        Self {
            ignore_properties: true,
        }
    }
}

impl Rule for NoMultiSpaces {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        self.check_element(ctx, &MarkupElement::new(element));
    }
}

impl MarkupRule for NoMultiSpaces {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.check_element(ctx.lint(), element);
    }
}

impl NoMultiSpaces {
    fn check_element(&self, ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        let mut previous_binding_end = None;

        element.walk_opening_item_ranges(&mut |range| {
            let gap_start = previous_binding_end
                .unwrap_or_else(|| tag_name_end_hint(element.range(), element.tag()));
            self.check_gap(ctx, gap_start, range.start);
            previous_binding_end = Some(range.end);
        });
    }

    fn check_gap(&self, ctx: &mut LintContext<'_>, gap_start: u32, gap_end: u32) {
        let gap_start = gap_start as usize;
        let gap_end = gap_end as usize;
        let gap_start = first_whitespace_offset(ctx.source, gap_start, gap_end);
        if gap_end <= gap_start {
            return;
        }

        let gap = &ctx.source[gap_start..gap_end];
        if !is_invalid_gap(gap) {
            return;
        }

        let fix = Fix::new(
            "Replace multiple spaces with single space",
            TextEdit::replace(gap_start as u32, gap_end as u32, " "),
        );

        ctx.report(
            LintDiagnostic::warn(
                META.name,
                "Multiple consecutive spaces",
                gap_start as u32,
                gap_end as u32,
            )
            .with_fix(fix),
        );
    }
}

fn tag_name_end_hint(range: ByteRange, tag: &str) -> u32 {
    // For JSX member/namespaced tags the facade tag can be shorter than the
    // written tag. Starting early is fine because `first_whitespace_offset`
    // advances to the actual gap before checking it.
    range.start.saturating_add(1 + tag.len() as u32)
}

fn is_invalid_gap(gap: &str) -> bool {
    gap.len() > 1
        && gap
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t'))
        && !gap
            .as_bytes()
            .iter()
            .any(|byte| matches!(byte, b'\n' | b'\r'))
}

fn first_whitespace_offset(source: &str, start: usize, end: usize) -> usize {
    let mut offset = start;
    let bytes = source.as_bytes();
    while offset < end && !matches!(bytes[offset], b' ' | b'\t' | b'\n' | b'\r') {
        offset += 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::NoMultiSpaces;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoMultiSpaces::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_single_space() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo" id="bar"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_multiple_spaces() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo"  id="bar"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
    }

    #[test]
    fn test_invalid_multiple_spaces_before_first_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div  class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
    }

    #[test]
    fn test_valid_multiline_attributes() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button
  class="btn"
  :disabled="isDisabled"
>
</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
