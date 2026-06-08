use vize_carton::{CompactString, SmallVec};

use crate::drawer::helpers::is_valid_identifier_fast;

/// Extract parameters from inline arrow function or function expression
#[inline]
pub fn extract_inline_callback_params(expr: &str) -> Option<SmallVec<[CompactString; 4]>> {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return None;
    }

    // Skip leading whitespace
    let mut i = 0;
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= len {
        return None;
    }

    // Fast path: check for arrow "=>"
    let arrow_pos = find_arrow(bytes, i);

    if let Some(arrow_idx) = arrow_pos {
        let mut end = arrow_idx;
        while end > i && bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        if end <= i {
            return None;
        }

        let before_bytes = &bytes[i..end];

        // Check for async prefix
        let (param_start, param_end) = if before_bytes.starts_with(b"async")
            && before_bytes.len() > 5
            && before_bytes[5].is_ascii_whitespace()
        {
            let mut s = 5;
            while s < before_bytes.len() && before_bytes[s].is_ascii_whitespace() {
                s += 1;
            }
            (i + s, end)
        } else {
            (i, end)
        };

        let param_bytes = &bytes[param_start..param_end];

        // (params) => pattern
        if param_bytes.first() == Some(&b'(') && param_bytes.last() == Some(&b')') {
            let inner = &expr[param_start + 1..param_end - 1];
            let inner_trimmed = inner.trim();
            if inner_trimmed.is_empty() {
                return Some(SmallVec::new());
            }
            return Some(extract_param_list_fast(inner_trimmed));
        }

        // Single param: e =>
        let param = &expr[param_start..param_end];
        if is_valid_identifier_fast(param.as_bytes()) {
            let mut result = SmallVec::new();
            result.push(CompactString::new(param));
            return Some(result);
        }
    }

    // Check for function expression
    if bytes[i..].starts_with(b"function") {
        let fn_end = i + 8;
        let mut paren_start = fn_end;
        while paren_start < len && bytes[paren_start] != b'(' {
            paren_start += 1;
        }
        if paren_start >= len {
            return None;
        }
        let mut paren_end = paren_start + 1;
        let mut depth = 1;
        while paren_end < len && depth > 0 {
            match bytes[paren_end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            paren_end += 1;
        }
        if depth == 0 {
            let inner = &expr[paren_start + 1..paren_end - 1];
            let inner_trimmed = inner.trim();
            if inner_trimmed.is_empty() {
                return Some(SmallVec::new());
            }
            return Some(extract_param_list_fast(inner_trimmed));
        }
    }

    None
}

/// Find arrow "=>" position in bytes
#[inline]
fn find_arrow(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    if len < start + 2 {
        return None;
    }
    let mut i = start;
    while i < len - 1 {
        if bytes[i] == b'=' && bytes[i + 1] == b'>' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Extract parameter list from comma-separated string
#[inline]
fn extract_param_list_fast(params: &str) -> SmallVec<[CompactString; 4]> {
    let bytes = params.as_bytes();
    let len = bytes.len();
    let mut result = SmallVec::new();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Skip rest parameter prefix (...)
        if i + 2 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            i += 3;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        }

        // Skip destructuring patterns
        if i < len && (bytes[i] == b'{' || bytes[i] == b'[') {
            let open = bytes[i];
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 1;
            i += 1;
            while i < len && depth > 0 {
                if bytes[i] == open {
                    depth += 1;
                } else if bytes[i] == close {
                    depth -= 1;
                }
                i += 1;
            }
            while i < len && bytes[i] != b',' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
            continue;
        }

        // Extract identifier
        let ident_start = i;
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }

        if i > ident_start {
            result.push(CompactString::new(&params[ident_start..i]));
        }

        // Skip to next comma
        while i < len && bytes[i] != b',' {
            i += 1;
        }
        if i < len {
            i += 1;
        }
    }

    result
}
