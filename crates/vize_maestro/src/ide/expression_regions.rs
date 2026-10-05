//! Original expression regions shared by highlighting and comment navigation.
//! This extends the existing structural token scan, without parsing an AST.

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegionKind {
    Code,
    String,
    Comment,
}

/// Visit disjoint byte ranges, retaining template substitutions as code.
pub(crate) fn visit_regions(expr: &str, visit: impl FnMut(Range<usize>, RegionKind)) {
    scan_regions(expr, false, visit);
}

pub(crate) fn visit_html_regions(expr: &str, visit: impl FnMut(Range<usize>, RegionKind)) {
    scan_regions(expr, true, visit);
}

fn scan_regions(expr: &str, html: bool, mut visit: impl FnMut(Range<usize>, RegionKind)) {
    let bytes = expr.as_bytes();
    let mut cursor = 0;
    let mut start = 0;
    let mut template = false;
    let mut substitutions = Vec::<usize>::new();
    let mut operand = true;
    while let Some(&byte) = bytes.get(cursor) {
        if template {
            if byte == b'\\' {
                cursor = escaped_end(expr, cursor);
            } else if byte == b'`' {
                cursor += 1;
                visit(start..cursor, RegionKind::String);
                start = cursor;
                template = false;
                operand = false;
            } else if bytes.get(cursor..cursor + 2) == Some(b"${".as_slice()) {
                visit(start..cursor, RegionKind::String);
                cursor += 2;
                start = cursor;
                substitutions.push(0);
                template = false;
                operand = true;
            } else {
                cursor += 1;
            }
            continue;
        }
        let tail = expr.get(cursor..).unwrap_or_default();
        let comment_end = if tail.starts_with("//") {
            Some(
                cursor
                    + tail
                        .find(['\r', '\n', '\u{2028}', '\u{2029}'])
                        .unwrap_or(tail.len()),
            )
        } else if tail.starts_with("/*") {
            Some(
                cursor
                    + tail
                        .get(2..)
                        .and_then(|rest| rest.find("*/"))
                        .map_or(tail.len(), |end| end + 4),
            )
        } else {
            None
        };
        if let Some(end) = comment_end {
            visit(start..cursor, RegionKind::Code);
            visit(cursor..end, RegionKind::Comment);
            cursor = end;
            start = cursor;
            continue;
        }
        if html && let Some((quote, width)) = html_quote(expr, cursor) {
            visit(start..cursor, RegionKind::Code);
            let end = html_quoted_end(expr, cursor + width, quote);
            // Retain the old authored entity token wire; shielding only keeps
            // encoded string data out of new comment/navigation decisions.
            visit(cursor..end, RegionKind::Code);
            cursor = end;
            start = cursor;
            operand = false;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            visit(start..cursor, RegionKind::Code);
            let end = if html {
                html_quoted_end(expr, cursor + 1, byte)
            } else {
                quoted_end(expr, cursor, byte)
            };
            visit(cursor..end, RegionKind::String);
            cursor = end;
            start = cursor;
            operand = false;
            continue;
        }
        if byte == b'`' {
            visit(start..cursor, RegionKind::Code);
            start = cursor;
            cursor += 1;
            template = true;
            continue;
        }
        if byte == b'}' {
            if let Some(depth) = substitutions.last_mut() {
                if *depth == 0 {
                    visit(start..cursor, RegionKind::Code);
                    substitutions.pop();
                    start = cursor;
                    cursor += 1;
                    template = true;
                    continue;
                }
                *depth -= 1;
            }
        } else if byte == b'{'
            && let Some(depth) = substitutions.last_mut()
        {
            *depth += 1;
        }
        // Preserve prior regex token classification, but do not mistake its
        // escaped slashes or character-class data for comments/quotes.
        if byte == b'/'
            && operand
            && let Some(end) = regexp_end(expr, cursor)
        {
            cursor = end;
            operand = false;
            continue;
        }
        if matches!(bytes.get(cursor..cursor + 2), Some(b"++" | b"--")) {
            cursor += 2;
            continue;
        }
        let Some(ch) = tail.chars().next() else { break };
        if oxc_syntax::identifier::is_identifier_start(ch) {
            let word_start = cursor;
            cursor += ch.len_utf8();
            while let Some(ch) = expr.get(cursor..).and_then(|rest| rest.chars().next()) {
                if !oxc_syntax::identifier::is_identifier_part(ch) {
                    break;
                }
                cursor += ch.len_utf8();
            }
            operand = matches!(
                expr.get(word_start..cursor),
                Some(
                    "return"
                        | "throw"
                        | "case"
                        | "delete"
                        | "void"
                        | "typeof"
                        | "new"
                        | "yield"
                        | "await"
                        | "in"
                        | "of"
                        | "instanceof"
                )
            );
            continue;
        }
        if !ch.is_whitespace() {
            operand = matches!(
                byte,
                b'(' | b'['
                    | b'{'
                    | b','
                    | b';'
                    | b':'
                    | b'?'
                    | b'='
                    | b'!'
                    | b'~'
                    | b'+'
                    | b'-'
                    | b'*'
                    | b'%'
                    | b'/'
                    | b'&'
                    | b'|'
                    | b'^'
                    | b'<'
                    | b'>'
            );
        }
        cursor += ch.len_utf8();
    }
    visit(
        start..cursor,
        if template {
            RegionKind::String
        } else {
            RegionKind::Code
        },
    );
}

fn escaped_end(expr: &str, cursor: usize) -> usize {
    cursor
        + 1
        + expr
            .get(cursor + 1..)
            .and_then(|rest| rest.chars().next())
            .map_or(0, char::len_utf8)
}

fn quoted_end(expr: &str, start: usize, quote: u8) -> usize {
    let mut cursor = start + 1;
    while let Some(&byte) = expr.as_bytes().get(cursor) {
        if byte == b'\\' {
            cursor = escaped_end(expr, cursor);
        } else {
            cursor += 1;
            if byte == quote {
                break;
            }
        }
    }
    cursor
}

fn regexp_end(expr: &str, start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    let mut class = false;
    while let Some(&byte) = expr.as_bytes().get(cursor) {
        match byte {
            b'\r' | b'\n' => return None,
            b'\\' => {
                cursor = escaped_end(expr, cursor);
                continue;
            }
            b'[' => class = true,
            b']' => class = false,
            b'/' if !class => return Some(cursor + 1),
            _ => {}
        }
        cursor += 1;
    }
    None
}

#[cfg(test)]
fn comment_at(expr: &str, offset: usize) -> bool {
    let mut found = false;
    visit_regions(expr, |range, kind| {
        found |= kind == RegionKind::Comment && range.contains(&offset);
    });
    found
}

fn html_quote(expr: &str, cursor: usize) -> Option<(u8, usize)> {
    let bytes = expr.as_bytes().get(cursor..)?;
    if bytes.first() != Some(&b'&') {
        return None;
    }
    let (ch, width) = vize_armature::tokenizer::entity_decode::try_decode_entity(
        bytes,
        htmlize::Context::Attribute,
    )?;
    matches!(ch, '\'' | '"').then_some((ch as u8, width))
}

fn html_quoted_end(expr: &str, mut cursor: usize, quote: u8) -> usize {
    while let Some(&byte) = expr.as_bytes().get(cursor) {
        if byte == b'\\' {
            cursor = escaped_end(expr, cursor);
            continue;
        }
        if let Some((decoded, width)) = html_quote(expr, cursor) {
            cursor += width;
            if decoded == quote {
                break;
            }
        } else {
            cursor += 1;
            if byte == quote {
                break;
            }
        }
    }
    cursor
}

pub(crate) fn html_comment_at(expr: &str, offset: usize) -> bool {
    let mut found = false;
    visit_html_regions(expr, |range, kind| {
        found |= kind == RegionKind::Comment && range.contains(&offset);
    });
    found
}

#[cfg(test)]
mod tests;
