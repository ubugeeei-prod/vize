use vize_carton::String;

use super::find_matching_paren;

pub(super) fn normalize_scoped_selector_body(selector: &str) -> String {
    let selector = super::unwrap_pseudo_functions(selector, &["::v-global(", ":global("]);
    normalize(selector.as_str())
}

fn normalize(selector: &str) -> String {
    let Some(marker) = find_marker_from(selector, 0) else {
        return String::from(selector);
    };

    let before = selector[..marker.start].trim_end();
    let after = selector[marker.end..].trim_start();
    let target = deep_target(after).unwrap_or_else(|| String::from(after));
    let target = strip_legacy_markers(target.as_str());

    let mut output = String::with_capacity(selector.len() + 8);
    output.push_str(before);
    if !before.is_empty() {
        output.push(' ');
    }
    output.push_str(":deep(");
    output.push_str(target.as_str());
    output.push(')');

    output
}

fn deep_target(after: &str) -> Option<String> {
    if !after.starts_with('(') {
        return None;
    }

    let end = find_matching_paren(after, 0)?;
    let mut target = String::with_capacity(after.len().saturating_sub(2));
    target.push_str(&after[1..end]);
    target.push_str(&after[end + 1..]);
    Some(target)
}

fn strip_legacy_markers(selector: &str) -> String {
    let mut output = String::with_capacity(selector.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while let Some(marker) = find_marker_from(selector, cursor) {
        output.push_str(selector[cursor..marker.start].trim_end());
        push_descendant_space(&mut output);
        cursor = skip_ws(selector, marker.end);
        if let Some(target) = parenthesized_deep_target(selector, cursor) {
            output.push_str(
                strip_legacy_markers(&selector[target.inner_start..target.inner_end]).trim(),
            );
            cursor = target.end;
        }
        changed = true;
    }

    if !changed {
        return String::from(selector);
    }

    output.push_str(&selector[cursor..]);
    output
}

struct ParenthesizedTarget {
    inner_start: usize,
    inner_end: usize,
    end: usize,
}

fn parenthesized_deep_target(selector: &str, cursor: usize) -> Option<ParenthesizedTarget> {
    if !selector[cursor..].starts_with('(') {
        return None;
    }

    let inner_start = cursor + 1;
    let inner_end = find_matching_paren(selector, cursor)?;
    Some(ParenthesizedTarget {
        inner_start,
        inner_end,
        end: inner_end + 1,
    })
}

fn push_descendant_space(output: &mut String) {
    if !output.is_empty()
        && output
            .chars()
            .next_back()
            .is_some_and(|char| !char.is_whitespace())
    {
        output.push(' ');
    }
}

fn skip_ws(input: &str, cursor: usize) -> usize {
    input[cursor..]
        .char_indices()
        .find(|(_, char)| !char.is_whitespace())
        .map_or(input.len(), |(index, _)| cursor + index)
}

struct Marker {
    start: usize,
    end: usize,
}

fn find_marker_from(selector: &str, cursor: usize) -> Option<Marker> {
    let mut scanner = Scanner::new(selector, cursor);

    while let Some(index) = scanner.next_syntax_index() {
        if let Some(marker) = marker_at(selector, index) {
            return Some(marker);
        }
    }

    None
}

fn marker_at(selector: &str, index: usize) -> Option<Marker> {
    for marker in [">>>", "/deep/", "::v-deep"] {
        if selector[index..].starts_with(marker)
            && has_marker_boundary(selector, index + marker.len())
        {
            return Some(Marker {
                start: index,
                end: index + marker.len(),
            });
        }
    }

    None
}

fn has_marker_boundary(selector: &str, index: usize) -> bool {
    selector[index..]
        .chars()
        .next()
        .is_none_or(|char| !matches!(char, '-' | '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
}

struct Scanner<'a> {
    selector: &'a str,
    cursor: usize,
    quote: Option<u8>,
    bracket_depth: usize,
    in_comment: bool,
}

impl<'a> Scanner<'a> {
    fn new(selector: &'a str, cursor: usize) -> Self {
        Self {
            selector,
            cursor,
            quote: None,
            bracket_depth: 0,
            in_comment: false,
        }
    }

    fn next_syntax_index(&mut self) -> Option<usize> {
        while self.cursor < self.selector.len() {
            let index = self.cursor;
            let bytes = self.selector.as_bytes();
            let byte = bytes[index];
            let next = bytes.get(index + 1).copied();

            if self.in_comment {
                self.cursor += 1;
                if byte == b'*' && next == Some(b'/') {
                    self.cursor += 1;
                    self.in_comment = false;
                }
                continue;
            }

            if let Some(quote) = self.quote {
                self.cursor += if byte == b'\\' && next.is_some() {
                    2
                } else {
                    1
                };
                if byte == quote {
                    self.quote = None;
                }
                continue;
            }

            match byte {
                b'\\' => {
                    self.cursor += if next.is_some() { 2 } else { 1 };
                }
                b'/' if next == Some(b'*') => {
                    self.cursor += 2;
                    self.in_comment = true;
                }
                b'\'' | b'"' => {
                    self.cursor += 1;
                    self.quote = Some(byte);
                }
                b'[' => {
                    self.cursor += 1;
                    self.bracket_depth += 1;
                }
                b']' => {
                    self.cursor += 1;
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                }
                _ => {
                    self.cursor += byte_char_len(byte);
                    if self.bracket_depth == 0 {
                        return Some(index);
                    }
                }
            }
        }

        None
    }
}

fn byte_char_len(byte: u8) -> usize {
    if byte < 0x80 {
        1
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}
