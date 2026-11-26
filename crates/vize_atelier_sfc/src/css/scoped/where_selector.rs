use vize_carton::StringBuilder;

use super::super::scoped_selector::{find_top_level_pseudo, leading_universal_selector_end};

/// Add scope attribute to an element selector
pub(crate) fn add_scope_to_element(
    out: &mut StringBuilder<'_>,
    selector: &str,
    attr_selector: &str,
) {
    let selector = leading_universal_selector_end(selector)
        .and_then(|end| selector.get(end..))
        .unwrap_or(selector);

    if let Some(scoped) = crate::style::scope_bare_where(selector, attr_selector) {
        out.push_str(&scoped);
        return;
    }

    // Find the first top-level pseudo-element or pseudo-class so the scope
    // attribute lands on the compound selector, not inside a functional
    // pseudo-class argument.
    if let Some((before, after)) =
        find_top_level_pseudo(selector).and_then(|pos| selector.split_at_checked(pos))
        && !before.ends_with('\\')
    {
        out.push_str(before);
        out.push_str(attr_selector);
        out.push_str(after);
        return;
    }

    out.push_str(selector);
    out.push_str(attr_selector);
}
