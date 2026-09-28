use vize_carton::Vec as ArenaVec;

use super::super::scoped_selector::{find_top_level_pseudo, leading_universal_selector_end};

/// Add scope attribute to an element selector
pub(crate) fn add_scope_to_element(out: &mut ArenaVec<u8>, selector: &str, attr_selector: &[u8]) {
    let selector = leading_universal_selector_end(selector)
        .and_then(|end| selector.get(end..))
        .unwrap_or(selector);

    if let Ok(attr) = std::str::from_utf8(attr_selector)
        && let Some(scoped) = crate::style::scope_bare_where(selector, attr)
    {
        out.extend_from_slice(scoped.as_bytes());
        return;
    }

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
