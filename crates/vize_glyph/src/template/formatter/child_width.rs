//! Width of sole interpolation children hugged to their parent tags.
use super::{
    TemplateFormatter, format_interpolations_with_vue_version, parse_interpolation_range,
    suppression::LineJoiner,
};
use crate::template::helpers::{is_whitespace, parse_closing_tag};
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
    // A multiline interpolation may be followed by plain inline runtime text
    // before the matching close (`{{ value }}%`). That tail remains
    // byte-adjacent; a later interpolation keeps the established mixed-run
    // depth instead.
    let closing_start = end + memchr::memchr(b'<', source.get(end..)?)?;
    let tail = source.get(end..closing_start)?;
    if tail.iter().any(|byte| matches!(byte, b'\r' | b'\n'))
        || tail.windows(2).any(|window| window == b"{{")
    {
        return None;
    }
    let closing = source.get(closing_start..)?;
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
    /// Width after the attributes when a same-line child is immediately hugged
    /// by its matching closing tag. Including this in the opening-tag decision
    /// lets the formatter use whitespace inside the tag as the safe break point
    /// before runtime text would push the completed element past print width.
    pub(super) fn hugged_inline_suffix_width(
        &self,
        input: &str,
        start: usize,
        tag_name: &str,
    ) -> Option<usize> {
        let source = input.as_bytes();
        let close = start + memchr::memchr(b'<', source.get(start..)?)?;
        let child = input.get(start..close)?;
        if child.is_empty() {
            return None;
        }
        // An authored multiline leading interpolation owns its established
        // layout. Later multiline interpolations in a mixed text run may be
        // normalized to one line, so they still contribute their formatted
        // width to the opening-tag decision.
        if parse_interpolation_range(child.as_bytes(), 0).is_some_and(
            |(expression_start, expression_end, _)| {
                child
                    .as_bytes()
                    .get(expression_start..expression_end)
                    .is_some_and(|expression| {
                        expression.iter().any(|byte| matches!(byte, b'\r' | b'\n'))
                    })
            },
        ) {
            return None;
        }
        let (closing_name, _) = parse_closing_tag(input, close)?;
        if closing_name != tag_name {
            return None;
        }

        let formatted =
            format_interpolations_with_vue_version(child, self.options, self.vue_version);
        // Sole interpolations have a syntax-only break inside `{{ }}` and keep
        // the established #7876 layout. Text and mixed children need the
        // opening tag itself to supply the semantics-preserving break.
        if parse_interpolation_range(formatted.as_bytes(), 0)
            .is_some_and(|(_, _, end)| end == formatted.len())
            || formatted
                .as_bytes()
                .iter()
                .any(|byte| matches!(byte, b'\r' | b'\n'))
        {
            return None;
        }

        Some(1 + formatted.width() + 3 + tag_name.width())
    }

    pub(super) fn hugged_inline_suffix_overflows(
        &self,
        output: &[u8],
        suffix_width: usize,
    ) -> bool {
        let line = output
            .rsplit(|byte| matches!(byte, b'\r' | b'\n'))
            .next()
            .unwrap_or_default();
        let tabs = line.iter().filter(|&&byte| byte == b'\t').count();
        let byte_width = line.len()
            + suffix_width
            + self.base_depth * self.options.tab_width as usize
            + tabs * self.options.tab_width.saturating_sub(1) as usize;
        if byte_width <= self.options.print_width as usize && line.is_ascii() {
            return false;
        }
        let line = core::str::from_utf8(line).unwrap_or_default();
        line.split('\t').map(str::width).sum::<usize>()
            + suffix_width
            + self.base_depth * self.options.tab_width as usize
            + tabs * self.options.tab_width as usize
            > self.options.print_width as usize
    }

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
