use vize_l0::String;

/// The pinned JSX transform drops normalized-empty text, including its helper.
pub(super) fn has_text(value: &str) -> bool {
    chars(value).next().is_some()
}

/// Normalize actual retained values; unchanged payloads need no allocation.
/// Entities stay refused before writing. Unicode line separators are text.
pub(super) fn normalized(value: &str) -> Option<String> {
    if !value.contains(['\r', '\n', '\t']) {
        return None;
    }
    Some(chars(value).collect())
}

/// Presence and emission consume the same borrowed normalized character stream.
fn chars(value: &str) -> impl Iterator<Item = char> + '_ {
    // Splitting CRLF into an extra empty line has the same target semantics.
    let (last_line, last_non_empty) = if value.contains(['\r', '\n']) {
        value
            .split(['\r', '\n'])
            .enumerate()
            .fold((0, 0), |(_, last_non_empty), (index, line)| {
                let last_non_empty = if line.contains(|character| !matches!(character, ' ' | '\t'))
                {
                    index
                } else {
                    last_non_empty
                };
                (index, last_non_empty)
            })
    } else {
        (0, 0)
    };
    value
        .split(['\r', '\n'])
        .enumerate()
        .flat_map(move |(index, line)| {
            let line = if index == 0 {
                line
            } else {
                line.trim_start_matches([' ', '\t'])
            };
            let line = if index == last_line {
                line
            } else {
                line.trim_end_matches([' ', '\t'])
            };
            line.chars()
                .map(|character| if character == '\t' { ' ' } else { character })
                .chain((!line.is_empty() && index != last_non_empty).then_some(' '))
        })
}
