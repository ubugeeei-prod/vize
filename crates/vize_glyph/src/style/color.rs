//! Keep authored CSS color syntax while lightningcss formats the surrounding CSS.

use lightningcss::{traits::Parse, values::color::CssColor};
use vize_s0::{String, ToCompactString};

pub(super) struct ColorProtection<'a> {
    pub(super) source: String,
    originals: Vec<&'a str>,
    marker_prefix: String,
}

impl ColorProtection<'_> {
    pub(super) fn restore(&self, formatted: String) -> String {
        if self.originals.is_empty() {
            return formatted;
        }
        let mut result = formatted;
        for (index, original) in self.originals.iter().enumerate() {
            result = result
                .replace(marker(&self.marker_prefix, index).as_str(), original)
                .into();
        }
        result
    }
}

pub(super) fn protect(source: &str) -> ColorProtection<'_> {
    let mut spans = Vec::new();
    for value in declaration_values(source) {
        scan_colors(source, value, &mut spans);
    }

    let mut marker_prefix = "--vize-glyph-preserve-color-".to_compact_string();
    while source.contains(marker_prefix.as_str()) {
        marker_prefix.push('-');
    }

    if spans.is_empty() {
        return ColorProtection {
            source: source.into(),
            originals: Vec::new(),
            marker_prefix,
        };
    }

    let mut protected = String::with_capacity(source.len() + spans.len() * 32);
    let mut originals = Vec::with_capacity(spans.len());
    let mut previous_end = 0;
    for (start, end) in spans {
        let (Some(before), Some(original)) =
            (source.get(previous_end..start), source.get(start..end))
        else {
            return ColorProtection {
                source: source.into(),
                originals: Vec::new(),
                marker_prefix,
            };
        };
        protected.push_str(before);
        protected.push_str(&marker(&marker_prefix, originals.len()));
        originals.push(original);
        previous_end = end;
    }
    protected.push_str(source.get(previous_end..).unwrap_or_default());

    ColorProtection {
        source: protected,
        originals,
        marker_prefix,
    }
}

fn marker(prefix: &str, index: usize) -> String {
    let mut marker = String::with_capacity(prefix.len() + 16);
    marker.push_str("var(");
    marker.push_str(prefix);
    marker.push_str(&index.to_compact_string());
    marker.push(')');
    marker
}

/// Return declaration value ranges, excluding selectors and at-rule preambles.
fn declaration_values(source: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = source.as_bytes();
    let mut values = Vec::new();
    let mut index = 0;
    let mut item_start = 0;
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;

    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\'' | b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_comment(bytes, index);
            }
            b'(' | b'[' => {
                paren_depth += 1;
                index += 1;
            }
            b')' | b']' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += 1;
            }
            b'{' if paren_depth == 0 => {
                brace_depth += 1;
                item_start = index + 1;
                index += 1;
            }
            b';' | b'}' if paren_depth == 0 => {
                if brace_depth > 0
                    && let Some(item) = source.get(item_start..index)
                    && let Some(value_start) = declaration_value_start(item)
                {
                    values.push(item_start + value_start..index);
                }
                if byte == b'}' {
                    brace_depth = brace_depth.saturating_sub(1);
                }
                item_start = index + 1;
                index += 1;
            }
            _ => index += 1,
        }
    }
    values
}

fn declaration_value_start(item: &str) -> Option<usize> {
    let bytes = item.as_bytes();
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\'' | b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => index = skip_comment(bytes, index),
            b':' => {
                let property = item.get(..index)?.trim();
                if property.is_empty()
                    || !property
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                {
                    return None;
                }
                return Some(index + 1);
            }
            _ => index += 1,
        }
    }
    None
}

fn scan_colors(source: &str, value: std::ops::Range<usize>, spans: &mut Vec<(usize, usize)>) {
    let bytes = source.as_bytes();
    let mut index = value.start;
    while index < value.end {
        let Some(&byte) = bytes.get(index) else {
            break;
        };
        match byte {
            b'\'' | b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => index = skip_comment(bytes, index),
            b'#' => {
                let start = index;
                index += 1;
                while index < value.end && bytes.get(index).is_some_and(u8::is_ascii_hexdigit) {
                    index += 1;
                }
                if is_color(source, start, index) {
                    spans.push((start, index));
                }
            }
            byte if byte.is_ascii_alphabetic() || byte == b'-' => {
                let start = index;
                index += 1;
                while index < value.end
                    && bytes.get(index).is_some_and(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
                    })
                {
                    index += 1;
                }
                if bytes.get(index) == Some(&b'(') {
                    if source
                        .get(start..index)
                        .is_some_and(|name| name.eq_ignore_ascii_case("url"))
                    {
                        index = skip_function(bytes, index, value.end);
                        continue;
                    }
                    let end = skip_function(bytes, index, value.end);
                    if is_color(source, start, end) {
                        spans.push((start, end));
                        index = end;
                    } else {
                        index += 1;
                    }
                } else if is_color(source, start, index) {
                    spans.push((start, index));
                }
            }
            _ => index += 1,
        }
    }
}

fn is_color(source: &str, start: usize, end: usize) -> bool {
    source
        .get(start..end)
        .is_some_and(|token| CssColor::parse_string(token).is_ok())
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let Some(&quote) = bytes.get(start) else {
        return bytes.len();
    };
    let mut index = start + 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    index
}

fn skip_comment(bytes: &[u8], start: usize) -> usize {
    bytes
        .get(start + 2..)
        .and_then(|rest| memchr::memmem::find(rest, b"*/"))
        .map_or(bytes.len(), |end| start + 2 + end + 2)
}

fn skip_function(bytes: &[u8], open: usize, limit: usize) -> usize {
    let mut depth = 1;
    let mut index = open + 1;
    while index < limit {
        let Some(&byte) = bytes.get(index) else {
            break;
        };
        match byte {
            b'\'' | b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => index = skip_comment(bytes, index),
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
    index
}
