use super::split_pseudo_function;
use crate::css::scoped_selector::{find_top_level_pseudo, leading_universal_selector_end};
use vize_carton::StringBuilder;

/// Transform :slotted() for slot content.
pub(in crate::css) fn transform_slotted(
    out: &mut StringBuilder<'_>,
    selector: &str,
    start: usize,
    attr_selector: &str,
) {
    if let Some((before, inner, rest)) = split_pseudo_function(selector, start, ":slotted(") {
        out.push_str(before);
        push_slotted_target(out, inner, attr_selector);
        out.push_str(rest);
    } else {
        out.push_str(selector);
    }
}

fn push_slotted_target(out: &mut StringBuilder<'_>, inner: &str, attr_selector: &str) {
    let inner = inner.trim();
    let inner = leading_universal_selector_end(inner)
        .and_then(|end| inner.get(end..))
        .unwrap_or(inner);

    if let Some((before, after)) =
        find_top_level_pseudo(inner).and_then(|pos| inner.split_at_checked(pos))
        && !before.ends_with('\\')
    {
        out.push_str(before);
        push_slotted_attr(out, attr_selector);
        out.push_str(after);
        return;
    }

    out.push_str(inner);
    push_slotted_attr(out, attr_selector);
}

fn push_slotted_attr(out: &mut StringBuilder<'_>, attr_selector: &str) {
    if let Some(open) = attr_selector.strip_suffix(']') {
        out.push_str(open);
        out.push_str("-s]");
    } else {
        out.push_str(attr_selector);
        out.push_str("-s");
    }
}
