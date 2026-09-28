use super::{find_top_level_pseudo, split_parenthesized_argument, split_top_level_commas};
use vize_carton::String;

/// Add scope attribute to an element selector
pub(super) fn add_scope_to_element(selector: &str, attr_selector: &str) -> String {
    if let Some(scoped) = scope_bare_where(selector, attr_selector) {
        return scoped;
    }
    // Find the FIRST top-level pseudo-element or pseudo-class so the scope
    // attribute lands on the compound selector, not inside a functional
    // pseudo-class argument (e.g. `.x:not(:checked)` → `.x[attr]:not(:checked)`,
    // not `.x:not(:[attr]checked)`). Skip colons inside parentheses. (#971)
    if let Some(pseudo_pos) = find_top_level_pseudo(selector)
        && let Some((before, after)) = selector.split_at_checked(pseudo_pos)
    {
        // Avoid splitting at a pseudo that is part of an escape sequence
        // (`\:`), which is rare but valid in CSS.
        if !before.ends_with('\\') {
            let mut result =
                String::with_capacity(before.len() + attr_selector.len() + after.len());
            result.push_str(before);
            result.push_str(attr_selector);
            result.push_str(after);
            return result;
        }
    }

    let mut result = String::with_capacity(selector.len() + attr_selector.len());
    result.push_str(selector);
    result.push_str(attr_selector);
    result
}

/// Keep the injected attribute inside `:where()` so specificity remains zero.
pub(crate) fn scope_bare_where(selector: &str, attr_selector: &str) -> Option<String> {
    let after_open = selector.strip_prefix(":where(")?;
    let (inner, trailing) = split_parenthesized_argument(after_open)?;
    if inner.trim().is_empty() {
        return None;
    }

    let mut scoped = String::with_capacity(selector.len() + attr_selector.len());
    scoped.push_str(":where(");
    for (index, branch) in split_top_level_commas(inner).into_iter().enumerate() {
        if index > 0 {
            scoped.push_str(", ");
        }
        scoped.push_str(&add_scope_to_element(branch.trim(), attr_selector));
    }
    scoped.push(')');
    scoped.push_str(trailing);
    Some(scoped)
}
