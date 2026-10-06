use super::{find_last_compound_start, find_scope_insert_position, push_scope_attr};
use crate::css::scoped_selector::split_before_trailing_universal_or_pseudo;
use vize_carton::String;

pub(super) fn add_scope_to_selector_end(selector: &str, scope_id: &str) -> String {
    if let Some((prefix, boundary, suffix)) = split_before_trailing_universal_or_pseudo(selector) {
        let mut output = add_scope_to_selector_end(prefix.trim_end(), scope_id);
        output.push_str(boundary);
        output.push_str(suffix.trim_start());
        return output;
    }

    let (before_target, target) = selector
        .split_at_checked(find_last_compound_start(selector))
        .unwrap_or(("", selector));
    let (head, tail) = target
        .split_at_checked(find_scope_insert_position(target))
        .unwrap_or((target, ""));

    let mut output = String::with_capacity(selector.len() + scope_id.len() + 2);
    output.push_str(before_target);
    output.push_str(head);
    push_scope_attr(&mut output, scope_id);
    output.push_str(tail);
    output
}
