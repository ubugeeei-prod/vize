//! Selective lightningcss fixed-point passes.
//!
//! The real-project idempotence corpus found one construct whose first printed
//! form is not stable: `background-position` shorthand (#3248). Re-parsing all
//! other style blocks doubled their parse/print work to guard one property.

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
    if !may_need_another_pass(current.as_bytes()) {
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

fn may_need_another_pass(printed: &[u8]) -> bool {
    memmem::find(printed, b"background-position").is_some()
}

#[cfg(test)]
mod tests {
    use super::format_to_fixed_point;
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
}
