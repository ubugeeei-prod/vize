//! Module-specifier scanning for the `vize check` transitive import walk.

use vize_s0::{String, ToCompactString};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ModuleSpecifierOccurrence {
    pub(super) specifier: String,
    pub(super) mode: vize_canon::PackageResolutionMode,
}

/// Collect module specifiers of `source`'s import/export/dynamic-imports.
///
/// This is a deliberately lightweight byte scan rather than a full parse: the
/// transitive walk runs on every checked file, so an AST per file regressed the
/// benchmark. Over-matching (e.g. an import-like fragment inside a string) is
/// harmless because each specifier is resolved against the filesystem and only
/// real source files are registered.
pub(super) fn extract_module_specifier_occurrences(source: &str) -> Vec<ModuleSpecifierOccurrence> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut specifiers = Vec::new();
    let mut i = 0;

    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'\'' | b'"' | b'`' => {
                i = skip_quoted(bytes, i).unwrap_or(len);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = bytes
                    .get(i + 2..)
                    .unwrap_or_default()
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(len, |offset| i + 2 + offset + 1);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = bytes
                    .get(i + 2..)
                    .unwrap_or_default()
                    .windows(2)
                    .position(|window| window == b"*/")
                    .map_or(len, |offset| i + 2 + offset + 2);
                continue;
            }
            _ => {}
        }
        let (keyword_len, keyword_mode) = if matches_keyword(bytes, i, b"from") {
            (4, vize_canon::PackageResolutionMode::Contextual)
        } else if matches_keyword(bytes, i, b"import") {
            (6, vize_canon::PackageResolutionMode::Contextual)
        } else if matches_keyword(bytes, i, b"require") {
            (7, vize_canon::PackageResolutionMode::Require)
        } else {
            i += 1;
            continue;
        };

        let mut j = skip_trivia(bytes, i + keyword_len);
        // `import('./x')` / `import ( './x' )` — step over the call paren.
        let call_import = j < len && bytes.get(j) == Some(&b'(');
        if call_import {
            j = skip_trivia(bytes, j + 1);
        }

        if let Some(&quote) = bytes.get(j).filter(|&&byte| byte == b'"' || byte == b'\'') {
            let start = j + 1;
            let mut k = start;
            while bytes.get(k).is_some_and(|&byte| byte != quote) {
                k += 1;
            }
            if k < len {
                let specifier = source.get(start..k).unwrap_or_default();
                let default_mode = if call_import {
                    vize_canon::PackageResolutionMode::Import
                } else {
                    keyword_mode
                };
                let mode = if keyword_mode != vize_canon::PackageResolutionMode::Require {
                    explicit_resolution_mode(source, k + 1, call_import).unwrap_or(default_mode)
                } else {
                    keyword_mode
                };
                specifiers.push(ModuleSpecifierOccurrence {
                    specifier: specifier.to_compact_string(),
                    mode,
                });
                i = k + 1;
                continue;
            }
        }
        // `import {` / `import Foo` — no string yet; keep scanning for `from`.
        i += keyword_len;
    }

    specifiers
}

fn explicit_resolution_mode(
    source: &str,
    after_specifier: usize,
    call_import: bool,
) -> Option<vize_canon::PackageResolutionMode> {
    let bytes = source.as_bytes();
    let mut cursor = skip_trivia(bytes, after_specifier);
    if call_import {
        if bytes.get(cursor) != Some(&b',') {
            return None;
        }
        cursor = skip_trivia(bytes, cursor + 1);
    } else {
        let keyword = [b"with".as_slice(), b"assert".as_slice()]
            .into_iter()
            .find(|keyword| matches_keyword(bytes, cursor, keyword))?;
        cursor = skip_trivia(bytes, cursor + keyword.len());
    }
    let (start, end) = object_literal_bounds(bytes, cursor)?;
    let mut at = start + 1;
    while at < end {
        at = skip_trivia(bytes, at);
        if at >= end {
            break;
        }
        let Some((key, key_end)) = string_literal_at(source, at) else {
            at += 1;
            continue;
        };
        if key != "resolution-mode" {
            at = key_end;
            continue;
        }
        let colon = skip_trivia(bytes, key_end);
        if bytes.get(colon) != Some(&b':') {
            at = key_end;
            continue;
        }
        let value_start = skip_trivia(bytes, colon + 1);
        let (value, _) = string_literal_at(source, value_start)?;
        return vize_canon::PackageResolutionMode::from_explicit_attribute(value);
    }
    None
}

fn object_literal_bounds(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    if bytes.get(start) != Some(&b'{') {
        return None;
    }
    let mut depth = 0usize;
    let mut cursor = start;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'\'' | b'"' => {
                cursor = skip_quoted(bytes, cursor)?;
                continue;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(bytes, cursor);
                continue;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                cursor = skip_block_comment(bytes, cursor)?;
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some((start, cursor));
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn string_literal_at(source: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let quote = *bytes.get(start)?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let end = skip_quoted(bytes, start)?;
    Some((source.get(start + 1..end - 1).unwrap_or_default(), end))
}

fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    let mut cursor = start + 1;
    while let Some(&byte) = bytes.get(cursor) {
        if byte == b'\\' {
            cursor += 2;
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

fn skip_trivia(bytes: &[u8], mut cursor: usize) -> usize {
    loop {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) == Some(&b'/') && bytes.get(cursor + 1) == Some(&b'/') {
            cursor = skip_line_comment(bytes, cursor);
            continue;
        }
        if bytes.get(cursor) == Some(&b'/') && bytes.get(cursor + 1) == Some(&b'*') {
            let Some(next) = skip_block_comment(bytes, cursor) else {
                return bytes.len();
            };
            cursor = next;
            continue;
        }
        return cursor;
    }
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 2;
    while cursor < bytes.len() {
        if let Some(len) = line_terminator_len(bytes, cursor) {
            return cursor + len;
        }
        cursor += 1;
    }
    bytes.len()
}

fn line_terminator_len(bytes: &[u8], cursor: usize) -> Option<usize> {
    match bytes.get(cursor)? {
        b'\n' => Some(1),
        b'\r' => Some(if bytes.get(cursor + 1) == Some(&b'\n') {
            2
        } else {
            1
        }),
        0xe2 if bytes.get(cursor..cursor + 3) == Some(b"\xe2\x80\xa8".as_slice()) => Some(3),
        0xe2 if bytes.get(cursor..cursor + 3) == Some(b"\xe2\x80\xa9".as_slice()) => Some(3),
        _ => None,
    }
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    bytes
        .get(start + 2..)?
        .windows(2)
        .position(|window| window == b"*/")
        .map(|offset| start + 2 + offset + 2)
}

/// Whether `bytes[at..]` begins with `keyword` as a standalone identifier token.
fn matches_keyword(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    if bytes.get(at..at + keyword.len()) != Some(keyword) {
        return false;
    }
    let before_ok = !at
        .checked_sub(1)
        .and_then(|prev| bytes.get(prev))
        .is_some_and(|&byte| is_identifier_byte(byte));
    let after_ok = !bytes
        .get(at + keyword.len())
        .is_some_and(|&byte| is_identifier_byte(byte));
    before_ok && after_ok
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

pub(super) fn is_relative_specifier(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

#[cfg(test)]
#[path = "imports_specifiers_tests.rs"]
mod tests;
