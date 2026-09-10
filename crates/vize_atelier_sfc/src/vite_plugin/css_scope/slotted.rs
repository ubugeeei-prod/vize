use super::{
    PseudoFunction, add_scope_before_trailing_combinator, find_scope_insert_position,
    trailing_combinator_start,
};
use vize_carton::String;

pub(super) fn scope_slotted_selector(
    body: &str,
    slotted: &PseudoFunction,
    scope_id: &str,
) -> String {
    let before = body[..slotted.start].trim_end();
    let inner = &body[slotted.inner_start..slotted.inner_end];
    let after = &body[slotted.end..];
    let mut scoped = String::with_capacity(body.len() + scope_id.len() * 2 + 4);

    if !before.is_empty() {
        let scoped_before = add_scope_before_trailing_combinator(before, scope_id);
        scoped.push_str(scoped_before.as_str());
        if !selector_ends_with_combinator(before) {
            scoped.push(' ');
        }
    }

    push_slotted_target(&mut scoped, inner, scope_id);
    scoped.push_str(after);
    scoped
}

fn selector_ends_with_combinator(selector: &str) -> bool {
    trailing_combinator_start(selector.trim_end()).is_some()
}

fn push_slotted_target(output: &mut String, inner: &str, scope_id: &str) {
    let target = strip_slotted_leading_universal(inner.trim());
    let insert_at = find_scope_insert_position(target);
    output.push_str(&target[..insert_at]);
    push_slotted_scope_attr(output, scope_id);
    output.push_str(&target[insert_at..]);
}

fn strip_slotted_leading_universal(selector: &str) -> &str {
    let Some(end) = slotted_leading_universal_end(selector) else {
        return selector;
    };

    &selector[end..]
}

fn slotted_leading_universal_end(selector: &str) -> Option<usize> {
    let bytes = selector.as_bytes();
    if bytes.first() == Some(&b'*') {
        if bytes.get(1) == Some(&b'|') && bytes.get(2) == Some(&b'*') {
            return Some(3);
        }
        return Some(1);
    }

    if bytes.first() == Some(&b'|') && bytes.get(1) == Some(&b'*') {
        return Some(2);
    }

    None
}

fn push_slotted_scope_attr(output: &mut String, scope_id: &str) {
    output.push('[');
    output.push_str(scope_id);
    output.push_str("-s]");
}

#[cfg(test)]
mod tests {
    use super::super::scope_css_for_pipeline;

    #[test]
    fn scopes_slotted_selectors_with_slotted_scope_id() {
        assert_eq!(
            scope_css_for_pipeline(":slotted(.foo) { color: red; }", "data-v-x").as_str(),
            ".foo[data-v-x-s]{color: red;}"
        );
        assert_eq!(
            scope_css_for_pipeline("::v-slotted(.foo) { color: red; }", "data-v-x").as_str(),
            ".foo[data-v-x-s]{color: red;}"
        );
        assert_eq!(
            scope_css_for_pipeline(":slotted(.foo):hover { color: red; }", "data-v-x").as_str(),
            ".foo[data-v-x-s]:hover{color: red;}"
        );
    }

    #[test]
    fn scopes_nested_slotted_selector_with_slotted_scope_id() {
        assert_eq!(
            scope_css_for_pipeline(".host { :slotted(.foo) { color: red; } }", "data-v-x").as_str(),
            " .host[data-v-x] .foo[data-v-x-s]{color: red;}"
        );
    }

    #[test]
    fn scopes_nested_slotted_selectors_after_parent_combinators() {
        assert_eq!(
            scope_css_for_pipeline(
                ".card { & > :slotted(*) { flex: 1; } & > :slotted(.title), & > :slotted(.subtitle) { line-height: 1.2; } & > :slotted(*):not(:last-child) { margin-bottom: 2px; } }",
                "data-v-x"
            )
            .as_str(),
            " .card[data-v-x] >[data-v-x-s]{flex: 1;} .card[data-v-x] >.title[data-v-x-s],.card[data-v-x] >.subtitle[data-v-x-s]{line-height: 1.2;} .card[data-v-x] >[data-v-x-s]:not(:last-child){margin-bottom: 2px;}"
        );
        assert_eq!(
            scope_css_for_pipeline(
                ".card { & || :slotted(*) { display: table-cell; } }",
                "data-v-x"
            )
            .as_str(),
            " .card[data-v-x] ||[data-v-x-s]{display: table-cell;}"
        );
    }
}
