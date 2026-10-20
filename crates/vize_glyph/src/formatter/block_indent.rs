//! Indenting `<script>` / `<style>` block bodies for `vueIndentScriptAndStyle`.
//!
//! Indentation is a presentation choice everywhere except inside a template
//! literal, where every byte between the backticks is part of the string's
//! runtime value. Prefixing those continuation lines rewrote the emitted
//! string — a `ChartStyle.vue` that builds CSS gained two spaces on every
//! line, and the glyph corpus parse-preservation property caught it as a
//! changed `TemplateElement` (#3334). #3269 established the same guarantee for
//! template-position interpolations; this is the script-block half.

use crate::template::helpers::template_literal_state_after_line_from;

/// Write `content` into `output`, indenting each line with `indent` except
/// lines that continue a multi-line template literal, which are emitted
/// verbatim.
///
/// The opening line of a template literal is still indented: its leading
/// whitespace sits before the backtick and is not part of the string.
pub(super) fn write_indented_block(
    output: &mut Vec<u8>,
    content: &str,
    indent: &[u8],
    newline: &[u8],
) {
    let trimmed = content.trim_end_matches('\n').trim_end_matches('\r');
    let mut inside_template_literal = false;

    // Every terminator, LF, CRLF or a lone CR, becomes the configured newline:
    // a body kept as written may still use another ending than the layout.
    let lines = trimmed
        .split('\n')
        .flat_map(|line| line.strip_suffix('\r').unwrap_or(line).split('\r'));
    for line in lines {
        if !line.is_empty() && !inside_template_literal {
            output.extend_from_slice(indent);
        }
        output.extend_from_slice(line.as_bytes());
        output.extend_from_slice(newline);
        inside_template_literal =
            template_literal_state_after_line_from(inside_template_literal, line);
    }
}

/// Trim a block body kept as written and remove the indentation its lines
/// share, so a body indented by an earlier pass is not indented again.
pub(super) fn kept_block_body(content: &str) -> vize_l0::String {
    let is_terminator = |c: char| c == '\n' || c == '\r';
    // Start at the first non-blank line, keeping its indentation.
    let start = content
        .find(|c: char| !c.is_whitespace())
        .and_then(|first| content.get(..first))
        .and_then(|blank| blank.rfind(is_terminator))
        .map_or(0, |terminator| terminator + 1);
    let body = content.get(start..).unwrap_or_default().trim_end();
    let mut shared: Option<&str> = None;
    let mut inside_template_literal = false;
    for line in body
        .split('\n')
        .flat_map(|line| line.strip_suffix('\r').unwrap_or(line).split('\r'))
    {
        let text = line.trim_start_matches([' ', '\t']);
        if !inside_template_literal && !text.is_empty() {
            let indent = line.strip_suffix(text).unwrap_or_default();
            shared = Some(shared.map_or(indent, |shared| {
                let common = shared
                    .bytes()
                    .zip(indent.bytes())
                    .take_while(|(a, b)| a == b)
                    .count();
                shared.get(..common).unwrap_or_default()
            }));
        }
        inside_template_literal =
            template_literal_state_after_line_from(inside_template_literal, line);
    }
    let shared = shared.unwrap_or_default();
    if shared.is_empty() {
        return body.into();
    }
    let mut output = vize_l0::String::with_capacity(body.len());
    let mut inside_template_literal = false;
    for (index, line) in body
        .split('\n')
        .flat_map(|line| line.strip_suffix('\r').unwrap_or(line).split('\r'))
        .enumerate()
    {
        if index > 0 {
            output.push('\n');
        }
        if inside_template_literal {
            output.push_str(line);
        } else {
            output.push_str(line.strip_prefix(shared).unwrap_or(line));
        }
        inside_template_literal =
            template_literal_state_after_line_from(inside_template_literal, line);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{kept_block_body, write_indented_block};

    fn indent(content: &str) -> vize_l0::String {
        indent_with(content, b"\n")
    }

    fn indent_with(content: &str, newline: &[u8]) -> vize_l0::String {
        let mut output = Vec::new();
        write_indented_block(&mut output, content, b"  ", newline);
        core::str::from_utf8(&output).unwrap().into()
    }

    #[test]
    fn ordinary_lines_are_indented() {
        assert_eq!(
            indent("const a = 1\nconst b = 2\n"),
            "  const a = 1\n  const b = 2\n"
        );
    }

    #[test]
    fn template_literal_continuation_lines_stay_verbatim() {
        // The line opening the literal is indented (that whitespace precedes
        // the backtick); every line inside it must be emitted untouched, or
        // the string's runtime value changes.
        let source = "const css = `\nbody {\n  color: red;\n}\n`\n";
        assert_eq!(
            indent(source),
            "  const css = `\nbody {\n  color: red;\n}\n`\n"
        );
    }

    #[test]
    fn indentation_resumes_after_the_literal_closes() {
        let source = "const a = `\nx\n`\nconst b = 2\n";
        assert_eq!(indent(source), "  const a = `\nx\n`\n  const b = 2\n");
    }

    #[test]
    fn escaped_backticks_do_not_toggle_the_literal() {
        let source = "const a = '\\`'\nconst b = 2\n";
        assert_eq!(indent(source), "  const a = '\\`'\n  const b = 2\n");
    }

    #[test]
    fn nested_literals_close_correctly() {
        // A literal opened and closed on one line leaves the state clean.
        let source = "const a = `x`\nconst b = 2\n";
        assert_eq!(indent(source), "  const a = `x`\n  const b = 2\n");
    }

    #[test]
    fn blank_lines_are_never_indented() {
        assert_eq!(indent("a\n\nb\n"), "  a\n\n  b\n");
    }

    #[test]
    fn crlf_content_emits_the_configured_newline_once() {
        // The '\r' left behind by splitting on '\n' must not survive, or a
        // CRLF newline would double it into "\r\r\n".
        assert_eq!(
            indent_with("const a = 1\r\nconst b = 2\r\n", b"\r\n"),
            "  const a = 1\r\n  const b = 2\r\n"
        );
        assert_eq!(
            indent_with("const a = 1\r\nconst b = 2\r\n", b"\n"),
            "  const a = 1\n  const b = 2\n"
        );
    }

    #[test]
    fn kept_template_literal_bytes_reach_a_fixed_point() {
        let source = "const css = `\nbody {\n  color: red;\n}\n`\nconst x = ;\n";
        let first = indent(&kept_block_body(source));
        let second = indent(&kept_block_body(&first));
        assert_eq!(second, first);
    }
}
