//! Text and interpolation chunk emission.
//!
//! Keeps authored chunk adjacency separate from the tag parser so formatter
//! layout whitespace cannot become a Vue runtime text node.

use super::{
    TemplateFormatter, format_interpolation_expression, format_interpolations,
    suppression::{LineJoiner, TextRun},
};
use vize_s0::String;

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
        let formatted = format_interpolations(text.as_str(), self.options);
        let start = text.start();
        let end = text.end();
        text.clear();
        // If the formatted expression wraps onto multiple lines, single-line
        // `{{ expr }}` emission would leave the wrapped lines indented
        // relative to column 0 instead of the interpolation's depth. Emit the
        // canonical multi-line shape on the first pass. (#957)
        if formatted.contains('\n')
            && let Some(rewrapped) =
                self.rewrap_text_with_multiline_interpolation(&formatted, depth)
        {
            let join = joiner.open(start);
            if join.is_some() {
                self.open_chunk(output, depth, join);
                let indent_len = self.indent.len() * depth;
                output.extend_from_slice(&rewrapped.as_bytes()[indent_len.min(rewrapped.len())..]);
            } else {
                output.extend_from_slice(rewrapped.as_bytes());
            }
            joiner.finish(end);
            return;
        }
        self.open_chunk(output, depth, joiner.open(start));
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
            if bytes[i] == b'{' && bytes[i + 1] == b'{' {
                let mut j = i + 2;
                let mut depth_in = 1;
                let mut saw_newline = false;
                while j + 1 < bytes.len() {
                    if bytes[j] == b'\n' {
                        saw_newline = true;
                    }
                    if bytes[j] == b'{' && bytes[j + 1] == b'{' {
                        depth_in += 1;
                        j += 2;
                    } else if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                        depth_in -= 1;
                        if depth_in == 0 {
                            has_multiline_interp = saw_newline;
                            j += 2;
                            break;
                        }
                        j += 2;
                    } else {
                        j += 1;
                    }
                }
                i = j;
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
            while next + 1 < bytes.len() && !(bytes[next] == b'{' && bytes[next + 1] == b'{') {
                next += 1;
            }
            if next + 1 >= bytes.len() || !(bytes[next] == b'{' && bytes[next + 1] == b'{') {
                out.push_str(&text[cursor..]);
                out.push_str(self.newline_str());
                break;
            }
            // Text belongs to the rendered DOM. Keep its bytes immediately
            // adjacent to the mustache; only whitespace *inside* `{{ }}` is
            // syntax and may be expanded for layout.
            out.push_str(&text[cursor..next]);

            let mut k = next + 2;
            let mut interpolation_depth = 1;
            while k + 1 < bytes.len() {
                if bytes[k] == b'{' && bytes[k + 1] == b'{' {
                    interpolation_depth += 1;
                    k += 2;
                } else if bytes[k] == b'}' && bytes[k + 1] == b'}' {
                    interpolation_depth -= 1;
                    if interpolation_depth == 0 {
                        break;
                    }
                    k += 2;
                } else {
                    k += 1;
                }
            }
            if interpolation_depth != 0 {
                return None;
            }
            let expr = &text[next + 2..k];
            if !expr.contains('\n') {
                out.push_str(&text[next..k + 2]);
                cursor = k + 2;
                continue;
            }
            out.push_str("{{");
            out.push_str(self.newline_str());
            out.push_str(self.render_interpolation_expr_lines(expr, depth).as_str());
            self.write_indent_string(&mut out, depth);
            out.push_str("}}");
            cursor = k + 2;
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
        let formatted_expr = format_interpolation_expression(expr, self.options);
        output.extend_from_slice(
            self.render_interpolation_expr_lines(&formatted_expr, depth)
                .as_bytes(),
        );
        self.write_indented_line(output, b"}}", depth);
    }
}
