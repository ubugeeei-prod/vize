//! Selective lightningcss fixed-point passes.
//!
//! The real-project idempotence corpus found a small set of constructs whose
//! first printed form is not stable. Re-parsing all other style blocks doubled
//! their parse/print work to guard those properties.

use crate::error::FormatError;
use memchr::memmem;
use vize_carton::String;

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
}

fn contains_legacy_ms_keyframes(source: &[u8]) -> bool {
    const NAME: &[u8] = b"@-ms-keyframes";

    memchr::memchr_iter(b'@', source).any(|start| {
        source
            .get(start..start + NAME.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(NAME))
    })
}

#[cfg(test)]
mod tests {
    use super::format_to_fixed_point;
    use crate::{options::FormatOptions, style::format_style_content};
    use std::cell::Cell;
    use vize_carton::ToCompactString;

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
}
