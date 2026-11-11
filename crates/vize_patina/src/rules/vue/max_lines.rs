//! max-lines
//!
//! Count every line of the file. The default maximum is 300, so enabling the
//! rule without options stays silent on a short SFC. A trailing newline is not
//! an extra line.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::MaxLinesOptions;
use vize_l0::cstr;

use super::line_count::{comment_mask, comment_only, file_lines};

static META: RuleMeta = RuleMeta {
    name: "max-lines",
    description: "Enforce a maximum number of lines in a file",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

const DEFAULT_MAX: u32 = 300;

/// ESLint core `max-lines`, counted over the whole file Vize lints.
pub struct MaxLines {
    max: u32,
    skip_comments: bool,
    skip_blank_lines: bool,
}

impl Default for MaxLines {
    fn default() -> Self {
        Self {
            max: DEFAULT_MAX,
            skip_comments: false,
            skip_blank_lines: false,
        }
    }
}

impl MaxLines {
    /// Build the rule from `linter.ruleOptions["max-lines"]`.
    pub fn from_options(options: &MaxLinesOptions) -> Self {
        match options {
            MaxLinesOptions::Max(max) => Self {
                max: *max,
                skip_comments: false,
                skip_blank_lines: false,
            },
            MaxLinesOptions::Object(object) => Self {
                max: object.max.unwrap_or(DEFAULT_MAX),
                skip_comments: object.skip_comments == Some(true),
                skip_blank_lines: object.skip_blank_lines == Some(true),
            },
        }
    }
}

impl Rule for MaxLines {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let source = ctx.source;
        let mut lines = file_lines(source);
        if self.skip_blank_lines {
            lines.retain(|line| !line.text.trim().is_empty());
        }
        if self.skip_comments {
            let mask = comment_mask(source);
            lines.retain(|line| !comment_only(source, line, &mask));
        }
        let actual = lines.len();
        if actual <= self.max as usize {
            return;
        }
        let Some(first_over) = lines.get(self.max as usize) else {
            return;
        };
        let start = first_over.start;
        ctx.report_in_sfc(LintDiagnostic::warn(
            META.name,
            cstr!(
                "File has too many lines ({actual}). Maximum allowed is {}.",
                self.max
            ),
            start,
            source.len() as u32,
        ));
    }
}

#[cfg(test)]
mod tests;
