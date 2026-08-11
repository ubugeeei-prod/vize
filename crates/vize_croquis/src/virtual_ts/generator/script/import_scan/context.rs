//! Source context tracking for line-oriented import extraction.
//!
//! Block comments and template literals span lines, so the import scanner has
//! to know which lines actually begin in code before treating one as a
//! statement.

/// Source context of a template literal being scanned.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TemplateContext {
    /// Inside the literal text of a template.
    Literal,
    /// Inside a `${ … }` expression, tracking its unbalanced `{` count.
    Expression(u32),
}

/// Report, for every line, whether it starts in code context.
///
/// Import extraction is line oriented, so it would otherwise hoist an
/// `import`-looking line out of a template literal or a block comment and
/// corrupt both the module scope and the setup body. Strings and `//` comments
/// cannot span lines, so only block comments and template literals carry over.
pub(crate) fn code_line_starts(lines: &[&str]) -> Vec<bool> {
    let mut starts = Vec::with_capacity(lines.len());
    let mut block_comment = false;
    let mut templates: Vec<TemplateContext> = Vec::new();

    for line in lines {
        starts.push(!block_comment && templates.is_empty());

        let bytes = line.as_bytes();
        let mut quote: Option<u8> = None;
        let mut index = 0;

        while index < bytes.len() {
            let byte = bytes[index];

            if block_comment {
                if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    block_comment = false;
                    index += 2;
                    continue;
                }
                index += 1;
                continue;
            }

            if let Some(open) = quote {
                if byte == b'\\' {
                    index += 2;
                    continue;
                }
                if byte == open {
                    quote = None;
                }
                index += 1;
                continue;
            }

            if matches!(templates.last(), Some(TemplateContext::Literal)) {
                match byte {
                    b'\\' => {
                        index += 2;
                        continue;
                    }
                    b'`' => {
                        templates.pop();
                    }
                    b'$' if bytes.get(index + 1) == Some(&b'{') => {
                        templates.push(TemplateContext::Expression(0));
                        index += 2;
                        continue;
                    }
                    _ => {}
                }
                index += 1;
                continue;
            }

            match byte {
                // The rest of the line is a comment.
                b'/' if bytes.get(index + 1) == Some(&b'/') => break,
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    block_comment = true;
                    index += 2;
                    continue;
                }
                b'"' | b'\'' => quote = Some(byte),
                b'`' => templates.push(TemplateContext::Literal),
                b'{' => {
                    if let Some(TemplateContext::Expression(depth)) = templates.last_mut() {
                        *depth += 1;
                    }
                }
                b'}' => {
                    if let Some(TemplateContext::Expression(depth)) = templates.last_mut() {
                        if *depth == 0 {
                            templates.pop();
                        } else {
                            *depth -= 1;
                        }
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }

    starts
}
