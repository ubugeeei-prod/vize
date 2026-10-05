//! HTML pre/textarea text normalization without changing authored spans.
//! L1 text is still entity encoded here; remove the spelling of only the first
//! decoded newline, so later entity decoding remains exactly once.

use vize_l0::String;

use crate::lower::cx::Cx;

pub(super) fn normalize_special_text(cx: &Cx<'_>, text: &str, start: u32) -> Option<String> {
    let leading = cx.ignore_newline_at == Some(start);
    let normalize = cx.normalize_pre_newlines();
    if !leading && !normalize {
        return None;
    }
    let skip = if leading {
        leading_newline_len(text)
    } else {
        0
    };
    let remaining = text.get(skip..).unwrap_or_default();
    if normalize && let Some(normalized) = normalize_encoded_crlf(remaining) {
        return Some(normalized);
    }
    (skip > 0).then(|| String::from(remaining))
}

/// Keep entity spellings until the normal decoding boundary, replacing only
/// pairs that decode to CRLF. Escaped references therefore never decode twice.
fn normalize_encoded_crlf(text: &str) -> Option<String> {
    if !text.contains('\r') && !text.contains("&#") {
        return None;
    }
    let mut output = String::default();
    let mut copied = 0;
    let mut cursor = 0;
    while cursor < text.len() {
        let tail = text.get(cursor..)?;
        if let Some(('\r', first)) = leading_line_break(tail)
            && let Some(('\n', second)) = tail.get(first..).and_then(leading_line_break)
        {
            output.push_str(text.get(copied..cursor)?);
            output.push('\n');
            cursor += first + second;
            copied = cursor;
        } else {
            cursor += tail.chars().next()?.len_utf8();
        }
    }
    if copied == 0 {
        return None;
    }
    output.push_str(text.get(copied..)?);
    Some(output)
}

fn leading_newline_len(text: &str) -> usize {
    match leading_line_break(text) {
        Some(('\n', len)) => len,
        Some(('\r', len)) => text
            .get(len..)
            .and_then(leading_line_break)
            .filter(|(character, _)| *character == '\n')
            .map_or(0, |(_, next)| len + next),
        _ => 0,
    }
}

/// Only LF and CR can participate in the rule. Named NewLine requires its
/// semicolon; numeric references allow the same optional terminator as HTML.
fn leading_line_break(text: &str) -> Option<(char, usize)> {
    if text.starts_with('\n') {
        return Some(('\n', 1));
    }
    if text.starts_with('\r') {
        return Some(('\r', 1));
    }
    if text.starts_with("&NewLine;") {
        return Some(('\n', "&NewLine;".len()));
    }
    let numeric = text.strip_prefix("&#")?;
    let (digits, radix, prefix) = if let Some(hex) = numeric
        .strip_prefix('x')
        .or_else(|| numeric.strip_prefix('X'))
    {
        (hex, 16, 3)
    } else {
        (numeric, 10, 2)
    };
    let len = digits
        .bytes()
        .take_while(|byte| {
            if radix == 16 {
                byte.is_ascii_hexdigit()
            } else {
                byte.is_ascii_digit()
            }
        })
        .count();
    let value = u32::from_str_radix(digits.get(..len)?, radix).ok()?;
    let character = match value {
        10 => '\n',
        13 => '\r',
        _ => return None,
    };
    let end = prefix + len;
    Some((
        character,
        end + usize::from(text.as_bytes().get(end) == Some(&b';')),
    ))
}
