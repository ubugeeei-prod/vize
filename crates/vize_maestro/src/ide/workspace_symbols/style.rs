//! Selector names for workspace symbols from `<style>` rule preludes.

/// `.class` and `#id` names in rule preludes, with their offsets.
/// Declaration values (`#fff`, `.5s`, `url(a.png)`), comments and
/// strings are not selectors.
pub(super) fn css_selector_names(style: &str) -> Vec<(usize, char, &str)> {
    let bytes = style.as_bytes();
    let mut names = Vec::new();
    let mut prelude = 0;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = style
                    .get(index + 2..)
                    .and_then(|rest| rest.find("*/"))
                    .map_or(bytes.len(), |end| index + 2 + end + 2);
                continue;
            }
            b'"' | b'\'' => index = css_string_end(bytes, index),
            b'{' => {
                prelude_names(style, prelude, index, &mut names);
                prelude = index + 1;
            }
            b'}' | b';' => prelude = index + 1,
            _ => {}
        }
        index += 1;
    }
    names
}

fn css_string_end(bytes: &[u8], open: usize) -> usize {
    let quote = bytes.get(open).copied();
    let mut index = open + 1;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' {
            index += 2;
            continue;
        }
        if Some(byte) == quote {
            return index;
        }
        index += 1;
    }
    bytes.len()
}

fn prelude_names<'a>(
    style: &'a str,
    start: usize,
    end: usize,
    names: &mut Vec<(usize, char, &'a str)>,
) {
    let bytes = style.as_bytes();
    let mut index = start;
    while index < end {
        let Some(&byte) = bytes.get(index) else { break };
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = style
                    .get(index + 2..end)
                    .and_then(|rest| rest.find("*/"))
                    .map_or(end, |close| index + 2 + close + 2);
                continue;
            }
            b'"' | b'\'' => index = css_string_end(bytes, index),
            b'[' => {
                index = style
                    .get(index..end)
                    .and_then(|rest| rest.find(']'))
                    .map_or(end, |close| index + close);
            }
            b'.' | b'#' => {
                let name_start = index + 1;
                let len = style
                    .get(name_start..end)
                    .unwrap_or_default()
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                    .unwrap_or(end - name_start);
                let name = style.get(name_start..name_start + len).unwrap_or_default();
                let starts_ident = name
                    .trim_start_matches('-')
                    .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_');
                if starts_ident {
                    names.push((name_start, char::from(byte), name));
                }
                index = name_start + len;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
}
