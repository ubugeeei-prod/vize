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

    while i < len {
        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                i = skip_quoted(bytes, i).unwrap_or(len);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = bytes[i + 2..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(len, |offset| i + 2 + offset + 1);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = bytes[i + 2..]
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
        let call_import = j < len && bytes[j] == b'(';
        if call_import {
            j = skip_trivia(bytes, j + 1);
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
    while cursor < bytes.len() {
        match bytes[cursor] {
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
    Some((&source[start + 1..end - 1], end))
}

fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == quote {
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
    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| start + 2 + offset + 1)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start + 2..]
        .windows(2)
        .position(|window| window == b"*/")
        .map(|offset| start + 2 + offset + 2)
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

#[cfg(test)]
mod tests {
    use super::extract_module_specifier_occurrences;
    use vize_canon::PackageResolutionMode::{Contextual, Import, Require};

    #[test]
    fn preserves_import_and_require_occurrence_modes() {
        let source = r#"
import direct from "direct"
export { named } from 'exported'
const dynamic = import ( "dynamic" )
const commonjs = require('commonjs')
import legacy = require("import-equals")
"#;
        let actual = extract_module_specifier_occurrences(source)
            .into_iter()
            .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("direct".to_owned(), Contextual),
                ("exported".to_owned(), Contextual),
                ("dynamic".to_owned(), Import),
                ("commonjs".to_owned(), Require),
                ("import-equals".to_owned(), Require),
            ]
        );
    }

    #[test]
    fn explicit_resolution_mode_attributes_override_import_occurrences() {
        let source = r#"
import type { Static } from "static-package" with { "resolution-mode": "require" }
type Dynamic = import("dynamic-package", { with: { "resolution-mode": "require" } })
import type { Native } from "import-package" with { "resolution-mode": "import" }
"#;
        let actual = extract_module_specifier_occurrences(source)
            .into_iter()
            .filter(|occurrence| occurrence.specifier.contains("package"))
            .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("static-package".to_owned(), Require),
                ("dynamic-package".to_owned(), Require),
                ("import-package".to_owned(), Import),
            ]
        );
    }

    #[test]
    fn comments_inside_import_syntax_do_not_hide_specifiers() {
        let source = r#"
const view = import(
  /* webpackChunkName: "profile" */
  "dynamic-package",
  /* import attributes may be split across build-tool comments */
  {
    with: {
      /* keep package resolution parity with TypeScript */
      "resolution-mode": /* inline */ "require",
    },
  },
)
const commonjs = require(
  /* real-world generated helper comment */
  "commonjs-package"
)
export { named } from /* generated barrel comment */ "reexported-package"
import type { Static } from "static-package" /* bundler hint */ with /* mode */ {
  "resolution-mode": "require"
}
"#;
        let actual = extract_module_specifier_occurrences(source)
            .into_iter()
            .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("dynamic-package".to_owned(), Require),
                ("commonjs-package".to_owned(), Require),
                ("reexported-package".to_owned(), Contextual),
                ("static-package".to_owned(), Require),
            ]
        );
    }

    #[test]
    fn keywords_must_be_standalone_identifier_tokens() {
        let source = r#"
const imported = "ignored-one"
const requirement = "ignored-two"
import value from "kept"
"#;
        let actual = extract_module_specifier_occurrences(source);

        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].specifier.as_str(), "kept");
        assert_eq!(actual[0].mode, Contextual);
    }

    #[test]
    fn keywords_inside_strings_and_comments_cannot_consume_later_imports() {
        let source = r#"
const words = "require import from"
// require("ignored-comment")
/* import("ignored-block") */
import("kept-dynamic", { with: { "resolution-mode": "require" } })
import type { Kept } from "kept-static" with { "resolution-mode": "import" }
"#;
        let actual = extract_module_specifier_occurrences(source)
            .into_iter()
            .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("kept-dynamic".to_owned(), Require),
                ("kept-static".to_owned(), Import),
            ]
        );
    }
}
