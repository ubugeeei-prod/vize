//! Keep authored CSS color syntax while lightningcss formats the surrounding CSS.

use lightningcss::{traits::Parse, values::color::CssColor};
use vize_s0::String;

pub(super) struct ColorProtection<'a> {
    pub(super) source: String,
    originals: Vec<&'a str>,
    marker_prefix: std::string::String,
}

impl ColorProtection<'_> {
    pub(super) fn restore(&self, formatted: String) -> String {
        if self.originals.is_empty() {
            return formatted;
        }
        let mut result = formatted.as_str().to_owned();
        for (index, original) in self.originals.iter().enumerate() {
            result = result.replace(&marker(&self.marker_prefix, index), original);
        }
        result.into()
    }
}

pub(super) fn protect(source: &str) -> ColorProtection<'_> {
    let mut spans = Vec::new();
    for value in declaration_values(source) {
        scan_colors(source, value, &mut spans);
    }

    let mut marker_prefix = "--vize-glyph-preserve-color-".to_owned();
    while source.contains(&marker_prefix) {
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
        protected.push_str(&source[previous_end..start]);
        protected.push_str(&marker(&marker_prefix, originals.len()));
        originals.push(&source[start..end]);
        previous_end = end;
    }
    protected.push_str(&source[previous_end..]);

    ColorProtection {
        source: protected,
        originals,
        marker_prefix,
    }
}

fn marker(prefix: &str, index: usize) -> std::string::String {
    let mut marker = std::string::String::with_capacity(prefix.len() + 16);
    marker.push_str("var(");
    marker.push_str(prefix);
    marker.push_str(&index.to_string());
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

    while index < bytes.len() {
        match bytes[index] {
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
                if brace_depth > 0 {
                    if let Some(value_start) = declaration_value_start(&source[item_start..index]) {
                        values.push(item_start + value_start..index);
                    }
                }
                if bytes[index] == b'}' {
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
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => index = skip_comment(bytes, index),
            b':' => {
                let property = item[..index].trim();
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
        match bytes[index] {
            b'\'' | b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => index = skip_comment(bytes, index),
            b'#' => {
                let start = index;
                index += 1;
                while index < value.end && bytes[index].is_ascii_hexdigit() {
                    index += 1;
                }
                if CssColor::parse_string(&source[start..index]).is_ok() {
                    spans.push((start, index));
                }
            }
            byte if byte.is_ascii_alphabetic() || byte == b'-' => {
                let start = index;
                index += 1;
                while index < value.end
                    && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'-' | b'_'))
                {
                    index += 1;
                }
                if bytes.get(index) == Some(&b'(') {
                    if source[start..index].eq_ignore_ascii_case("url") {
                        index = skip_function(bytes, index, value.end);
                        continue;
                    }
                    let end = skip_function(bytes, index, value.end);
                    if CssColor::parse_string(&source[start..end]).is_ok() {
                        spans.push((start, end));
                        index = end;
                    } else {
                        index += 1;
                    }
                } else if CssColor::parse_string(&source[start..index]).is_ok() {
                    spans.push((start, index));
                }
            }
            _ => index += 1,
        }
    }
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    index
}

fn skip_comment(bytes: &[u8], start: usize) -> usize {
    memchr::memmem::find(&bytes[start + 2..], b"*/").map_or(bytes.len(), |end| start + 2 + end + 2)
}

fn skip_function(bytes: &[u8], open: usize, limit: usize) -> usize {
    let mut depth = 1;
    let mut index = open + 1;
    while index < limit {
        match bytes[index] {
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
