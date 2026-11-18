//! Standalone layout for a value authored across lines and printed on one.

use super::format_directive_value;
use crate::options::FormatOptions;
use vize_l0::String;

/// A value authored across lines that prints on one line overflowing its
/// attribute line moves under the attribute, where the next pass reads it as
/// a standalone value and gives it the value budget. Format that standalone
/// spelling now so both passes agree. `None` keeps the printed value.
#[cold]
#[inline(never)]
pub(super) fn format(
    (name, value, trimmed): (&str, &str, &str),
    (options, vue_version, attribute_depth): (&FormatOptions, crate::VueVersion, usize),
) -> Option<(String, bool, bool)> {
    let starts_inline = value
        .split_once('\n')
        .is_some_and(|(first_line, _)| !first_line.trim().is_empty());
    if !starts_inline {
        return None;
    }
    let mut standalone = String::with_capacity(trimmed.len() + 1);
    standalone.push('\n');
    standalone.push_str(trimmed);
    Some(format_directive_value(
        name,
        &standalone,
        options,
        vue_version,
        attribute_depth,
    ))
}
