//! Keep selector-list nesting intact: `&` inherits the maximum specificity of
//! its entire parent list, which cannot be represented by a Cartesian product.
use super::{
    find_matching_brace, find_next_top_level_brace, find_pseudo_function_any,
    find_rule_header_start, scope_attr, scope_selector_list, should_recurse_at_rule,
    split_leading_trivia, split_selector_list, unwrap_deep_selectors,
};
use vize_carton::{SmallVec, String};

enum Node<'a> {
    Declarations(&'a str),
    Rule { header: &'a str, body: &'a str },
}

fn nodes(css: &str) -> SmallVec<[Node<'_>; 4]> {
    let mut result = SmallVec::new();
    let mut cursor = 0;
    while let Some(brace) = find_next_top_level_brace(css, cursor) {
        let Some(end) = find_matching_brace(css, brace) else {
            break;
        };
        let header_start = find_rule_header_start(css, cursor, brace);
        let Some(pending) = css.get(cursor..header_start) else {
            break;
        };
        let Some(header) = css.get(header_start..brace) else {
            break;
        };
        let Some(body) = css.get(brace + 1..end) else {
            break;
        };
        result.push(Node::Declarations(pending));
        result.push(Node::Rule { header, body });
        cursor = end + 1;
    }
    result.push(Node::Declarations(css.get(cursor..).unwrap_or_default()));
    result
}

fn is_style(header: &str) -> bool {
    !split_leading_trivia(header).1.starts_with('@')
}

fn has_style_descendant(css: &str) -> bool {
    nodes(css).iter().any(|node| match node {
        Node::Rule { header, body } => {
            is_style(header)
                || (should_recurse_at_rule(split_leading_trivia(header).1)
                    && has_style_descendant(body))
        }
        Node::Declarations(_) => false,
    })
}

pub(super) fn requires_preservation(selector: &str, body: &str) -> bool {
    if split_selector_list(selector).len() > 1 && has_style_descendant(body) {
        return true;
    }
    nodes(body).iter().any(|node| match node {
        Node::Rule { header, body } => {
            let statement = split_leading_trivia(header).1;
            if is_style(header) {
                requires_preservation(statement, body)
            } else {
                should_recurse_at_rule(statement) && requires_preservation("", body)
            }
        }
        Node::Declarations(_) => false,
    })
}

fn has_deep(selector: &str) -> bool {
    find_pseudo_function_any(selector, &[":deep(", "::v-deep(", "::deep("]).is_some()
}

fn without_scope(selector_list: &str) -> String {
    let mut result = String::default();
    for (index, selector) in split_selector_list(selector_list).iter().enumerate() {
        if index > 0 {
            result.push(',');
        }
        let global = find_pseudo_function_any(selector, &[":global(", "::v-global("]);
        if let Some(global) = global {
            result.push_str(global.parts(selector).1);
        } else {
            result.push_str(unwrap_deep_selectors(selector.trim()).as_str());
        }
    }
    result
}

pub(super) fn scope_rule(
    selector: &str,
    body: &str,
    scope_id: &str,
    inherited_deep: bool,
) -> String {
    let children = nodes(body);
    let has_nested_rule = children
        .iter()
        .any(|node| matches!(node, Node::Rule { header, .. } if is_style(header)));
    let own_deep = has_deep(selector);
    let inject = !inherited_deep && (!has_nested_rule || own_deep);
    let mut result = if inject {
        scope_selector_list(selector.trim(), scope_id)
    } else {
        without_scope(selector)
    };
    result.push('{');
    result.push_str(
        render_body(
            body,
            scope_id,
            inherited_deep || own_deep,
            has_nested_rule && !own_deep,
        )
        .as_str(),
    );
    result.push('}');
    result
}

fn render_body(css: &str, scope_id: &str, deep: bool, wrap_declarations: bool) -> String {
    let children = nodes(css);
    let mut result = String::default();
    if wrap_declarations {
        let mut declarations = String::default();
        for node in &children {
            if let Node::Declarations(value) = node {
                declarations.push_str(value.trim());
            }
        }
        if !declarations.is_empty() {
            result.push('&');
            if !deep {
                result.push_str(scope_attr(scope_id).as_str());
            }
            result.push('{');
            result.push_str(declarations.as_str());
            result.push('}');
        }
    }
    for node in children {
        match node {
            Node::Declarations(value) => {
                if !wrap_declarations {
                    result.push_str(value);
                }
            }
            Node::Rule { header, body } => {
                let (leading, statement) = split_leading_trivia(header);
                result.push_str(leading);
                if is_style(header) {
                    result.push_str(scope_rule(statement, body, scope_id, deep).as_str());
                } else {
                    result.push_str(statement);
                    result.push('{');
                    if should_recurse_at_rule(statement) {
                        result.push_str(
                            render_body(body, scope_id, deep, wrap_declarations).as_str(),
                        );
                    } else {
                        result.push_str(body);
                    }
                    result.push('}');
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::scope_css_for_pipeline;

    #[test]
    fn nested_lists_keep_their_whole_parent_specificity_and_scope_positions() {
        for (source, expected) in [
            (
                ".x .y,.z{&:hover{color:blue}}",
                ".x .y,.z{&[data-v-x]:hover{color:blue}}",
            ),
            (
                ".outer{.a,#b{color:red;.child{color:blue}}}",
                ".outer{.a,#b{&[data-v-x]{color:red;}.child[data-v-x]{color:blue}}}",
            ),
            (".a,#b{& &{color:blue}}", ".a,#b{& &[data-v-x]{color:blue}}"),
            (
                ".a,#b{[data-symbol=\"&\"]{color:blue}}",
                ".a,#b{[data-symbol=\"&\"][data-v-x]{color:blue}}",
            ),
            (
                ".a,#b{@media (min-width:1px){.child{color:blue}}}",
                ".a[data-v-x],#b[data-v-x]{@media (min-width:1px){.child[data-v-x]{color:blue}}}",
            ),
            (
                ":global(.a),#b{.child{color:blue}}",
                ".a,#b{.child[data-v-x]{color:blue}}",
            ),
            (
                ".a :deep(.b),#c{.child{color:blue}}",
                ".a[data-v-x] .b,#c[data-v-x]{.child{color:blue}}",
            ),
            (
                ".a,#b{:slotted(.child){color:blue}}",
                ".a,#b{.child[data-v-x-s]{color:blue}}",
            ),
        ] {
            assert_eq!(
                scope_css_for_pipeline(source, "data-v-x").as_str(),
                expected,
                "{source}"
            );
        }
    }
}
