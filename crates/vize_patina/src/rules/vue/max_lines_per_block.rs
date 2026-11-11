//! vue/max-lines-per-block
//!
//! Count the lines between a block's opening-tag line and its closing-tag
//! line. An omitted block key is not limited. An empty options object checks
//! nothing.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::MaxLinesPerBlockOptions;
use vize_l0::cstr;

use super::line_count::empty_line_count;

static META: RuleMeta = RuleMeta {
    name: "vue/max-lines-per-block",
    description: "Enforce a maximum number of lines in each Vue SFC block",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/max-lines-per-block`.
#[derive(Default)]
pub struct MaxLinesPerBlock {
    script: Option<u32>,
    template: Option<u32>,
    style: Option<u32>,
    skip_blank_lines: bool,
}

impl MaxLinesPerBlock {
    /// Build the rule from `linter.ruleOptions["vue/max-lines-per-block"]`.
    ///
    /// A missing key, or a limit of zero, does not limit that block.
    pub fn from_options(options: &MaxLinesPerBlockOptions) -> Self {
        Self {
            script: positive(options.script),
            template: positive(options.template),
            style: positive(options.style),
            skip_blank_lines: options.skip_blank_lines == Some(true),
        }
    }
}

fn positive(limit: Option<u32>) -> Option<u32> {
    limit.filter(|value| *value >= 1)
}

impl Rule for MaxLinesPerBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        let mut blocks = Vec::new();
        if let Some(block) = &descriptor.script {
            blocks.push((self.script, block.loc.tag_start, block.loc.tag_end));
        }
        if let Some(block) = &descriptor.script_setup {
            blocks.push((self.script, block.loc.tag_start, block.loc.tag_end));
        }
        if let Some(block) = &descriptor.template {
            blocks.push((self.template, block.loc.tag_start, block.loc.tag_end));
        }
        for block in &descriptor.styles {
            blocks.push((self.style, block.loc.tag_start, block.loc.tag_end));
        }
        let source = ctx.source;
        let skip_blank_lines = self.skip_blank_lines;
        let mut findings = Vec::new();
        for (limit, tag_start, tag_end) in blocks {
            let Some(limit) = limit else {
                continue;
            };
            if tag_end <= tag_start || tag_end > source.len() {
                continue;
            }
            let start_line = ctx.offset_to_line(tag_start as u32);
            let end_line = ctx.offset_to_line(tag_end as u32 - 1);
            let mut line_count = i64::from(end_line) - i64::from(start_line) - 1;
            if skip_blank_lines {
                line_count -= empty_line_count(source.get(tag_start..tag_end).unwrap_or("")) as i64;
            }
            if line_count > i64::from(limit) {
                findings.push((tag_start, tag_end, line_count, limit));
            }
        }
        for (start, end, line_count, limit) in findings {
            ctx.report_in_sfc(LintDiagnostic::warn(
                META.name,
                cstr!("Block has too many lines ({line_count}). Maximum allowed is {limit}."),
                start as u32,
                end as u32,
            ));
        }
    }
}

#[cfg(test)]
mod tests;
