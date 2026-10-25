//! Wrap only authored HTML whitespace outside JavaScript interpolation syntax.
use super::{TemplateFormatter, parse_interpolation_range, suppression::LineJoiner};
use unicode_width::UnicodeWidthStr;

impl TemplateFormatter<'_> {
    fn display_width(&self, text: &str) -> usize {
        text.split('\t').map(str::width).sum::<usize>()
            + text.bytes().filter(|byte| *byte == b'\t').count() * self.options.tab_width as usize
    }

    fn output_column(&self, output: &[u8]) -> usize {
        let line = output
            .rsplit(|byte| matches!(byte, b'\r' | b'\n'))
            .next()
            .unwrap_or_default();
        self.display_width(core::str::from_utf8(line).unwrap_or_default())
            + self.base_depth * self.options.tab_width as usize
    }

    pub(super) fn write_wrapped_text_run(
        &self,
        output: &mut Vec<u8>,
        text: &str,
        range: (usize, usize),
        depth: usize,
        joiner: &mut LineJoiner<'_>,
    ) -> bool {
        if joiner.locks_current_line() || text.contains(['\r', '\n']) {
            return false;
        }
        let name = joiner.hugged_closing_name(range.0, range.1);
        let suffix = name.map_or(0, |name| name.len() + 3);
        let width = self.options.print_width as usize;
        // UTF-8 bytes bound display width in the ordinary tab-free path.
        let line_start = memchr::memrchr2(b'\r', b'\n', output).map_or(0, |pos| pos + 1);
        let line = output.get(line_start..).unwrap_or_default();
        if line.len() + self.base_depth * self.options.tab_width as usize + text.len() + suffix
            <= width
            && memchr::memchr(b'\t', line).is_none()
            && !text.contains('\t')
        {
            return false;
        }
        if self.output_column(output) + self.display_width(text) + suffix <= width {
            return false;
        }
        let words = text_words(text);
        if words.len() < 2 {
            return false;
        }
        // When a hugged child really wraps, its opening `>` shares the child
        // indentation. The matching closing `>` stays inside tag syntax too.
        let hugged = name.is_some();
        if hugged {
            let line = output
                .rsplit(|byte| matches!(byte, b'\r' | b'\n'))
                .next()
                .unwrap_or_default();
            if line.last() == Some(&b'>')
                && line
                    .get(..line.len().saturating_sub(1))
                    .is_some_and(|prefix| prefix.iter().all(|byte| matches!(byte, b' ' | b'\t')))
            {
                output.pop();
                output.extend_from_slice(self.indent);
                output.push(b'>');
            }
        }
        let mut previous_end = 0;
        for (index, &(start, end)) in words.iter().enumerate() {
            let word = text.get(start..end).unwrap_or_default();
            let gap = text.get(previous_end..start).unwrap_or_default();
            let closing = if index + 1 == words.len() {
                suffix.saturating_sub(usize::from(hugged))
            } else {
                0
            };
            if index > 0
                && self.output_column(output)
                    + self.display_width(gap)
                    + self.display_width(word)
                    + closing
                    > width
            {
                output.extend_from_slice(self.newline);
                self.write_indent(output, depth);
            } else {
                output.extend_from_slice(gap.as_bytes());
            }
            output.extend_from_slice(word.as_bytes());
            previous_end = end;
        }
        output.extend_from_slice(self.newline);
        if hugged {
            joiner.wrapped_text(range.1);
        }
        true
    }

    /// A text run adjacent to a closing tag can move to a new line by breaking
    /// before `>` without introducing a runtime space between the two nodes.
    #[inline]
    pub(super) fn following_text_overflows<const CANONICAL: bool>(
        &self,
        output: &[u8],
        source: &str,
        start: usize,
    ) -> bool {
        if matches!(
            source.as_bytes().get(start),
            None | Some(b'<' | b'\r' | b'\n')
        ) {
            return false;
        }
        let Some(tail) = source.get(start..) else {
            return false;
        };
        self.following_inline_text_overflows::<CANONICAL>(output, tail)
    }

    #[cold]
    #[inline(never)]
    fn following_inline_text_overflows<const CANONICAL: bool>(
        &self,
        output: &[u8],
        tail: &str,
    ) -> bool {
        let mut end = 0;
        while let Some(&byte) = tail.as_bytes().get(end) {
            if byte == b'<' || matches!(byte, b'\r' | b'\n') {
                break;
            }
            if tail.as_bytes().get(end..end + 2) == Some(b"{{")
                && let Some((_, _, close)) = parse_interpolation_range(tail.as_bytes(), end)
            {
                end = close;
            } else {
                end += 1;
            }
        }
        let tail = tail.get(..end).unwrap_or_default();
        if !CANONICAL && memchr::memmem::find(tail.as_bytes(), b"{{").is_some_and(|open| open > 0) {
            // The default profile already wraps a prefixed sole mustache
            // inside its syntax; keep that established layout.
            return false;
        }
        let column = self.output_column(output) + 1;
        if column + tail.len() <= self.options.print_width as usize && !tail.contains('\t') {
            return false;
        }
        let formatted =
            super::format_interpolations_with_vue_version(tail, self.options, self.vue_version);
        !formatted.contains(['\r', '\n'])
            && column + self.display_width(&formatted) > self.options.print_width as usize
    }
}

fn text_words(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut words = Vec::new();
    let mut start = 0;
    let mut cursor = 0;
    while let Some(&byte) = bytes.get(cursor) {
        if matches!(byte, b' ' | b'\t') {
            if start < cursor {
                words.push((start, cursor));
            }
            cursor += 1;
            start = cursor;
        } else if bytes.get(cursor..cursor + 2) == Some(b"{{")
            && let Some((_, _, end)) = parse_interpolation_range(bytes, cursor)
        {
            cursor = end;
        } else {
            cursor += 1;
        }
    }
    if start < bytes.len() {
        words.push((start, bytes.len()));
    }
    words
}

pub(super) fn compact_text_child(text: &str) -> Option<vize_l0::String> {
    let mut cursor = 0;
    while let Some(offset) = memchr::memmem::find(text.as_bytes().get(cursor..)?, b"{{") {
        let (start, end, close) = parse_interpolation_range(text.as_bytes(), cursor + offset)?;
        if text.get(start..end)?.contains(['\r', '\n']) {
            return None;
        }
        cursor = close;
    }
    let mut compact = vize_l0::String::with_capacity(text.len());
    for line in text.split(['\r', '\n']) {
        let line = line.trim_matches([' ', '\t']);
        if line.is_empty() {
            continue;
        }
        if !compact.is_empty() {
            compact.push(' ');
        }
        compact.push_str(line);
    }
    Some(compact)
}
