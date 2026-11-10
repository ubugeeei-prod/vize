//! Content outside the authored SFC block envelope.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/no-content-outside-block",
    description: "Disallow content outside SFC top-level blocks",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-content-outside-block`.
pub struct NoContentOutsideBlock;

impl Rule for NoContentOutsideBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let ranges = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let mut ranges = Vec::with_capacity(
                usize::from(descriptor.template.is_some())
                    + usize::from(descriptor.script.is_some())
                    + usize::from(descriptor.script_setup.is_some())
                    + descriptor.styles.len()
                    + descriptor.custom_blocks.len(),
            );
            if let Some(block) = &descriptor.template {
                ranges.push((block.loc.tag_start, block.loc.tag_end));
            }
            if let Some(block) = &descriptor.script {
                ranges.push((block.loc.tag_start, block.loc.tag_end));
            }
            if let Some(block) = &descriptor.script_setup {
                ranges.push((block.loc.tag_start, block.loc.tag_end));
            }
            ranges.extend(
                descriptor
                    .styles
                    .iter()
                    .map(|block| (block.loc.tag_start, block.loc.tag_end)),
            );
            ranges.extend(
                descriptor
                    .custom_blocks
                    .iter()
                    .map(|block| (block.loc.tag_start, block.loc.tag_end)),
            );
            ranges.sort_unstable_by_key(|range| range.0);
            ranges
        };

        let mut cursor = 0;
        for (start, end) in ranges {
            if let Some(range) = visible_text_range(ctx.source, cursor, start) {
                report_outside_content(ctx, range);
            }
            cursor = cursor.max(end);
        }
        if let Some(range) = visible_text_range(ctx.source, cursor, ctx.source.len()) {
            report_outside_content(ctx, range);
        }
    }
}

fn visible_text_range(source: &str, start: usize, end: usize) -> Option<ByteRange> {
    let slice = source.get(start..end)?;
    let bytes = slice.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if slice
            .get(cursor..)
            .is_some_and(|tail| tail.starts_with("<!--"))
        {
            cursor += slice
                .get(cursor + 4..)
                .and_then(|tail| tail.find("-->"))
                .map_or(bytes.len() - cursor, |length| length + 7);
            continue;
        }
        let character = slice.get(cursor..)?.chars().next()?;
        if !character.is_whitespace() {
            let content_start = cursor;
            let content_end = slice
                .get(content_start..)?
                .find("<!--")
                .map_or(bytes.len(), |length| content_start + length);
            let trimmed = slice.get(content_start..content_end)?.trim_end();
            return Some(ByteRange {
                start: (start + content_start) as u32,
                end: (start + content_start + trimmed.len()) as u32,
            });
        }
        cursor += character.len_utf8();
    }
    None
}

fn report_outside_content(ctx: &mut LintContext<'_>, range: ByteRange) {
    ctx.warn_at_with_help(
        "Unexpected content outside of the top-level blocks",
        range,
        "Move the content into a top-level block or remove it; Vue drops it during compilation",
    );
}
