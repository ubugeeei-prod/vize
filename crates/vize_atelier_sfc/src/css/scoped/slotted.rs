use super::split_pseudo_function;
use crate::css::scoped_selector::{find_top_level_pseudo, leading_universal_selector_end};
use vize_carton::Vec as ArenaVec;

/// Transform :slotted() for slot content.
pub(in crate::css) fn transform_slotted(
    out: &mut ArenaVec<u8>,
    selector: &str,
    start: usize,
    attr_selector: &[u8],
) {
    if let Some((before, inner, rest)) = split_pseudo_function(selector, start, ":slotted(") {
        out.extend_from_slice(before.as_bytes());
        push_slotted_target(out, inner, attr_selector);
        out.extend_from_slice(rest.as_bytes());
    } else {
        out.extend_from_slice(selector.as_bytes());
    }
}

fn push_slotted_target(out: &mut ArenaVec<u8>, inner: &str, attr_selector: &[u8]) {
    let inner = inner.trim();
    let inner = leading_universal_selector_end(inner)
        .and_then(|end| inner.get(end..))
        .unwrap_or(inner);

    if let Some((before, after)) =
        find_top_level_pseudo(inner).and_then(|pos| inner.split_at_checked(pos))
        && !before.ends_with('\\')
    {
        out.extend_from_slice(before.as_bytes());
        push_slotted_attr(out, attr_selector);
        out.extend_from_slice(after.as_bytes());
        return;
    }

    out.extend_from_slice(inner.as_bytes());
    push_slotted_attr(out, attr_selector);
}

fn push_slotted_attr(out: &mut ArenaVec<u8>, attr_selector: &[u8]) {
    if let Some(open) = attr_selector.strip_suffix(b"]") {
        out.extend_from_slice(open);
        out.extend_from_slice(b"-s]");
    } else {
        out.extend_from_slice(attr_selector);
        out.extend_from_slice(b"-s");
    }
}
