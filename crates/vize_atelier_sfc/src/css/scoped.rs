//! Scoped CSS transformation.
//!
//! Applies Vue-style scoped CSS by adding attribute selectors (e.g., `[data-v-xxx]`)
//! to CSS selectors. Handles special pseudo-selectors: `:deep()`, `:slotted()`, `:global()`.

mod slotted;

use vize_carton::{Allocator, Vec as ArenaVec};

pub(super) use slotted::transform_slotted;

use super::scoped_selector::{
    find_top_level_pseudo, leading_universal_selector_end,
    split_before_trailing_universal_or_pseudo,
};
use super::transform::find_matching_paren;

/// Apply scoped CSS transformation
pub(crate) fn apply_scoped_css<'a>(bump: &'a Allocator, css: &str, scope_id: &str) -> &'a str {
    let css_bytes = css.as_bytes();

    // Build attr_selector: [scope_id]
    let mut attr_selector = ArenaVec::with_capacity_in(scope_id.len() + 2, &bump);
    attr_selector.push(b'[');
    attr_selector.extend_from_slice(scope_id.as_bytes());
    attr_selector.push(b']');
    let attr_selector = bump.alloc_slice_copy(&attr_selector);

    let mut output = ArenaVec::with_capacity_in(css_bytes.len() * 2, &bump);
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
                let prev_byte = i.checked_sub(1).and_then(|p| css_bytes.get(p)).copied();
                if prev_byte != Some(b'\\') {
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
                let remaining = css.get(i + 1..).unwrap_or_default();
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
    if in_selector && let Some(rest) = css_bytes.get(last_selector_end..) {
        output.extend_from_slice(rest);
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
    out: &mut ArenaVec<u8>,
    selector: &str,
    attr_selector: &[u8],
) {
    let Some(prefix_end) = leading_css_comment_trivia_end(selector) else {
        scope_selector(out, selector.trim(), attr_selector);
        return;
    };

    let (prefix, selector_body) = selector
        .split_at_checked(prefix_end)
        .unwrap_or((selector, ""));
    out.extend_from_slice(prefix.as_bytes());

    let selector_body = selector_body.trim();
    if !selector_body.is_empty() {
        scope_selector(out, selector_body, attr_selector);
    }
}

fn leading_css_comment_trivia_end(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    let mut found_comment = false;

    loop {
        let ws_end = value
            .get(cursor..)?
            .char_indices()
            .find(|(_, char)| !char.is_whitespace())
            .map_or(value.len(), |(index, _)| cursor + index);
        cursor = ws_end;

        let Some(comment) = value.get(cursor..).and_then(|tail| tail.strip_prefix("/*")) else {
            return found_comment.then_some(cursor);
        };

        found_comment = true;
        let Some(end) = comment.find("*/") else {
            return Some(value.len());
        };
        cursor += 2 + end + 2;
    }
}

/// Add scope attribute to a selector
fn scope_selector(out: &mut ArenaVec<u8>, selector: &str, attr_selector: &[u8]) {
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
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut last = 0;

    for (i, byte) in s.bytes().enumerate() {
        match byte {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                out.push(s.get(last..i).unwrap_or_default());
                last = i + 1;
            }
            _ => {}
        }
    }

    out.push(s.get(last..).unwrap_or_default());
    out
}

/// Add scope attribute to a single selector
fn scope_single_selector(out: &mut ArenaVec<u8>, selector: &str, attr_selector: &[u8]) {
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

    if let Some((prefix, boundary, suffix)) = split_before_trailing_universal_or_pseudo(selector) {
        scope_single_selector(out, prefix.trim_end(), attr_selector);
        out.extend_from_slice(boundary.as_bytes());
        out.extend_from_slice(suffix.trim_start().as_bytes());
        return;
    }

    // Find the last top-level compound selector to append the attribute.
    let parts: Vec<&str> = split_top_level_whitespace(selector);
    // Add scope to the last part
    let Some((last, leading)) = parts.split_last() else {
        out.extend_from_slice(selector.as_bytes());
        return;
    };
    for part in leading {
        out.extend_from_slice(part.as_bytes());
        out.push(b' ');
    }
    add_scope_to_element(out, last, attr_selector);
}

fn split_top_level_whitespace(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut start: Option<usize> = None;

    for (i, byte) in s.bytes().enumerate() {
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
                if let Some(part) = start.take().and_then(|start_pos| s.get(start_pos..i)) {
                    out.push(part);
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
    }

    if let Some(part) = start.and_then(|start_pos| s.get(start_pos..)) {
        out.push(part);
    }

    out
}

/// Add scope attribute to an element selector
pub(super) fn add_scope_to_element(out: &mut ArenaVec<u8>, selector: &str, attr_selector: &[u8]) {
    let selector = leading_universal_selector_end(selector)
        .and_then(|end| selector.get(end..))
        .unwrap_or(selector);

    // Find the first top-level pseudo-element or pseudo-class so the scope
    // attribute lands on the compound selector, not inside a functional
    // pseudo-class argument.
    if let Some((before, after)) =
        find_top_level_pseudo(selector).and_then(|pos| selector.split_at_checked(pos))
        && !before.ends_with('\\')
    {
        out.extend_from_slice(before.as_bytes());
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(after.as_bytes());
        return;
    }

    out.extend_from_slice(selector.as_bytes());
    out.extend_from_slice(attr_selector);
}

/// Transform :deep() to descendant selector
pub(super) fn transform_deep(
    out: &mut ArenaVec<u8>,
    selector: &str,
    start: usize,
    attr_selector: &[u8],
) {
    if let Some((before, inner, rest)) = split_pseudo_function(selector, start, ":deep(") {
        push_deep_scope_prefix(out, before, attr_selector);
        out.push(b' ');
        out.extend_from_slice(inner.as_bytes());
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

fn push_deep_scope_prefix(out: &mut ArenaVec<u8>, before: &str, attr_selector: &[u8]) {
    let before = before.trim_end();
    if before.is_empty() {
        out.extend_from_slice(attr_selector);
        return;
    }

    let Some(combinator_start) = trailing_combinator_start(before) else {
        scope_single_selector(out, before.trim(), attr_selector);
        return;
    };

    let Some((target, combinator)) = before.split_at_checked(combinator_start) else {
        scope_single_selector(out, before.trim(), attr_selector);
        return;
    };
    let target = target.trim_end();
    if target.is_empty() {
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(combinator.as_bytes());
        return;
    }

    scope_single_selector(out, target, attr_selector);
    out.extend_from_slice(before.get(target.len()..).unwrap_or_default().as_bytes());
}

fn trailing_combinator_start(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    match bytes {
        [.., b'|', b'|'] => Some(bytes.len() - 2),
        [.., b'>' | b'+' | b'~'] => Some(bytes.len() - 1),
        _ => None,
    }
}

/// Transform :global() to unscoped
pub(super) fn transform_global(out: &mut ArenaVec<u8>, selector: &str, start: usize) {
    if let Some((before, inner, rest)) = split_pseudo_function(selector, start, ":global(") {
        out.extend_from_slice(before.as_bytes());
        out.extend_from_slice(inner.as_bytes());
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

/// Split `selector` around the pseudo function `marker` found at `start`:
/// the text before it, its parenthesised argument, and the text after the
/// matching `)`. `None` when the argument is unterminated.
pub(super) fn split_pseudo_function<'s>(
    selector: &'s str,
    start: usize,
    marker: &str,
) -> Option<(&'s str, &'s str, &'s str)> {
    let (before, pseudo) = selector.split_at_checked(start)?;
    let after = pseudo.get(marker.len()..)?;
    let end = find_matching_paren(after)?;
    let (inner, rest) = after.split_at_checked(end)?;
    Some((before, inner, rest.get(1..)?))
}
