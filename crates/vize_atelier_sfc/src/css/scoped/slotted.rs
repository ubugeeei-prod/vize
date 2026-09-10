use super::{push_deep_scope_prefix, trailing_combinator_start};
use crate::css::scoped_selector::{find_top_level_pseudo, leading_universal_selector_end};
use crate::css::transform::find_matching_paren;
use vize_carton::Vec as ArenaVec;

/// Transform :slotted() for slot content.
pub(in crate::css) fn transform_slotted(
    out: &mut ArenaVec<u8>,
    selector: &str,
    start: usize,
    attr_selector: &[u8],
) {
    let before = &selector[..start];
    let after = &selector[start + 9..];

    if let Some(end) = find_matching_paren(after) {
        let inner = &after[..end];
        let rest = &after[end + 1..];

        push_slotted_scope_prefix(out, before, attr_selector);
        push_slotted_target(out, inner, attr_selector);
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

fn push_slotted_scope_prefix(out: &mut ArenaVec<u8>, before: &str, attr_selector: &[u8]) {
    let before = before.trim_end();
    if before.is_empty() {
        return;
    }

    push_deep_scope_prefix(out, before, attr_selector);
    if trailing_combinator_start(before).is_none() {
        out.push(b' ');
    }
}

fn push_slotted_target(out: &mut ArenaVec<u8>, inner: &str, attr_selector: &[u8]) {
    let inner = inner.trim();
    let inner = if let Some(end) = leading_universal_selector_end(inner) {
        &inner[end..]
    } else {
        inner
    };

    if let Some(pseudo_pos) = find_top_level_pseudo(inner)
        && !inner[..pseudo_pos].ends_with('\\')
    {
        out.extend_from_slice(&inner.as_bytes()[..pseudo_pos]);
        push_slotted_attr(out, attr_selector);
        out.extend_from_slice(&inner.as_bytes()[pseudo_pos..]);
        return;
    }

    out.extend_from_slice(inner.as_bytes());
    push_slotted_attr(out, attr_selector);
}

fn push_slotted_attr(out: &mut ArenaVec<u8>, attr_selector: &[u8]) {
    if attr_selector.last() == Some(&b']') {
        out.extend_from_slice(&attr_selector[..attr_selector.len() - 1]);
        out.extend_from_slice(b"-s]");
    } else {
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(b"-s");
    }
}
