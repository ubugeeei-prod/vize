//! Rename scoped keyframes and their animation references.

use regex::Regex;
use std::sync::LazyLock;
use vize_carton::String;

static KEYFRAMES_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)@(?:-webkit-|-moz-|-o-)?keyframes\s+([_a-z][\w-]*)").ok());
static ANIMATION_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)(?:^|[;{])\s*(?:-webkit-)?animation(?:-name)?\s*:\s*").ok());

/// Mask strings and comments while retaining byte offsets into the original CSS.
fn code_mask(css: &str) -> Vec<u8> {
    let bytes = css.as_bytes();
    let mut masked = bytes.to_vec();
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index..index + 2) == Some(b"/*".as_slice()) {
            let start = index;
            index += 2;
            while index + 1 < bytes.len() && bytes.get(index..index + 2) != Some(b"*/".as_slice()) {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            masked[start..index].fill(b' ');
        } else if bytes[index] == b'\'' || bytes[index] == b'"' {
            let start = index;
            let quote = bytes[index];
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == quote {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
            masked[start..index].fill(b' ');
        } else {
            if !bytes[index].is_ascii() {
                masked[index] = b' ';
            }
            index += 1;
        }
    }
    masked
}

fn ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

pub(super) fn scope_keyframes(css: &str, scope_id: &str) -> String {
    let Some(keyframes_pattern) = KEYFRAMES_PATTERN.as_ref() else {
        return css.into();
    };
    let Some(animation_pattern) = ANIMATION_PATTERN.as_ref() else {
        return css.into();
    };
    let suffix = scope_id.strip_prefix("data-v-").unwrap_or(scope_id);
    let mask = code_mask(css);
    let mask = std::str::from_utf8(&mask).unwrap_or_default();
    let mut names = Vec::<&str>::new();
    let mut edits = Vec::<(usize, usize, String)>::new();

    for capture in keyframes_pattern.captures_iter(mask) {
        let Some(name_match) = capture.get(1) else {
            continue;
        };
        let Some(name) = css.get(name_match.start()..name_match.end()) else {
            continue;
        };
        if !names.contains(&name) {
            names.push(name);
        }
        edits.push((
            name_match.start(),
            name_match.end(),
            suffixed_name(name, suffix),
        ));
    }

    if names.is_empty() {
        return css.into();
    }

    let bytes = mask.as_bytes();
    for declaration in animation_pattern.find_iter(mask) {
        let mut index = declaration.end();
        let mut parens = 0usize;
        while index < bytes.len() {
            match bytes[index] {
                b'(' => parens += 1,
                b')' => parens = parens.saturating_sub(1),
                b';' | b'}' if parens == 0 => break,
                _ => {}
            }
            if ident_byte(bytes[index]) {
                let start = index;
                index += 1;
                while index < bytes.len() && ident_byte(bytes[index]) {
                    index += 1;
                }
                if let Some(name) = css.get(start..index)
                    && names.contains(&name)
                {
                    edits.push((start, index, suffixed_name(name, suffix)));
                }
                continue;
            }
            index += 1;
        }
    }

    edits.sort_unstable_by_key(|(start, _, _)| *start);
    let mut output = String::with_capacity(css.len() + edits.len() * (suffix.len() + 1));
    let mut last = 0;
    for (start, end, replacement) in edits {
        if start < last {
            continue;
        }
        if let Some(fragment) = css.get(last..start) {
            output.push_str(fragment);
            output.push_str(&replacement);
            last = end;
        }
    }
    if let Some(fragment) = css.get(last..) {
        output.push_str(fragment);
    }
    output
}

fn suffixed_name(name: &str, suffix: &str) -> String {
    let mut result = String::with_capacity(name.len() + suffix.len() + 1);
    result.push_str(name);
    result.push('-');
    result.push_str(suffix);
    result
}
