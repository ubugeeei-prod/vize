//! Width of sole interpolation children hugged to their parent tags.
use super::{
    TemplateFormatter, format_interpolations_with_vue_version, parse_interpolation_range,
    suppression::{ChunkJoin, LineJoiner},
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
    #[inline(always)]
    pub(super) fn hugged_inline_suffix_width(
        &self,
        input: &str,
        start: usize,
        tag_name: &str,
    ) -> Option<usize> {
        let source = input.as_bytes();
        let close = start + memchr::memchr(b'<', source.get(start..)?)?;
        let child = input.get(start..close)?;
        let child_bytes = child.as_bytes();
        if child.is_empty() {
            return None;
        }
        if source.get(close + 1) != Some(&b'/') {
            return None;
        }
        // An authored multiline leading interpolation owns its established
        // layout. Later multiline interpolations in a mixed text run may be
        // normalized to one line, so they still contribute their formatted
        // width to the opening-tag decision.
        if child_bytes.iter().any(|byte| matches!(byte, b'\r' | b'\n'))
            && authored_multiline_child_owns_layout(child_bytes)
        {
            return None;
        }
        let (closing_name, _) = parse_closing_tag(input, close)?;
        if closing_name != tag_name {
            return None;
        }

        // Sole interpolations are deliberately excluded below. Detect them
        // before formatting so the opening-tag width probe does not parse and
        // print the same JavaScript expression that the text pass owns.
        if parse_interpolation_range(child_bytes, 0)
            .is_some_and(|(_, _, end)| end == child_bytes.len())
        {
            return None;
        }
        // Interpolation formatting is identity for brace-free text. Measure
        // it directly instead of allocating a duplicate child string.
        if memchr::memchr(b'{', child_bytes).is_none() {
            return Some(1 + child.width() + 3 + tag_name.width());
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

    #[inline(always)]
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
    pub(super) fn write_wrapped_text_interpolation(
        &self,
        output: &mut Vec<u8>,
        formatted: &str,
        source_range: (usize, usize),
        depth: usize,
        joiner: &LineJoiner<'_>,
        join: ChunkJoin,
    ) -> bool {
        let (start, end) = source_range;
        if joiner.locks_current_line() {
            return false;
        }
        let Some(first_open) = memchr::memmem::find(formatted.as_bytes(), b"{{") else {
            return false;
        };
        let bytes = formatted.as_bytes();
        let Some((expr_start, expr_end, close)) = parse_interpolation_range(bytes, first_open)
        else {
            return false;
        };
        if memchr::memmem::find(bytes.get(close..).unwrap_or_default(), b"{{").is_some() {
            // Mixed interpolation runs have an established formatter layout
            // and may need several independent width decisions. Keep this
            // narrow path to the single interpolation reported in #10177.
            return false;
        }
        let hugged_name = joiner.hugged_closing_name(start, end);
        if join.is_continuation()
            && (first_open != 0 || close != bytes.len())
            && (hugged_name.is_some() || joiner.previous_chunk_is_interpolation(start))
        {
            // A mixed text/interpolation chunk that continues an existing
            // interpolation or is hugged directly by its parent has a larger
            // established layout. Rewrapping it in isolation would make a
            // formatted second pass differ from the first. Continuations after
            // an ordinary closing tag can still wrap at the mustache boundary.
            return false;
        }
        let line_start = memchr::memrchr2(b'\r', b'\n', output).map_or(0, |pos| pos + 1);
        let line = output.get(line_start..).unwrap_or_default();
        let suffix_width = hugged_name.map_or(0, |name| name.len() + 3)
            + self.base_depth * self.options.tab_width as usize;
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
        // Break only inside mustache syntax, never between runtime text and an
        // interpolation. This preserves text such as `<{{ value }}>` byte for
        // byte while giving the interpolation on the overflowing line a stable
        // layout. A child touching both parent tags uses the established
        // parent depth; an authored child line keeps its own depth.
        let layout_depth = if hugged_name.is_some() {
            depth.saturating_sub(1)
        } else {
            depth
        };
        output.extend_from_slice(bytes.get(..first_open + 2).unwrap_or_default());
        output.extend_from_slice(self.newline);
        let expression = formatted.get(expr_start..expr_end).unwrap_or_default();
        output.extend_from_slice(
            self.render_interpolation_expr_lines(expression, layout_depth)
                .as_bytes(),
        );
        self.write_indent(output, layout_depth);
        output.extend_from_slice(b"}}");
        output.extend_from_slice(bytes.get(close..).unwrap_or_default());
        output.extend_from_slice(self.newline);
        true
    }
}

#[cold]
#[inline(never)]
fn authored_multiline_child_owns_layout(bytes: &[u8]) -> bool {
    if !bytes.windows(2).any(|window| window == b"{{") {
        return true;
    }
    parse_interpolation_range(bytes, 0).is_some_and(|(expression_start, expression_end, _)| {
        bytes
            .get(expression_start..expression_end)
            .is_some_and(|expression| expression.iter().any(|byte| matches!(byte, b'\r' | b'\n')))
    })
}
