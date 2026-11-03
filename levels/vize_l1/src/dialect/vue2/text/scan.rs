use super::TextBoundaryKind;
use alloc::vec::Vec;

/// The pinned upstream state machine uses immediate previous escaping, opaque
/// templates and ASCII division look-behind (only SPACE is skipped). This is
/// intentionally historical syntax, not a second JavaScript lexer.
pub(super) fn scan(text: &str, separator: u8) -> Result<Vec<usize>, TextBoundaryKind> {
    let bytes = text.as_bytes();
    let mut quote = None;
    let mut regex = false;
    let (mut curly, mut square, mut paren) = (0i64, 0i64, 0i64);
    let mut separators = Vec::new();
    for (i, &byte) in bytes.iter().enumerate() {
        let prev = i.checked_sub(1).and_then(|index| bytes.get(index)).copied();
        if let Some(delimiter) = quote {
            if byte == delimiter && prev != Some(b'\\') {
                quote = None;
            }
        } else if regex {
            if byte == b'[' {
                return Err(TextBoundaryKind::RegexCharacterClass);
            }
            if byte == b'/' && prev != Some(b'\\') {
                regex = false;
            }
        } else if byte == separator
            && curly == 0
            && square == 0
            && paren == 0
            && (separator != b'|' || (prev != Some(b'|') && bytes.get(i + 1) != Some(&b'|')))
        {
            separators.push(i);
        } else {
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b'{' => curly += 1,
                b'}' => curly -= 1,
                b'[' => square += 1,
                b']' => square -= 1,
                b'(' => paren += 1,
                b')' => paren -= 1,
                _ => {}
            }
            if curly < 0 || square < 0 || paren < 0 {
                return Err(TextBoundaryKind::UnbalancedFilterSyntax);
            }
            if byte == b'/' {
                if matches!(bytes.get(i + 1), Some(b'/' | b'*')) {
                    return Err(TextBoundaryKind::CommentSyntax);
                }
                let previous = bytes
                    .get(..i)
                    .and_then(|prefix| prefix.iter().rev().copied().find(|byte| *byte != b' '));
                regex = previous.is_none_or(|byte| {
                    !byte.is_ascii_alphanumeric()
                        && !matches!(byte, b'_' | b')' | b'.' | b'+' | b'-' | b'$' | b']')
                });
            }
        }
    }
    if quote.is_some() || regex || curly != 0 || square != 0 || paren != 0 {
        return Err(TextBoundaryKind::UnbalancedFilterSyntax);
    }
    Ok(separators)
}
