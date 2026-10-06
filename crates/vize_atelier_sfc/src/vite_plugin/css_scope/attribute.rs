use super::{
    find_last_compound_start, find_matching_paren, find_pseudo_function_from,
    find_scope_insert_position, push_scope_attr, split_selector_list,
};
use crate::css::scoped_selector::split_before_trailing_universal_or_pseudo;
use vize_carton::String;

pub(super) fn add_scope_to_selector_end(selector: &str, scope_id: &str) -> String {
    if let Some(scoped) = scope_functional_anchor(selector, scope_id) {
        return scoped;
    }
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

/// Vue recursively scopes a leading :is/:where only while it remains the
/// selector's scope anchor. A later class, type or attribute owns that role.
pub(super) fn scope_functional_anchor(selector: &str, scope_id: &str) -> Option<String> {
    let marker = if selector.starts_with(":where(") {
        ":where("
    } else if selector.starts_with(":is(") {
        ":is("
    } else {
        return None;
    };
    let function = find_pseudo_function_from(selector, marker, 0)?;
    if !only_pseudos_and_combinators(&selector[function.end..]) {
        return None;
    }
    let mut output = String::with_capacity(selector.len() + scope_id.len() + 2);
    output.push_str(&selector[..function.inner_start]);
    for (index, branch) in split_selector_list(function.parts(selector).1)
        .iter()
        .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        output.push_str(add_scope_to_selector_end(branch.trim(), scope_id).as_str());
    }
    output.push_str(&selector[function.inner_end..]);
    Some(output)
}

fn only_pseudos_and_combinators(mut tail: &str) -> bool {
    while !tail.is_empty() {
        tail = tail.trim_start_matches([' ', '\t', '\n', '\r', '\u{c}', '>', '+', '~', '|', '*']);
        if tail.is_empty() {
            return true;
        }
        if let Some(comment) = tail.strip_prefix("/*") {
            let Some(end) = comment.find("*/") else {
                return false;
            };
            tail = &comment[end + 2..];
            continue;
        }
        let Some(pseudo) = tail.strip_prefix(':') else {
            return false;
        };
        let end = pseudo
            .find(|c: char| !(c.is_alphanumeric() || matches!(c, '-' | '_' | ':')))
            .unwrap_or(pseudo.len());
        tail = &pseudo[end..];
        if tail.starts_with('(') {
            let Some(end) = find_matching_paren(tail, 0) else {
                return false;
            };
            tail = &tail[end + 1..];
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::super::scope_css_for_pipeline;

    #[test]
    fn original_mixed_and_functional_rules_have_complete_pipeline_output() {
        for (source, expected) in [
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/mixed.css"
                ),
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/mixed.pipeline.css"
                ),
            ),
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/functional.css"
                ),
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/functional.pipeline.css"
                ),
            ),
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/is-list.css"
                ),
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/is-list.pipeline.css"
                ),
            ),
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/later-anchor.css"
                ),
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/later-anchor.pipeline.css"
                ),
            ),
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/class-anchor.css"
                ),
                include_str!(
                    "../../../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/class-anchor.pipeline.css"
                ),
            ),
        ] {
            assert_eq!(
                scope_css_for_pipeline(source, "data-v-1").as_str(),
                expected
            );
        }
    }
}
