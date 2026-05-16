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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::ElementNode;

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

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let props: Vec<_> = element.props.iter().collect();
        if props.is_empty() {
            return;
        }

        let first_prop_start = props[0].loc().start.offset as usize;
        let tag_end = element.loc.start.offset as usize + 1 + element.tag.len();
        self.check_gap(ctx, tag_end, first_prop_start);

        for pair in props.windows(2) {
            let prev_end = pair[0].loc().end.offset as usize;
            let curr_start = pair[1].loc().start.offset as usize;
            self.check_gap(ctx, prev_end, curr_start);
        }
    }
}

impl NoMultiSpaces {
    fn check_gap<'a>(&self, ctx: &mut LintContext<'a>, gap_start: usize, gap_end: usize) {
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
