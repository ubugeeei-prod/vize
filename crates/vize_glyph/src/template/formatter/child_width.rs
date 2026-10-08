//! Width of sole interpolation children hugged to their parent tags.
use super::{TemplateFormatter, parse_interpolation_range, suppression::LineJoiner};
use crate::template::helpers::is_whitespace;
use unicode_width::UnicodeWidthStr;

pub(super) fn hugged_closing_name(
    source: &[u8],
    opening: Option<(usize, usize, usize)>,
    previous_end: Option<usize>,
    start: usize,
    end: usize,
) -> Option<&[u8]> {
    let (tag_start, tag_end, name_len) = opening?;
    if tag_end != start || previous_end != Some(start) {
        return None;
    }
    // The successful tag parser already classified this unchanged source name.
    let name = source.get(tag_start + 1..tag_start + 1 + name_len)?;
    let closing = source.get(end..)?;
    (closing.starts_with(b"</")
        && closing.get(2..2 + name_len) == Some(name)
        && closing
            .get(2 + name_len..)?
            .iter()
            .copied()
            .find(|&b| !is_whitespace(b))
            == Some(b'>'))
    .then_some(name)
}

impl TemplateFormatter<'_> {
    #[inline]
    pub(super) fn write_wrapped_hugged_child(
        &self,
        output: &mut Vec<u8>,
        formatted: &str,
        start: usize,
        end: usize,
        depth: usize,
        joiner: &LineJoiner<'_>,
    ) -> bool {
        if !formatted.starts_with("{{ ") || joiner.locks_current_line() {
            return false;
        }
        let Some(name) = joiner.hugged_closing_name(start, end) else {
            return false;
        };
        let line_start = memchr::memrchr2(b'\r', b'\n', output).map_or(0, |pos| pos + 1);
        let line = output.get(line_start..).unwrap_or_default();
        let suffix_width = name.len() + 3 + self.base_depth * self.options.tab_width as usize;
        // Without tabs, UTF-8 byte length bounds Unicode display width. Avoid
        // validation and exact width scans when even that upper bound fits.
        if line.len() + formatted.len() + suffix_width <= self.options.print_width as usize
            && memchr::memchr(b'\t', line).is_none()
            && !formatted.contains('\t')
        {
            return false;
        }
        let width = |value: &str| {
            if value.is_ascii() && !value.contains('\t') {
                value.len()
            } else {
                value.split('\t').map(str::width).sum::<usize>()
                    + value.bytes().filter(|byte| *byte == b'\t').count()
                        * self.options.tab_width as usize
            }
        };
        let line = core::str::from_utf8(line).unwrap_or_default();
        if width(line) + width(formatted) + suffix_width <= self.options.print_width as usize {
            return false;
        }
        let Some((expr_start, expr_end, close)) =
            parse_interpolation_range(formatted.as_bytes(), 0)
        else {
            return false;
        };
        if close != formatted.len() {
            return false;
        }
        // Both newlines are inside mustache syntax. The source-owned parent,
        // child and closing-tag adjacency stays exact; the expression was
        // formatted once already, including its raw literal continuations.
        let parent_depth = depth.saturating_sub(1);
        output.extend_from_slice(b"{{");
        output.extend_from_slice(self.newline);
        let expression = formatted.get(expr_start..expr_end).unwrap_or_default();
        output.extend_from_slice(
            self.render_interpolation_expr_lines(expression, parent_depth)
                .as_bytes(),
        );
        self.write_indent(output, parent_depth);
        output.extend_from_slice(b"}}");
        output.extend_from_slice(self.newline);
        true
    }
}
