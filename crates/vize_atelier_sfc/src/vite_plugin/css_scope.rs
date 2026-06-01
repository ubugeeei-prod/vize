use vize_carton::{SmallVec, String};

/// Scope CSS with the Vite plugin pipeline's selector model.
pub(super) fn scope_css_for_pipeline(css: &str, scope_id: &str) -> String {
    transform_css_block(css, scope_id)
}

pub(super) fn unwrap_deep_selectors(css: &str) -> String {
    unwrap_pseudo_functions(
        css,
        &["::v-deep(", "::deep(", ":deep(", "::v-global(", ":global("],
    )
}

fn transform_css_block(css: &str, scope_id: &str) -> String {
    transform_css_block_with_parents(css, scope_id, None)
}

fn transform_css_block_with_parents(
    css: &str,
    scope_id: &str,
    parent_selectors: Option<&[String]>,
) -> String {
    let mut output = String::with_capacity(css.len() + scope_id.len());
    let mut cursor = 0usize;
    let mut declarations = String::default();

    while cursor < css.len() {
        let Some(brace) = find_next_top_level_brace(css, cursor) else {
            declarations.push_str(&css[cursor..]);
            break;
        };

        let Some(end) = find_matching_brace(css, brace) else {
            declarations.push_str(&css[cursor..]);
            break;
        };

        let header_start = find_rule_header_start(css, cursor, brace);
        declarations.push_str(&css[cursor..header_start]);
        flush_declarations(
            &mut output,
            parent_selectors,
            scope_id,
            declarations.as_str(),
        );
        declarations.clear();

        let header = &css[header_start..brace];
        let body = &css[brace + 1..end];
        let (leading, statement) = split_leading_trivia(header);

        output.push_str(leading);
        if statement.trim_start().starts_with('@') {
            output.push_str(statement);
            output.push('{');
            if should_recurse_at_rule(statement) {
                output.push_str(
                    transform_css_block_with_parents(body, scope_id, parent_selectors).as_str(),
                );
            } else {
                output.push_str(body);
            }
            output.push('}');
        } else {
            let selectors = combine_selector_lists(parent_selectors, statement);
            output.push_str(
                transform_css_block_with_parents(body, scope_id, Some(&selectors)).as_str(),
            );
        }

        cursor = end + 1;
    }

    flush_declarations(
        &mut output,
        parent_selectors,
        scope_id,
        declarations.as_str(),
    );
    output
}

fn should_recurse_at_rule(statement: &str) -> bool {
    matches!(
        statement.split_whitespace().next(),
        Some("@container" | "@layer" | "@media" | "@supports")
    )
}

fn find_rule_header_start(css: &str, start: usize, brace: usize) -> usize {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut in_comment = false;
    let mut header_start = start;
    let mut iter = css[start..brace].char_indices().peekable();

    while let Some((relative, char)) = iter.next() {
        let index = start + relative;
        let next = iter.peek().map(|(_, next)| *next);

        if in_comment {
            if char == '*' && next == Some('/') {
                iter.next();
                in_comment = false;
            }
            continue;
        }

        if let Some(active_quote) = quote {
            if char == '\\' {
                iter.next();
            } else if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '/' if next == Some('*') => {
                iter.next();
                in_comment = true;
            }
            '\'' | '"' => quote = Some(char),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ';' if paren_depth == 0 && bracket_depth == 0 => {
                header_start = index + 1;
            }
            _ => {}
        }
    }

    header_start
}

fn flush_declarations(
    output: &mut String,
    parent_selectors: Option<&[String]>,
    scope_id: &str,
    declarations: &str,
) {
    let declarations = declarations.trim();
    if declarations.is_empty() {
        return;
    }

    let Some(selectors) = parent_selectors else {
        output.push_str(declarations);
        return;
    };

    let mut selector_list = String::default();
    for (index, selector) in selectors.iter().enumerate() {
        if index > 0 {
            selector_list.push(',');
        }
        selector_list.push_str(selector.as_str());
    }

    output.push_str(scope_selector_list(selector_list.as_str(), scope_id).as_str());
    output.push('{');
    output.push_str(declarations);
    output.push('}');
}

fn combine_selector_lists(
    parent_selectors: Option<&[String]>,
    selector_list: &str,
) -> SmallVec<[String; 4]> {
    let selectors = split_selector_list(selector_list);
    let mut output = SmallVec::new();

    let Some(parents) = parent_selectors else {
        for selector in selectors {
            output.push(String::from(selector.trim()));
        }
        return output;
    };

    for parent in parents {
        for selector in &selectors {
            output.push(combine_selector(parent.as_str(), selector.trim()));
        }
    }

    output
}

fn combine_selector(parent: &str, selector: &str) -> String {
    if selector.contains('&') {
        return String::from(selector.replace('&', parent).as_str());
    }

    let mut output = String::with_capacity(parent.len() + selector.len() + 1);
    output.push_str(parent);
    if selector
        .chars()
        .find(|char| !char.is_whitespace())
        .is_some_and(|char| matches!(char, '>' | '+' | '~'))
    {
        output.push_str(selector);
    } else {
        output.push(' ');
        output.push_str(selector);
    }
    output
}

fn find_next_top_level_brace(css: &str, start: usize) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut in_comment = false;
    let mut iter = css[start..].char_indices().peekable();

    while let Some((relative, char)) = iter.next() {
        let index = start + relative;
        let next = iter.peek().map(|(_, next)| *next);

        if in_comment {
            if char == '*' && next == Some('/') {
                iter.next();
                in_comment = false;
            }
            continue;
        }

        if let Some(active_quote) = quote {
            if char == '\\' {
                iter.next();
            } else if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '/' if next == Some('*') => {
                iter.next();
                in_comment = true;
            }
            '\'' | '"' => quote = Some(char),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' if paren_depth == 0 && bracket_depth == 0 => return Some(index),
            _ => {}
        }
    }

    None
}

fn find_matching_brace(css: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut in_comment = false;
    let mut iter = css[start..].char_indices().peekable();

    while let Some((relative, char)) = iter.next() {
        let index = start + relative;
        let next = iter.peek().map(|(_, next)| *next);

        if in_comment {
            if char == '*' && next == Some('/') {
                iter.next();
                in_comment = false;
            }
            continue;
        }

        if let Some(active_quote) = quote {
            if char == '\\' {
                iter.next();
            } else if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '/' if next == Some('*') => {
                iter.next();
                in_comment = true;
            }
            '\'' | '"' => quote = Some(char),
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn scope_selector_list(selector_list: &str, scope_id: &str) -> String {
    let selectors = split_selector_list(selector_list);
    let mut output = String::with_capacity(selector_list.len() + selectors.len() * scope_id.len());
    for (index, selector) in selectors.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(scope_selector(selector, scope_id).as_str());
    }
    output
}

fn split_selector_list(selector_list: &str) -> SmallVec<[&str; 4]> {
    let mut selectors = SmallVec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;

    let mut iter = selector_list.char_indices().peekable();
    while let Some((index, char)) = iter.next() {
        if let Some(active_quote) = quote {
            if char == '\\' {
                iter.next();
            } else if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '\\' => {
                iter.next();
            }
            '\'' | '"' => quote = Some(char),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                selectors.push(&selector_list[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }

    selectors.push(&selector_list[start..]);
    selectors
}

fn scope_selector(selector: &str, scope_id: &str) -> String {
    let Some(leading_length) = first_non_ws(selector) else {
        return String::from(selector);
    };

    let leading = &selector[..leading_length];
    let body_end = trailing_trim_end(selector);
    let trailing = &selector[body_end..];
    let mut body = unwrap_pseudo_functions(
        &selector[leading_length..body_end],
        &["::v-global(", ":global("],
    );

    if let Some(deep) = find_pseudo_function_any(body.as_str(), &["::v-deep(", "::deep(", ":deep("])
    {
        let before = body[..deep.start].trim_end();
        let inner = &body[deep.inner_start..deep.inner_end];
        let after = &body[deep.end..];
        let scoped_before = if before.is_empty() {
            scope_attr(scope_id)
        } else {
            add_scope_before_trailing_combinator(before, scope_id)
        };
        let mut scoped = String::with_capacity(scoped_before.len() + inner.len() + after.len() + 1);
        scoped.push_str(scoped_before.as_str());
        scoped.push(' ');
        scoped.push_str(inner);
        scoped.push_str(after);
        body = scoped;
    } else {
        body = add_scope_to_selector_end(body.as_str(), scope_id);
    }

    let mut output = String::with_capacity(leading.len() + body.len() + trailing.len());
    output.push_str(leading);
    output.push_str(body.as_str());
    output.push_str(trailing);
    output
}

fn split_leading_trivia(value: &str) -> (&str, &str) {
    let mut cursor = 0usize;

    loop {
        let ws_end = value[cursor..]
            .char_indices()
            .find(|(_, char)| !char.is_whitespace())
            .map_or(value.len(), |(index, _)| cursor + index);
        cursor = ws_end;

        if !value[cursor..].starts_with("/*") {
            return (&value[..cursor], &value[cursor..]);
        }

        let Some(end) = value[cursor + 2..].find("*/") else {
            return (&value[..cursor], &value[cursor..]);
        };
        cursor += 2 + end + 2;
    }
}

fn add_scope_before_trailing_combinator(selector: &str, scope_id: &str) -> String {
    let Some((combinator_start, _)) = selector
        .trim_end()
        .char_indices()
        .next_back()
        .filter(|(_, char)| matches!(char, '>' | '+' | '~'))
    else {
        return add_scope_to_selector_end(selector, scope_id);
    };

    let target = selector[..combinator_start].trim_end();
    let suffix = &selector[target.len()..];
    let mut output = if target.is_empty() {
        scope_attr(scope_id)
    } else {
        add_scope_to_selector_end(target, scope_id)
    };
    output.push_str(suffix);
    output
}

fn add_scope_to_selector_end(selector: &str, scope_id: &str) -> String {
    let target_start = find_last_compound_start(selector);
    let before_target = &selector[..target_start];
    let target = &selector[target_start..];
    let insert_at = find_scope_insert_position(target);

    let mut output = String::with_capacity(selector.len() + scope_id.len() + 2);
    output.push_str(before_target);
    output.push_str(&target[..insert_at]);
    push_scope_attr(&mut output, scope_id);
    output.push_str(&target[insert_at..]);
    output
}

fn find_last_compound_start(selector: &str) -> usize {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut whitespace_start = None;

    for (index, char) in selector.char_indices().rev() {
        if let Some(active_quote) = quote {
            if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '\'' | '"' => quote = Some(char),
            ')' => paren_depth += 1,
            '(' => paren_depth = paren_depth.saturating_sub(1),
            ']' => bracket_depth += 1,
            '[' => bracket_depth = bracket_depth.saturating_sub(1),
            '>' | '+' | '~' if paren_depth == 0 && bracket_depth == 0 => {
                return index + char.len_utf8();
            }
            char if paren_depth == 0 && bracket_depth == 0 && char.is_whitespace() => {
                whitespace_start = Some(index + char.len_utf8());
            }
            _ if whitespace_start.is_some() => {
                return whitespace_start.unwrap_or(0);
            }
            _ => {}
        }
    }

    0
}

fn find_scope_insert_position(target: &str) -> usize {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut iter = target.char_indices().peekable();

    while let Some((index, char)) = iter.next() {
        if let Some(active_quote) = quote {
            if char == '\\' {
                iter.next();
            } else if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '\\' => {
                iter.next();
            }
            '\'' | '"' => quote = Some(char),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if paren_depth == 0 && bracket_depth == 0 => return index,
            _ => {}
        }
    }

    target.len()
}

struct PseudoFunction {
    start: usize,
    inner_start: usize,
    inner_end: usize,
    end: usize,
}

fn unwrap_pseudo_functions(input: &str, markers: &[&str]) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while let Some(function) = find_pseudo_function_any_from(input, markers, cursor) {
        if function.start < cursor {
            break;
        }

        output.push_str(&input[cursor..function.start]);
        output.push_str(&input[function.inner_start..function.inner_end]);
        cursor = function.end;
        changed = true;
    }

    if !changed {
        return String::from(input);
    }

    output.push_str(&input[cursor..]);
    output
}

fn find_pseudo_function_any(input: &str, markers: &[&str]) -> Option<PseudoFunction> {
    find_pseudo_function_any_from(input, markers, 0)
}

fn find_pseudo_function_any_from(
    input: &str,
    markers: &[&str],
    cursor: usize,
) -> Option<PseudoFunction> {
    markers
        .iter()
        .filter_map(|marker| {
            let start = cursor + input[cursor..].find(marker)?;
            find_pseudo_function_from(input, marker, start)
        })
        .min_by_key(|function| function.start)
}

fn find_pseudo_function_from(input: &str, marker: &str, start: usize) -> Option<PseudoFunction> {
    let inner_start = start + marker.len();
    let inner_end = find_matching_paren(input, inner_start.checked_sub(1)?)?;
    Some(PseudoFunction {
        start,
        inner_start,
        inner_end,
        end: inner_end + 1,
    })
}

fn find_matching_paren(input: &str, open_paren: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut iter = input[open_paren..].char_indices().peekable();

    while let Some((relative, char)) = iter.next() {
        let index = open_paren + relative;
        if let Some(active_quote) = quote {
            if char == '\\' {
                iter.next();
            } else if char == active_quote {
                quote = None;
            }
            continue;
        }

        match char {
            '\'' | '"' => quote = Some(char),
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn first_non_ws(value: &str) -> Option<usize> {
    value
        .char_indices()
        .find(|(_, char)| !char.is_whitespace())
        .map(|(index, _)| index)
}

fn trailing_trim_end(value: &str) -> usize {
    value
        .char_indices()
        .rev()
        .find(|(_, char)| !char.is_whitespace())
        .map_or(0, |(index, char)| index + char.len_utf8())
}

fn scope_attr(scope_id: &str) -> String {
    let mut output = String::with_capacity(scope_id.len() + 2);
    push_scope_attr(&mut output, scope_id);
    output
}

fn push_scope_attr(output: &mut String, scope_id: &str) {
    output.push('[');
    output.push_str(scope_id);
    output.push(']');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_basic_selectors() {
        assert_eq!(
            scope_css_for_pipeline(".foo, .bar:hover { color: red; }", "data-v-x").as_str(),
            ".foo[data-v-x],.bar[data-v-x]:hover{color: red;}"
        );
    }

    #[test]
    fn unwraps_deep_inside_scoped_selector() {
        assert_eq!(
            scope_css_for_pipeline(
                ".parent :deep(.child:nth-child(2)) { color: red; }",
                "data-v-x"
            )
            .as_str(),
            ".parent[data-v-x] .child:nth-child(2){color: red;}"
        );
    }

    #[test]
    fn unwraps_legacy_v_deep_inside_scoped_selector() {
        assert_eq!(
            scope_css_for_pipeline(
                ".parent > ::v-deep(.child:nth-child(2)) { color: red; }",
                "data-v-x"
            )
            .as_str(),
            ".parent[data-v-x] > .child:nth-child(2){color: red;}"
        );
    }

    #[test]
    fn unwraps_preprocessor_special_selectors() {
        let css = "[data-v-x] .parent > ::v-deep(.child), [data-v-x] .foo:global(.bar) {}";

        assert_eq!(
            unwrap_deep_selectors(css).as_str(),
            "[data-v-x] .parent > .child, [data-v-x] .foo.bar {}"
        );
    }

    #[test]
    fn recurses_media_rules() {
        assert_eq!(
            scope_css_for_pipeline(
                "@media (min-width: 1px) { .foo { color: red; } }",
                "data-v-x"
            )
            .as_str(),
            "@media (min-width: 1px) { .foo[data-v-x]{color: red;}}"
        );
    }

    #[test]
    fn scopes_nested_css_like_vue_pipeline() {
        assert_eq!(
            scope_css_for_pipeline(
                "#pages-store { row-gap: 1.5rem; @media (--mobile) { row-gap: 1rem; } h1 { margin: 0; } :deep(.divider) { border: 0; } }",
                "data-v-x"
            )
            .as_str(),
            "#pages-store[data-v-x]{row-gap: 1.5rem;} @media (--mobile) {#pages-store[data-v-x]{row-gap: 1rem;}} #pages-store h1[data-v-x]{margin: 0;} #pages-store[data-v-x] .divider{border: 0;}"
        );
    }

    #[test]
    fn scopes_nested_css_under_media_rules() {
        assert_eq!(
            scope_css_for_pipeline(
                "@media (--mobile) { .foo { color: red; :deep(.bar) { color: blue; } } }",
                "data-v-x"
            )
            .as_str(),
            "@media (--mobile) { .foo[data-v-x]{color: red;} .foo[data-v-x] .bar{color: blue;}}"
        );
    }

    #[test]
    fn preserves_comments_before_media_rules() {
        assert_eq!(
            scope_css_for_pipeline(
                "/* desktop */\n@media (min-width: 1px) { .foo { color: red; } }",
                "data-v-x"
            )
            .as_str(),
            "/* desktop */\n@media (min-width: 1px) { .foo[data-v-x]{color: red;}}"
        );
    }

    #[test]
    fn preserves_comments_before_selectors() {
        assert_eq!(
            scope_css_for_pipeline("/* card */\n.foo { color: red; }", "data-v-x").as_str(),
            "/* card */\n.foo[data-v-x]{color: red;}"
        );
    }

    #[test]
    fn scopes_escaped_utility_selectors() {
        assert_eq!(
            scope_css_for_pipeline(
                ".hover\\:text-\\[\\#00DC82\\]:hover { color: red; }.text-\\[80px\\] { font-size: 80px; }",
                "data-v-x"
            )
            .as_str(),
            ".hover\\:text-\\[\\#00DC82\\][data-v-x]:hover{color: red;}.text-\\[80px\\][data-v-x]{font-size: 80px;}"
        );
    }
}
