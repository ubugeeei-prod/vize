use crate::options::FormatOptions;
use vize_l0::String;

/// Indent rules and declarations without passing authored tokens through the
/// CSS printer. This path keeps media queries, selectors, and values intact.
pub(super) fn format_layout_only(source: &str, options: &FormatOptions) -> String {
    let newline = options.newline_string();
    let indent = options.indent_string();
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len() + source.len() / 4);
    let mut depth = 0usize;
    let mut parens = 0usize;
    let mut brackets = 0usize;
    let mut quote = None;
    let mut start = 0usize;
    let mut index = 0usize;

    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' && index + 1 < bytes.len() {
            index += 2;
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            while index + 1 < bytes.len() && bytes.get(index..index + 2) != Some(b"*/".as_slice()) {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        match byte {
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'{' if parens == 0 && brackets == 0 => {
                write_css_line(
                    &mut output,
                    source.get(start..index).unwrap_or_default().trim(),
                    depth,
                    &indent,
                    newline,
                );
                // Replace the preceding newline with the opening brace.
                if output.ends_with(newline) {
                    output.truncate(output.len() - newline.len());
                }
                output.push_str(" {");
                output.push_str(newline);
                depth += 1;
                start = index + 1;
            }
            b';' if parens == 0 && brackets == 0 => {
                let statement = source.get(start..index).unwrap_or_default().trim();
                if !statement.is_empty() {
                    write_css_indent(&mut output, depth, &indent);
                    output.push_str(statement);
                    output.push(';');
                    output.push_str(newline);
                }
                start = index + 1;
            }
            b'}' if parens == 0 && brackets == 0 && depth > 0 => {
                write_css_line(
                    &mut output,
                    source.get(start..index).unwrap_or_default().trim(),
                    depth,
                    &indent,
                    newline,
                );
                depth -= 1;
                write_css_indent(&mut output, depth, &indent);
                output.push('}');
                output.push_str(newline);
                if depth == 0
                    && !source
                        .get(index + 1..)
                        .unwrap_or_default()
                        .trim()
                        .is_empty()
                {
                    output.push_str(newline);
                }
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    write_css_line(
        &mut output,
        source.get(start..).unwrap_or_default().trim(),
        depth,
        &indent,
        newline,
    );
    output
}

fn write_css_line(output: &mut String, content: &str, depth: usize, indent: &str, newline: &str) {
    if !content.is_empty() {
        write_css_indent(output, depth, indent);
        output.push_str(content);
        output.push_str(newline);
    }
}

fn write_css_indent(output: &mut String, depth: usize, indent: &str) {
    for _ in 0..depth {
        output.push_str(indent);
    }
}

pub(super) fn source_separator_lines(source: &str) -> usize {
    if source
        .as_bytes()
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
        >= 2
    {
        2
    } else {
        1
    }
}

pub(super) fn source_separator_lines_before(source: &str) -> usize {
    let prefix_len = source.len() - source.trim_start().len();
    source_separator_lines(source.get(..prefix_len).unwrap_or_default())
}

pub(super) fn source_separator_lines_after(source: &str) -> usize {
    source_separator_lines(source.get(source.trim_end().len()..).unwrap_or_default())
}

pub(super) fn changes_authored_css(source: &str, printed: &str) -> bool {
    // Ignore layout whitespace and an optional final declaration semicolon.
    // All other token changes, including inserted nesting ampersands, changed
    // media features, reordered values and shortened pseudo-elements, matter.
    fn tokens(source: &str) -> Vec<u8> {
        let bytes = source.as_bytes();
        let mut output = Vec::with_capacity(bytes.len());
        let mut index = 0;
        let mut quote = None;
        while let Some(&byte) = bytes.get(index) {
            if byte == b'\\' && index + 1 < bytes.len() {
                output.extend_from_slice(bytes.get(index..index + 2).unwrap_or_default());
                index += 2;
                continue;
            }
            if let Some(delimiter) = quote {
                output.push(byte);
                if byte == delimiter {
                    quote = None;
                }
                index += 1;
                continue;
            }
            if matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
                output.push(byte);
                index += 1;
                continue;
            }
            if byte.is_ascii_whitespace() {
                index += 1;
                continue;
            }
            if byte == b';'
                && bytes
                    .get(index + 1..)
                    .unwrap_or_default()
                    .iter()
                    .find(|b| !b.is_ascii_whitespace())
                    == Some(&b'}')
            {
                index += 1;
                continue;
            }
            output.push(byte);
            index += 1;
        }
        output
    }
    tokens(source) != tokens(printed)
}

/// Re-indent CSS output to match the configured indent style
pub(super) fn reindent_css(source: &str, options: &FormatOptions) -> String {
    let indent = options.indent_string();
    let newline = options.newline_string();
    let mut result: String = String::with_capacity(source.len());

    for line in source.lines() {
        // Count leading spaces (lightningcss uses 2-space indent)
        let leading_spaces = line.len() - line.trim_start().len();
        let indent_level = leading_spaces / 2;
        let trimmed = line.trim_start();

        if trimmed.is_empty() {
            result.push_str(newline);
            continue;
        }

        for _ in 0..indent_level {
            result.push_str(&indent);
        }
        result.push_str(trimmed);
        result.push_str(newline);
    }

    // Remove trailing newline added by the loop
    if result.ends_with(newline) {
        result.truncate(result.len() - newline.len());
    }

    result
}
