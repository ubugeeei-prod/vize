//! JSX text whitespace: what a markup text child renders.
//!
//! Lines are trimmed at their inner edges, lines left empty are dropped, the
//! remaining lines join with one space and tabs render as spaces. Text without
//! a line break or tab is unchanged.

use vize_l0::String;

/// The rendered text, borrowed when unchanged; empty when nothing renders.
pub(super) fn normalized(value: &str) -> Option<String> {
    if !value.contains(['\r', '\n', '\t']) {
        return None;
    }
    let lines = || value.split(['\r', '\n']);
    let last = lines().count().saturating_sub(1);
    // CRLF splits into an extra empty line, which renders the same.
    let last_non_empty = lines()
        .enumerate()
        .filter(|(_, line)| line.contains(|character| !matches!(character, ' ' | '\t')))
        .map(|(index, _)| index)
        .last()
        .unwrap_or(0);
    let mut rendered = String::with_capacity(value.len());
    for (index, line) in lines().enumerate() {
        let line = if index == 0 {
            line
        } else {
            line.trim_start_matches([' ', '\t'])
        };
        let line = if index == last {
            line
        } else {
            line.trim_end_matches([' ', '\t'])
        };
        rendered.extend(line.chars().map(|character| match character {
            '\t' => ' ',
            other => other,
        }));
        if !line.is_empty() && index != last_non_empty {
            rendered.push(' ');
        }
    }
    Some(rendered)
}
