//! Module-specifier scanning for the `vize check` transitive import walk.

use vize_carton::{String, ToCompactString};

/// Collect module specifiers of `source`'s import/export/dynamic-imports.
///
/// This is a deliberately lightweight byte scan rather than a full parse: the
/// transitive walk runs on every checked file, so an AST per file regressed the
/// benchmark. Over-matching (e.g. an import-like fragment inside a string) is
/// harmless because each specifier is resolved against the filesystem and only
/// real source files are registered.
pub(super) fn extract_import_specifiers(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut specifiers = Vec::new();
    let mut i = 0;

    while i < len {
        let keyword_len = if matches_keyword(bytes, i, b"from") {
            4
        } else if matches_keyword(bytes, i, b"import") {
            6
        } else {
            i += 1;
            continue;
        };

        let mut j = i + keyword_len;
        while j < len && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        // `import('./x')` / `import ( './x' )` — step over the call paren.
        if j < len && bytes[j] == b'(' {
            j += 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
        }

        if j < len && (bytes[j] == b'"' || bytes[j] == b'\'') {
            let quote = bytes[j];
            let start = j + 1;
            let mut k = start;
            while k < len && bytes[k] != quote {
                k += 1;
            }
            if k < len {
                let specifier = &source[start..k];
                specifiers.push(specifier.to_compact_string());
                i = k + 1;
                continue;
            }
        }
        // `import {` / `import Foo` — no string yet; keep scanning for `from`.
        i += keyword_len;
    }

    specifiers
}

/// Whether `bytes[at..]` begins with `keyword` as a standalone identifier token.
fn matches_keyword(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    if at + keyword.len() > bytes.len() || &bytes[at..at + keyword.len()] != keyword {
        return false;
    }
    let before_ok = at == 0 || !is_identifier_byte(bytes[at - 1]);
    let after = at + keyword.len();
    let after_ok = after >= bytes.len() || !is_identifier_byte(bytes[after]);
    before_ok && after_ok
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

pub(super) fn is_relative_specifier(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}
