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
    if options.use_tabs || options.tab_width != 2 {
        reindent_token_gaps(&output, options, false)
    } else {
        output
    }
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
    if options.use_tabs || options.tab_width != 2 {
        return reindent_token_gaps(source, options, true);
    }
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

// Unparsed declaration values retain authored whitespace in the CSS printer.
// Only whitespace gaps may be rebased; quoted/escaped token bytes stay intact.
fn reindent_token_gaps(source: &str, options: &FormatOptions, printer_indent: bool) -> String {
    let mut gaps = Vec::new();
    let mut pending = Vec::new();
    let mut declaration = None;
    for token in super::blank_lines::Tokens::new(source) {
        if token.gap.contains(['\r', '\n']) {
            let indentation = token.gap.rsplit(['\r', '\n']).next().unwrap_or_default();
            let level = if printer_indent {
                indentation.len() / 2
            } else {
                0
            };
            gaps.push((token.gap_start, token.start, level));
            if declaration.is_some() && !matches!(token.text, ";" | "}") {
                pending.push(gaps.len() - 1);
            }
        }
        if token.parens != 0 || token.brackets != 0 {
            continue;
        }
        match token.text {
            ":" if token.depth > 0 && declaration.is_none() => {
                declaration = Some(token.depth);
            }
            ";" | "}" => {
                if let Some(depth) = declaration.take() {
                    for index in pending.drain(..) {
                        if let Some((_, _, level)) = gaps.get_mut(index) {
                            *level = depth + 1;
                        }
                    }
                }
                pending.clear();
            }
            "{" => {
                declaration = None;
                pending.clear();
            }
            _ => {}
        }
    }
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, level) in gaps {
        output.push_str(source.get(cursor..start).unwrap_or_default());
        let gap = source.get(start..end).unwrap_or_default();
        let mut bytes = gap.bytes().peekable();
        while let Some(byte) = bytes.next() {
            if byte == b'\r' {
                if bytes.peek() == Some(&b'\n') {
                    bytes.next();
                }
                output.push_str(options.newline_string());
            } else if byte == b'\n' {
                output.push_str(options.newline_string());
            }
        }
        if printer_indent || level > 0 {
            write_css_indent(&mut output, level, &options.indent_string());
        } else {
            output.push_str(gap.rsplit(['\r', '\n']).next().unwrap_or_default());
        }
        cursor = end;
    }
    output.push_str(source.get(cursor..).unwrap_or_default());
    if printer_indent {
        output.truncate(output.trim_end_matches(['\r', '\n']).len());
    }
    output
}
