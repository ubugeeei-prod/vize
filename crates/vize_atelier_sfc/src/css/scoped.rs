//! Scoped CSS transformation.
//!
//! Applies Vue-style scoped CSS by adding attribute selectors (e.g., `[data-v-xxx]`)
//! to CSS selectors. Handles special pseudo-selectors: `:deep()`, `:slotted()`, `:global()`.

use vize_carton::{Bump, BumpVec};

use super::transform::find_matching_paren;

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
            if c == '*'
                && let Some(&(_, '/')) = chars.peek()
            {
                chars.next();
                in_comment = false;
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
                if let Some(stmt_str) = css.get(last_selector_end..=i).map(str::trim) {
                    output.extend_from_slice(stmt_str.as_bytes());
                }
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
                    if let Some(at_rule_str) = css.get(last_selector_end..i).map(str::trim) {
                        output.extend_from_slice(at_rule_str.as_bytes());
                    }
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
                    if let Some(kf_str) = css.get(last_selector_end..i).map(str::trim) {
                        output.extend_from_slice(kf_str.as_bytes());
                    }
                    output.push(b'{');
                    in_selector = false;
                    last_selector_end = i + 1;
                } else if in_selector
                    && (brace_depth == 1 || (at_rule_depth > 0 && brace_depth > at_rule_depth))
                {
                    // End of selector, apply scope
                    if let Some(selector_str) = css.get(last_selector_end..i) {
                        scope_selector_with_leading_comments(
                            &mut output,
                            selector_str,
                            attr_selector,
                        );
                    }
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

    // SAFETY: `output` is built by copying selector/content ranges from the
    // original UTF-8 `css` string and injecting ASCII-only scope attributes and
    // punctuation. Ranges are advanced at `char_indices` boundaries or ASCII
    // delimiter positions, so copied slices cannot split a code point. The arena
    // copy owns the bytes for the returned lifetime, and skipping revalidation
    // keeps scoped-style rewriting linear with minimal overhead.
    unsafe { std::str::from_utf8_unchecked(bump.alloc_slice_copy(&output)) }
}

/// Add scope to selector text while preserving leading CSS comments verbatim.
fn scope_selector_with_leading_comments(
    out: &mut BumpVec<u8>,
    selector: &str,
    attr_selector: &[u8],
) {
    let Some(prefix_end) = leading_css_comment_trivia_end(selector) else {
        scope_selector(out, selector.trim(), attr_selector);
        return;
    };

    out.extend_from_slice(&selector.as_bytes()[..prefix_end]);

    let selector_body = selector[prefix_end..].trim();
    if !selector_body.is_empty() {
        scope_selector(out, selector_body, attr_selector);
    }
}

fn leading_css_comment_trivia_end(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    let mut found_comment = false;

    loop {
        let ws_end = value[cursor..]
            .char_indices()
            .find(|(_, char)| !char.is_whitespace())
            .map_or(value.len(), |(index, _)| cursor + index);
        cursor = ws_end;

        if !value[cursor..].starts_with("/*") {
            return found_comment.then_some(cursor);
        }

        found_comment = true;
        let Some(end) = value[cursor + 2..].find("*/") else {
            return Some(value.len());
        };
        cursor += 2 + end + 2;
    }
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

    // Handle multiple selectors separated by top-level commas. Commas inside
    // functional pseudo-class arguments belong to the same selector.
    let mut first = true;
    for part in split_top_level_commas(selector) {
        if !first {
            out.extend_from_slice(b", ");
        }
        first = false;
        scope_single_selector(out, part.trim(), attr_selector);
    }
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut last = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&s[last..i]);
                last = i + 1;
            }
            _ => {}
        }
        i += 1;
    }

    out.push(&s[last..]);
    out
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

    // Find the last top-level compound selector to append the attribute.
    let parts: Vec<&str> = split_top_level_whitespace(selector);
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

fn split_top_level_whitespace(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut start: Option<usize> = None;
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];
        match byte {
            b'(' | b'[' => {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            }
            b')' | b']' => {
                depth -= 1;
            }
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                if let Some(start_pos) = start.take() {
                    out.push(&s[start_pos..i]);
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
        i += 1;
    }

    if let Some(start_pos) = start {
        out.push(&s[start_pos..]);
    }

    out
}

/// Add scope attribute to an element selector
pub(super) fn add_scope_to_element(out: &mut BumpVec<u8>, selector: &str, attr_selector: &[u8]) {
    // Find the first top-level pseudo-element or pseudo-class so the scope
    // attribute lands on the compound selector, not inside a functional
    // pseudo-class argument.
    if let Some(pseudo_pos) = find_top_level_pseudo(selector)
        && !selector[..pseudo_pos].ends_with('\\')
    {
        out.extend_from_slice(&selector.as_bytes()[..pseudo_pos]);
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(&selector.as_bytes()[pseudo_pos..]);
        return;
    }

    out.extend_from_slice(selector.as_bytes());
    out.extend_from_slice(attr_selector);
}

fn find_top_level_pseudo(selector: &str) -> Option<usize> {
    let bytes = selector.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }

    None
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

        push_deep_scope_prefix(out, before, attr_selector);
        out.push(b' ');
        out.extend_from_slice(inner.as_bytes());
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

fn push_deep_scope_prefix(out: &mut BumpVec<u8>, before: &str, attr_selector: &[u8]) {
    let before = before.trim_end();
    if before.is_empty() {
        out.extend_from_slice(attr_selector);
        return;
    }

    let Some(combinator_start) = trailing_combinator_start(before) else {
        scope_single_selector(out, before.trim(), attr_selector);
        return;
    };

    let target_end = before[..combinator_start].trim_end().len();
    if target_end == 0 {
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(&before.as_bytes()[combinator_start..]);
        return;
    }

    scope_single_selector(out, &before[..target_end], attr_selector);
    out.extend_from_slice(&before.as_bytes()[target_end..]);
}

fn trailing_combinator_start(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    match bytes.last().copied()? {
        b'>' | b'+' | b'~' => Some(bytes.len() - 1),
        b'|' if bytes.len() >= 2 && bytes[bytes.len() - 2] == b'|' => Some(bytes.len() - 2),
        _ => None,
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
