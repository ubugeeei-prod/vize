//! vue/mustache-interpolation-spacing
//!
//! Enforce consistent spacing inside mustache interpolations.
//!
//! ## Reported spans
//!
//! Upstream checks the two delimiters independently and reports each on its own
//! delimiter token, so `{{text}}` is two findings — one on `{{`, one on `}}` —
//! not one finding spanning the interpolation. Under `never` the reported span
//! also covers the offending whitespace, because that is the text the fix
//! removes.
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
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::InterpolationNode;

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
        // Note: end.offset is exclusive (points to the character AFTER the last one)
        let start = interpolation.loc.start.offset as usize;
        let end = interpolation.loc.end.offset as usize;
        if end <= start || end > ctx.source.len() {
            return;
        }
        let raw = &ctx.source[start..end];
        if raw.len() < 4 || !raw.starts_with("{{") || !raw.ends_with("}}") {
            return;
        }
        // An interpolation holding only whitespace has no expression, and
        // upstream's `VExpressionContainer[expression!=null]` selector skips it.
        let inner = &raw[2..raw.len() - 2];
        if inner.trim().is_empty() {
            return;
        }

        let opening = interpolation.loc.start.offset;
        let closing = interpolation.loc.end.offset;
        let leading = leading_whitespace(inner);
        let trailing = trailing_whitespace(inner);
        match self.style {
            SpacingStyle::Always => {
                if leading == 0 {
                    report(ctx, EXPECTED_AFTER, HELP_EXPECTED, opening, opening + 2);
                }
                if trailing == 0 {
                    report(ctx, EXPECTED_BEFORE, HELP_EXPECTED, closing - 2, closing);
                }
            }
            SpacingStyle::Never => {
                if leading > 0 {
                    let end = opening + 2 + leading;
                    report(ctx, UNEXPECTED_AFTER, HELP_UNEXPECTED, opening, end);
                }
                if trailing > 0 {
                    let start = closing - 2 - trailing;
                    report(ctx, UNEXPECTED_BEFORE, HELP_UNEXPECTED, start, closing);
                }
            }
        }
    }
}

const EXPECTED_AFTER: &str = "vue/mustache-interpolation-spacing.expected_after";
const EXPECTED_BEFORE: &str = "vue/mustache-interpolation-spacing.expected_before";
const UNEXPECTED_AFTER: &str = "vue/mustache-interpolation-spacing.unexpected_after";
const UNEXPECTED_BEFORE: &str = "vue/mustache-interpolation-spacing.unexpected_before";
const HELP_EXPECTED: &str = "vue/mustache-interpolation-spacing.help_expected";
const HELP_UNEXPECTED: &str = "vue/mustache-interpolation-spacing.help_unexpected";

fn report(ctx: &mut LintContext<'_>, message: &str, help: &str, start: u32, end: u32) {
    let message = ctx.t(message);
    let help = ctx.t(help);
    ctx.warn_at_with_help(message, ByteRange::new(start, end), help);
}

fn leading_whitespace(inner: &str) -> u32 {
    let trimmed = inner.trim_start();
    u32::try_from(inner.len() - trimmed.len()).unwrap_or(0)
}

fn trailing_whitespace(inner: &str) -> u32 {
    let trimmed = inner.trim_end();
    u32::try_from(inner.len() - trimmed.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests;
