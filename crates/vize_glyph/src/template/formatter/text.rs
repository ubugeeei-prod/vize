//! Text and interpolation chunk emission.
//!
//! Keeps authored chunk adjacency separate from the tag parser so formatter
//! layout whitespace cannot become a Vue runtime text node.

use super::{
    TemplateFormatter, format_interpolation_expression, format_interpolations_with_vue_version,
    parse_interpolation_range,
    suppression::{ChunkJoin, LineJoiner, TextRun},
};
use vize_l0::String;

impl TemplateFormatter<'_> {
    /// Flush accumulated text content with interpolation formatting.
    #[inline]
    pub(super) fn flush_text_buffer(
        &self,
        output: &mut Vec<u8>,
        text: &mut TextRun,
        depth: usize,
        joiner: &mut LineJoiner<'_>,
    ) {
        if text.is_empty() {
            return;
        }
        let formatted =
            format_interpolations_with_vue_version(text.as_str(), self.options, self.vue_version);
        let start = text.start();
        let end = text.end();
        let multiline = formatted.contains('\n');
        let depth = if multiline
            && parse_interpolation_range(text.as_str().as_bytes(), 0)
                .is_some_and(|(_, _, end)| end == text.as_str().len())
        {
            joiner.interpolation_depth(start, end, depth)
        } else {
            depth
        };
        text.clear();
        // If the formatted expression wraps onto multiple lines, single-line
        // `{{ expr }}` emission would leave the wrapped lines indented
        // relative to column 0 instead of the interpolation's depth. Emit the
        // canonical multi-line shape on the first pass. (#957)
        if multiline
            && let Some(rewrapped) =
                self.rewrap_text_with_multiline_interpolation(&formatted, depth)
        {
            let join = joiner.open(start);
            if join.is_continuation() {
                self.open_chunk(output, depth, join);
                let indent_len = self.indent.len() * depth;
                let dedented = rewrapped.as_bytes().get(indent_len..).unwrap_or_default();
                output.extend_from_slice(dedented);
            } else {
                if matches!(join, ChunkJoin::BlankLine) && !output.is_empty() {
                    output.extend_from_slice(self.newline);
                }
                output.extend_from_slice(rewrapped.as_bytes());
            }
            joiner.finish(end);
            return;
        }
        self.open_chunk(output, depth, joiner.open(start));
        if self.write_wrapped_hugged_child(output, &formatted, start, end, depth, joiner) {
            joiner.finish(end);
            return;
        }
        output.extend_from_slice(formatted.as_bytes());
        output.extend_from_slice(self.newline);
        joiner.finish(end);
    }

    /// Rewrap only the mustache syntax while keeping surrounding runtime text
    /// immediately adjacent. (#957)
    fn rewrap_text_with_multiline_interpolation(&self, text: &str, depth: usize) -> Option<String> {
        let bytes = text.as_bytes();
        let mut has_multiline_interp = false;
        let mut i = 0;
        while i + 1 < bytes.len() {
            if bytes.get(i..i + 2) == Some(b"{{".as_slice()) {
                let (expr_start, expr_end, end) = parse_interpolation_range(bytes, i)?;
                has_multiline_interp = bytes.get(expr_start..expr_end)?.contains(&b'\n');
                i = end;
                if has_multiline_interp {
                    break;
                }
                continue;
            }
            i += 1;
        }
        if !has_multiline_interp {
            return None;
        }

        let mut out = String::default();
        self.write_indent_string(&mut out, depth);
        let mut cursor = 0;
        loop {
            let mut next = cursor;
            while next + 1 < bytes.len() && !(bytes.get(next..next + 2) == Some(b"{{".as_slice())) {
                next += 1;
            }
            if next + 1 >= bytes.len() || !(bytes.get(next..next + 2) == Some(b"{{".as_slice())) {
                out.push_str(text.get(cursor..).unwrap_or_default());
                out.push_str(self.newline_str());
                break;
            }
            // Text belongs to the rendered DOM. Keep its bytes immediately
            // adjacent to the mustache; only whitespace *inside* `{{ }}` is
            // syntax and may be expanded for layout.
            out.push_str(text.get(cursor..next).unwrap_or_default());

            let (expr_start, expr_end, end) = parse_interpolation_range(bytes, next)?;
            let expr = text.get(expr_start..expr_end)?;
            if !expr.contains('\n') {
                out.push_str(text.get(next..end).unwrap_or_default());
                cursor = end;
                continue;
            }
            out.push_str("{{");
            out.push_str(self.newline_str());
            out.push_str(self.render_interpolation_expr_lines(expr, depth).as_str());
            self.write_indent_string(&mut out, depth);
            out.push_str("}}");
            cursor = end;
        }
        Some(out)
    }

    pub(super) fn write_indent_string(&self, out: &mut String, depth: usize) {
        let indent = std::str::from_utf8(self.indent).unwrap_or("  ");
        for _ in 0..depth {
            out.push_str(indent);
        }
    }

    pub(super) fn newline_str(&self) -> &str {
        std::str::from_utf8(self.newline).unwrap_or("\n")
    }

    pub(super) fn write_multiline_interpolation(
        &self,
        output: &mut Vec<u8>,
        expr: &str,
        depth: usize,
    ) {
        output.extend_from_slice(b"{{");
        output.extend_from_slice(self.newline);
        let formatted_expr = format_interpolation_expression(expr, self.options, self.vue_version);
        output.extend_from_slice(
            self.render_interpolation_expr_lines(&formatted_expr, depth)
                .as_bytes(),
        );
        self.write_indented_line(output, b"}}", depth);
    }
}
