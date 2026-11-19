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

/// Each top-level text node, trimmed the way JavaScript trims: like
/// eslint-plugin-vue, one report per text node, and a byte order mark is
/// whitespace (ESLint also strips a leading one before parsing). Comments and
/// elements end a text node and are never reported: the rule covers text
/// content only, even an element compiler-sfc drops (an empty custom block
/// or a void `<link>`).
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
        let text_len = markup_start(tail).unwrap_or(tail.len());
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
        if let Some(markup) = slice.get(cursor..).filter(|markup| !markup.is_empty()) {
            cursor += markup_len(markup).max(1);
        }
    }
    ranges
}

/// The offset of the first comment, start tag or end tag in `text`.
fn markup_start(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    memchr::memchr_iter(b'<', bytes).find(|&index| {
        let next = bytes.get(index + 1).copied();
        text.get(index..)
            .is_some_and(|rest| rest.starts_with("<!--"))
            || next.is_some_and(|byte| byte.is_ascii_alphabetic())
            || (next == Some(b'/') && bytes.get(index + 2).is_some_and(u8::is_ascii_alphabetic))
    })
}

/// The length of the comment or element that starts `markup`. An element
/// without its end tag runs to the end, as vue-eslint-parser reads it.
fn markup_len(markup: &str) -> usize {
    if let Some(body) = markup.strip_prefix("<!--") {
        return body.find("-->").map_or(markup.len(), |length| length + 7);
    }
    let closing = markup.starts_with("</");
    let name_start = if closing { 2 } else { 1 };
    let name_len = markup
        .get(name_start..)
        .and_then(|rest| {
            rest.find(|character: char| {
                character.is_ascii_whitespace() || matches!(character, '/' | '>')
            })
        })
        .unwrap_or(markup.len().saturating_sub(name_start));
    let name = markup
        .get(name_start..name_start + name_len)
        .unwrap_or_default();
    let tag_end = start_tag_end(markup).unwrap_or(markup.len());
    let self_closing = markup.get(..tag_end).is_some_and(|tag| tag.ends_with("/>"));
    if closing
        || self_closing
        || VOID_ELEMENTS
            .iter()
            .any(|void| void.eq_ignore_ascii_case(name))
    {
        return tag_end;
    }
    let content = markup.get(tag_end..).unwrap_or_default();
    end_tag(content, name).map_or(markup.len(), |end| tag_end + end)
}

/// The offset just past the `>` that closes the tag at the start of `markup`.
fn start_tag_end(markup: &str) -> Option<usize> {
    let mut quote = None;
    for (index, character) in markup.char_indices() {
        match (quote, character) {
            (Some(delimiter), _) if character == delimiter => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(character),
            (None, '>') => return Some(index + 1),
            _ => {}
        }
    }
    None
}

/// The offset just past `</name ... >` in raw block content.
fn end_tag(content: &str, name: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(found) = content.get(from..)?.find("</") {
        let at = from + found;
        let after = at + 2 + name.len();
        if content
            .get(at + 2..after)
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
            && content
                .get(after..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|character| character.is_ascii_whitespace() || character == '>')
        {
            return content
                .get(after..)?
                .find('>')
                .map(|close| after + close + 1);
        }
        from = at + 2;
    }
    None
}

/// HTML void elements, which never take an end tag.
const VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

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
