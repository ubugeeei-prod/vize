//! Selective lightningcss fixed-point passes.
//!
//! The real-project idempotence corpus found a small set of constructs whose
//! first printed form is not stable. Re-parsing all other style blocks doubled
//! their parse/print work to guard those properties.

use crate::error::FormatError;
use memchr::memmem;
use vize_s0::String;

/// Upper bound for a pathological non-converging lightningcss value.
const MAX_PASSES: usize = 4;

pub(super) fn format_to_fixed_point(
    source: &str,
    mut format_once: impl FnMut(&str) -> Result<String, FormatError>,
) -> Result<String, FormatError> {
    let mut current = format_once(source)?;
    if !may_need_another_pass(source.as_bytes(), current.as_bytes()) {
        return Ok(current);
    }

    for _ in 1..MAX_PASSES {
        let next = format_once(current.as_str())?;
        if next == current {
            return Ok(next);
        }
        current = next;
    }
    Ok(current)
}

fn may_need_another_pass(source: &[u8], printed: &[u8]) -> bool {
    memmem::find(printed, b"background-position").is_some()
        // lightningcss drops the unsupported legacy rule on its first pass but
        // leaves its surrounding whitespace behind until the second pass.
        || contains_legacy_ms_keyframes(source)
        // lightningcss first preserves this large float sentinel, then reparses
        // it to the stable integer maximum on a subsequent pass. Keep the
        // extra pass selective so ordinary style blocks still pay one parse.
        || contains_float_max_sentinel(source)
        || contains_float_max_sentinel(printed)
}

fn contains_legacy_ms_keyframes(source: &[u8]) -> bool {
    const NAME: &[u8] = b"@-ms-keyframes";

    memchr::memchr_iter(b'@', source).any(|start| {
        source
            .get(start..start + NAME.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(NAME))
    })
}

fn contains_float_max_sentinel(source: &[u8]) -> bool {
    const SENTINEL: &[u8] = b"3.40282e38";

    let mut index = 0usize;
    let mut string_quote: Option<u8> = None;
    let mut declaration_start = 0usize;
    let mut custom_property_value = false;

    while index < source.len() {
        let byte = source[index];
        if let Some(quote) = string_quote {
            if byte == b'\\' && index + 1 < source.len() {
                index += 2;
                continue;
            }
            if byte == quote {
                string_quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' | b'\'' => {
                string_quote = Some(byte);
                index += 1;
            }
            b'/' if source.get(index + 1) == Some(&b'*') => {
                index = comment_end(source, index + 2);
            }
            b'u' | b'U' => match url_function_body_start(source, index) {
                Some(body_start) => index = function_end(source, body_start),
                None => index += 1,
            },
            b'{' | b';' | b'}' => {
                declaration_start = index + 1;
                custom_property_value = false;
                index += 1;
            }
            b':' => {
                custom_property_value =
                    declaration_is_custom_property(source, declaration_start, index);
                index += 1;
            }
            b'3' if !custom_property_value
                && source
                    .get(index..index + SENTINEL.len())
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(SENTINEL))
                && is_numeric_token_start(source, index)
                && !is_number_continuation(source, index + SENTINEL.len()) =>
            {
                return true;
            }
            _ => index += 1,
        }
    }

    false
}

fn is_numeric_token_start(source: &[u8], start: usize) -> bool {
    match start.checked_sub(1).and_then(|index| source.get(index)) {
        None => true,
        Some(b'+' | b'-') => start <= 1 || is_numeric_token_boundary(source[start - 2]),
        Some(previous) => is_numeric_token_boundary(*previous),
    }
}

fn is_numeric_token_boundary(byte: u8) -> bool {
    !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-' | b'.')
}

fn is_number_continuation(source: &[u8], end: usize) -> bool {
    source
        .get(end)
        .is_some_and(|next| next.is_ascii_digit() || *next == b'.')
}

fn url_function_body_start(source: &[u8], start: usize) -> Option<usize> {
    if start > 0 && !is_css_identifier_boundary(source[start - 1]) {
        return None;
    }
    if !source
        .get(start..start + 3)
        .is_some_and(|name| name.eq_ignore_ascii_case(b"url"))
    {
        return None;
    }

    let mut index = start + 3;
    while source
        .get(index)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        index += 1;
    }
    if source.get(index) == Some(&b'(') {
        Some(index + 1)
    } else {
        None
    }
}

fn is_css_identifier_boundary(byte: u8) -> bool {
    !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-')
}

fn comment_end(source: &[u8], from: usize) -> usize {
    memmem::find(&source[from..], b"*/").map_or(source.len(), |offset| from + offset + 2)
}

fn function_end(source: &[u8], from: usize) -> usize {
    let mut index = from;
    let mut depth = 1usize;
    let mut string_quote: Option<u8> = None;

    while index < source.len() {
        let byte = source[index];
        if let Some(quote) = string_quote {
            if byte == b'\\' && index + 1 < source.len() {
                index += 2;
                continue;
            }
            if byte == quote {
                string_quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' | b'\'' => {
                string_quote = Some(byte);
                index += 1;
            }
            b'/' if source.get(index + 1) == Some(&b'*') => {
                index = comment_end(source, index + 2);
            }
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth -= 1;
                index += 1;
                if depth == 0 {
                    return index;
                }
            }
            _ => index += 1,
        }
    }

    source.len()
}

fn declaration_is_custom_property(source: &[u8], boundary: usize, colon: usize) -> bool {
    let mut property_start = boundary;
    while property_start < colon && source[property_start].is_ascii_whitespace() {
        property_start += 1;
    }
    source
        .get(property_start..property_start + 2)
        .is_some_and(|candidate| candidate == b"--")
}

#[cfg(test)]
mod tests {
    use super::format_to_fixed_point;
    use crate::{options::FormatOptions, style::format_style_content};
    use std::cell::Cell;
    use vize_s0::ToCompactString;

    #[test]
    fn ordinary_css_is_parsed_and_printed_once() {
        let calls = Cell::new(0);
        let result = format_to_fixed_point(".a{color:red}", |source| {
            calls.set(calls.get() + 1);
            Ok(source.to_compact_string())
        })
        .unwrap();

        assert_eq!(result.as_str(), ".a{color:red}");
        assert_eq!(calls.get(), 1, "ordinary CSS must not pay a stability pass");
    }

    #[test]
    fn background_position_runs_until_the_printed_form_is_stable() {
        let calls = Cell::new(0);
        let result = format_to_fixed_point("input", |_| {
            let pass = calls.get();
            calls.set(pass + 1);
            Ok(match pass {
                0 => ".a { background-position: 1em 50%; }",
                _ => ".a { background-position: 1em; }",
            }
            .to_compact_string())
        })
        .unwrap();

        assert_eq!(result.as_str(), ".a { background-position: 1em; }");
        assert_eq!(
            calls.get(),
            3,
            "the stable result must be observed, not assumed"
        );
    }

    #[test]
    fn legacy_keyframes_reach_fixed_point_in_one_pass() {
        let options = FormatOptions::default();
        for source in [
            concat!(
                "@-moz-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
                "@-ms-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
                "@keyframes orbit { 0% { transform: rotate(0deg); } }",
            ),
            concat!(
                "@-moz-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
                "@-MS-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
                "@keyframes orbit { 0% { transform: rotate(0deg); } }",
            ),
        ] {
            let result = format_style_content(source, &options).unwrap();
            let again = format_style_content(&result, &options).unwrap();

            assert_eq!(
                result, again,
                "legacy keyframe normalization must be idempotent after one format"
            );
        }
    }

    #[test]
    fn float_max_sentinel_reaches_fixed_point_in_one_pass() {
        let options = FormatOptions::default();
        for source in [
            ".a { max-height: 3.40282e38px; }",
            concat!(
                ".group\\/item.group\\/nested-items-open > * > ",
                ".group\\/items.translate-x-0 .group\\/button { ",
                "max-height: calc(infinity * 1px); display: flex; }",
            ),
        ] {
            let result = format_style_content(source, &options).unwrap();
            let again = format_style_content(result.as_str(), &options).unwrap();

            assert_eq!(result, again);
            assert!(result.as_str().contains("max-height: 2147483647px;"));
        }
    }

    #[test]
    fn float_max_sentinel_ignores_non_numeric_css_text() {
        for source in [
            r#".a { content: "3.40282e38"; }"#,
            ".a { --limit: 3.40282e38px; }",
            ".a { background-image: url(3.40282e38.png); }",
            ".a { color: x3.40282e38; }",
            ".a { max-height: 3.40282e380px; }",
        ] {
            let calls = Cell::new(0);
            let result = format_to_fixed_point(source, |source| {
                calls.set(calls.get() + 1);
                Ok(source.to_compact_string())
            })
            .unwrap();

            assert_eq!(result.as_str(), source);
            assert_eq!(calls.get(), 1, "{source} must not request a stability pass");
        }
    }
}
