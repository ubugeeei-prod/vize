//! Scoped CSS transformation.
//!
//! Applies Vue-style scoped CSS by adding attribute selectors (e.g., `[data-v-xxx]`)
//! to CSS selectors. Handles special pseudo-selectors: `:deep()`, `:slotted()`, `:global()`.

use vize_carton::{Bump, BumpVec};

use super::transform::{find_bytes, find_matching_paren, rfind_byte};

/// Apply scoped CSS transformation
pub(crate) fn apply_scoped_css<'a>(bump: &'a Bump, css: &str, scope_id: &str) -> &'a str {
    let css_bytes = css.as_bytes();

    // Build attr_selector: [scope_id]
    let mut attr_selector = BumpVec::with_capacity_in(scope_id.len() + 2, bump);
    attr_selector.push(b'[');
    attr_selector.extend_from_slice(scope_id.as_bytes());
    attr_selector.push(b']');
    let attr_selector = bump.alloc_slice_copy(&attr_selector);

    let mut output = BumpVec::with_capacity_in(css_bytes.len() * 2, bump);
    let mut chars = css.char_indices().peekable();
    let mut in_selector = true;
    let mut in_string = false;
    let mut string_char = b'"';
    let mut in_comment = false;
    let mut brace_depth = 0u32;
    let mut last_selector_end = 0usize;
    let mut in_at_rule = false;
    let mut at_rule_depth = 0u32;
    let mut pending_keyframes = false;
    let mut keyframes_brace_depth: Option<u32> = None;
    let mut saved_at_rule_depth: Option<u32> = None;

    while let Some((i, c)) = chars.next() {
        if in_comment {
            if c == '*' {
                if let Some(&(_, '/')) = chars.peek() {
                    chars.next();
                    in_comment = false;
                }
            }
            continue;
        }

        if in_string {
            if c as u8 == string_char {
                // Check for escape
                let prev_byte = if i > 0 { css_bytes[i - 1] } else { 0 };
                if prev_byte != b'\\' {
                    in_string = false;
                }
            }
            if !in_selector && !in_at_rule {
                output.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
            }
            continue;
        }

        match c {
            '"' | '\'' => {
                in_string = true;
                string_char = c as u8;
                if !in_selector && !in_at_rule {
                    output.push(c as u8);
                }
            }
            '/' => {
                if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    in_comment = true;
                } else if !in_selector && !in_at_rule {
                    output.push(b'/');
                }
            }
            '@' if in_selector => {
                in_at_rule = true;
                in_selector = false;
                // Look ahead to detect @keyframes (including vendor prefixes)
                let remaining = &css[i + 1..];
                pending_keyframes = remaining.starts_with("keyframes")
                    || remaining.starts_with("-webkit-keyframes")
                    || remaining.starts_with("-moz-keyframes")
                    || remaining.starts_with("-o-keyframes");
                // Don't output '@' — the entire at-rule header will be flushed
                // from the buffer when we encounter '{' or ';'
            }
            '@' => {
                // @ in non-selector context (e.g., CSS nesting @media inside a rule)
                output.push(b'@');
            }
            ';' if in_at_rule => {
                // Statement at-rule (e.g., @import, @charset, @namespace)
                // Flush the entire at-rule including the semicolon
                let stmt = &css_bytes[last_selector_end..=i];
                let stmt_str = unsafe { std::str::from_utf8_unchecked(stmt) }.trim();
                output.extend_from_slice(stmt_str.as_bytes());
                output.push(b'\n');
                in_at_rule = false;
                in_selector = true;
                pending_keyframes = false;
                last_selector_end = i + 1;
            }
            '{' => {
                brace_depth += 1;
                if in_at_rule {
                    in_at_rule = false;
                    // Flush the buffered at-rule header (e.g., "@media (--mobile)")
                    let at_rule_header = &css_bytes[last_selector_end..i];
                    let at_rule_str =
                        unsafe { std::str::from_utf8_unchecked(at_rule_header) }.trim();
                    output.extend_from_slice(at_rule_str.as_bytes());
                    output.push(b'{');
                    if pending_keyframes {
                        saved_at_rule_depth = Some(at_rule_depth);
                        keyframes_brace_depth = Some(brace_depth);
                        pending_keyframes = false;
                    }
                    at_rule_depth = brace_depth;
                    in_selector = true;
                    last_selector_end = i + 1;
                } else if keyframes_brace_depth.is_some_and(|d| brace_depth > d) {
                    // Inside @keyframes: output the stop name (from/to/0%/100%)
                    let kf_part = &css_bytes[last_selector_end..i];
                    let kf_str = unsafe { std::str::from_utf8_unchecked(kf_part) }.trim();
                    output.extend_from_slice(kf_str.as_bytes());
                    output.push(b'{');
                    in_selector = false;
                    last_selector_end = i + 1;
                } else if in_selector
                    && (brace_depth == 1 || (at_rule_depth > 0 && brace_depth > at_rule_depth))
                {
                    // End of selector, apply scope
                    let selector_bytes = &css_bytes[last_selector_end..i];
                    let selector_str =
                        unsafe { std::str::from_utf8_unchecked(selector_bytes) }.trim();
                    scope_selector(&mut output, selector_str, attr_selector);
                    output.push(b'{');
                    in_selector = false;
                    last_selector_end = i + 1;
                } else {
                    output.push(b'{');
                }
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                output.push(b'}');
                // Check @keyframes block end — restore parent at_rule_depth
                if keyframes_brace_depth.is_some_and(|d| brace_depth < d) {
                    keyframes_brace_depth = None;
                    if let Some(saved) = saved_at_rule_depth.take() {
                        at_rule_depth = saved;
                    }
                }
                if brace_depth == 0 {
                    in_selector = true;
                    last_selector_end = i + 1;
                    at_rule_depth = 0;
                } else if at_rule_depth > 0 && brace_depth >= at_rule_depth {
                    // Inside at-rule, back to selector mode for next rule
                    in_selector = true;
                    last_selector_end = i + 1;
                }
            }
            _ if in_selector || in_at_rule => {
                // Still building selector or at-rule header, don't output yet
            }
            _ => {
                output.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
            }
        }
    }

    // Handle any remaining content
    if in_selector && last_selector_end < css_bytes.len() {
        output.extend_from_slice(&css_bytes[last_selector_end..]);
    }

    // SAFETY: input is valid UTF-8, we only add ASCII bytes
    unsafe { std::str::from_utf8_unchecked(bump.alloc_slice_copy(&output)) }
}

/// Add scope attribute to a selector
fn scope_selector(out: &mut BumpVec<u8>, selector: &str, attr_selector: &[u8]) {
    if selector.is_empty() {
        return;
    }

    // Handle at-rules that don't have selectors
    if selector.starts_with('@') {
        out.extend_from_slice(selector.as_bytes());
        return;
    }

    // Handle multiple selectors separated by comma
    let mut first = true;
    for part in selector.split(',') {
        if !first {
            out.extend_from_slice(b", ");
        }
        first = false;
        scope_single_selector(out, part.trim(), attr_selector);
    }
}

/// Add scope attribute to a single selector
fn scope_single_selector(out: &mut BumpVec<u8>, selector: &str, attr_selector: &[u8]) {
    if selector.is_empty() {
        return;
    }

    // Handle :deep(), :slotted(), :global()
    if let Some(pos) = selector.find(":deep(") {
        transform_deep(out, selector, pos, attr_selector);
        return;
    }

    if let Some(pos) = selector.find(":slotted(") {
        transform_slotted(out, selector, pos, attr_selector);
        return;
    }

    if let Some(pos) = selector.find(":global(") {
        transform_global(out, selector, pos);
        return;
    }

    // Find the last simple selector to append the attribute
    let parts: Vec<&str> = selector.split_whitespace().collect();
    if parts.is_empty() {
        out.extend_from_slice(selector.as_bytes());
        return;
    }

    // Add scope to the last part
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push(b' ');
        }

        if i == parts.len() - 1 {
            // Last part - add scope
            add_scope_to_element(out, part, attr_selector);
        } else {
            out.extend_from_slice(part.as_bytes());
        }
    }
}

/// Add scope attribute to an element selector
pub(super) fn add_scope_to_element(out: &mut BumpVec<u8>, selector: &str, attr_selector: &[u8]) {
    let bytes = selector.as_bytes();

    // Handle pseudo-elements (::before, ::after, etc.)
    if let Some(pseudo_pos) = find_bytes(bytes, b"::") {
        out.extend_from_slice(&bytes[..pseudo_pos]);
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(&bytes[pseudo_pos..]);
        return;
    }

    // Handle pseudo-classes (:hover, :focus, etc.)
    if let Some(pseudo_pos) = rfind_byte(bytes, b':') {
        if pseudo_pos > 0 && bytes[pseudo_pos - 1] != b'\\' {
            out.extend_from_slice(&bytes[..pseudo_pos]);
            out.extend_from_slice(attr_selector);
            out.extend_from_slice(&bytes[pseudo_pos..]);
            return;
        }
    }

    out.extend_from_slice(bytes);
    out.extend_from_slice(attr_selector);
}

/// Transform :deep() to descendant selector
pub(super) fn transform_deep(
    out: &mut BumpVec<u8>,
    selector: &str,
    start: usize,
    attr_selector: &[u8],
) {
    let before = &selector[..start];
    let after = &selector[start + 6..];

    if let Some(end) = find_matching_paren(after) {
        let inner = &after[..end];
        let rest = &after[end + 1..];

        if before.is_empty() {
            out.extend_from_slice(attr_selector);
        } else {
            out.extend_from_slice(before.trim().as_bytes());
            out.extend_from_slice(attr_selector);
        }
        out.push(b' ');
        out.extend_from_slice(inner.as_bytes());
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

/// Transform :slotted() for slot content
pub(super) fn transform_slotted(
    out: &mut BumpVec<u8>,
    selector: &str,
    start: usize,
    attr_selector: &[u8],
) {
    let after = &selector[start + 9..];

    if let Some(end) = find_matching_paren(after) {
        let inner = &after.as_bytes()[..end];
        let rest = &after.as_bytes()[end + 1..];

        out.extend_from_slice(inner);
        // Convert [data-v-xxx] to [data-v-xxx-s] for slotted styles
        if attr_selector.last() == Some(&b']') {
            out.extend_from_slice(&attr_selector[..attr_selector.len() - 1]);
            out.extend_from_slice(b"-s]");
        } else {
            out.extend_from_slice(attr_selector);
            out.extend_from_slice(b"-s");
        }
        out.extend_from_slice(rest);
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

/// Transform :global() to unscoped
pub(super) fn transform_global(out: &mut BumpVec<u8>, selector: &str, start: usize) {
    let before = &selector[..start];
    let after = &selector[start + 8..];

    if let Some(end) = find_matching_paren(after) {
        let inner = &after[..end];
        let rest = &after[end + 1..];

        out.extend_from_slice(before.as_bytes());
        out.extend_from_slice(inner.as_bytes());
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}
