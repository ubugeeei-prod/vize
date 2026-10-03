use vize_l0::String;

/// The pinned JSX transform drops normalized-empty text, including its helper.
pub(super) fn has_text(value: &str) -> bool {
    if value.contains(['\r', '\n']) {
        value.contains(|character| !matches!(character, ' ' | '\t' | '\r' | '\n'))
    } else {
        !value.is_empty()
    }
}

/// Normalize actual retained values; unchanged payloads need no allocation.
/// Entities stay refused before writing. Unicode line separators are text.
pub(super) fn normalized(value: &str) -> Option<String> {
    if !value.contains(['\r', '\n', '\t']) {
        return None;
    }
    // Splitting CRLF into an extra empty line has the same target semantics.
    let last_non_empty = value
        .split(['\r', '\n'])
        .enumerate()
        .filter(|(_, line)| line.contains(|character| !matches!(character, ' ' | '\t')))
        .map(|(index, _)| index)
        .last()
        .unwrap_or(0);
    let mut lines = value.split(['\r', '\n']).enumerate().peekable();
    let mut output = String::default();
    while let Some((index, line)) = lines.next() {
        let line = if index == 0 {
            line
        } else {
            line.trim_start_matches([' ', '\t'])
        };
        let line = if lines.peek().is_none() {
            line
        } else {
            line.trim_end_matches([' ', '\t'])
        };
        if !line.is_empty() {
            for (part, text) in line.split('\t').enumerate() {
                if part > 0 {
                    output.push(' ');
                }
                output.push_str(text);
            }
            if index != last_non_empty {
                output.push(' ');
            }
        }
    }
    Some(output)
}
