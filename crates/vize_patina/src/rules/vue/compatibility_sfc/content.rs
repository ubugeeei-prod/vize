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

        let mut gaps = Vec::with_capacity(ranges.len() + 1);
        let mut cursor = 0;
        for (start, end) in ranges {
            gaps.push((cursor, start));
            cursor = cursor.max(end);
        }
        gaps.push((cursor, ctx.source.len()));
        for (start, end) in gaps {
            for range in visible_text_ranges(ctx.source, start, end) {
                report_outside_content(ctx, range);
            }
        }
    }
}

/// Each text run between comments, trimmed the way JavaScript trims: like
/// eslint-plugin-vue, one report per text node, and a byte order mark is
/// whitespace (ESLint also strips a leading one before parsing).
fn visible_text_ranges(source: &str, start: usize, end: usize) -> Vec<ByteRange> {
    let mut ranges = Vec::new();
    let Some(slice) = source.get(start..end) else {
        return ranges;
    };
    let mut cursor = 0;
    while cursor < slice.len() {
        let Some(tail) = slice.get(cursor..) else {
            break;
        };
        let text_len = tail.find("<!--").unwrap_or(tail.len());
        let text = tail.get(..text_len).unwrap_or_default();
        let trimmed = text.trim_start_matches(is_javascript_whitespace);
        let leading = text.len() - trimmed.len();
        let content = trimmed.trim_end_matches(is_javascript_whitespace);
        if !content.is_empty() {
            let content_start = start + cursor + leading;
            ranges.push(ByteRange {
                start: content_start as u32,
                end: (content_start + content.len()) as u32,
            });
        }
        cursor += text_len;
        if cursor < slice.len() {
            cursor += slice
                .get(cursor + 4..)
                .and_then(|tail| tail.find("-->"))
                .map_or(slice.len() - cursor, |length| length + 7);
        }
    }
    ranges
}

fn is_javascript_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

fn report_outside_content(ctx: &mut LintContext<'_>, range: ByteRange) {
    ctx.warn_at_with_help(
        "Unexpected content outside of the top-level blocks",
        range,
        "Move the content into a top-level block or remove it; Vue drops it during compilation",
    );
}
