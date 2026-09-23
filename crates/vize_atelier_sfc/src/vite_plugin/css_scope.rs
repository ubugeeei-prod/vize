use crate::css::scoped_selector::split_before_trailing_universal_or_pseudo;
use vize_carton::{SmallVec, String};

mod legacy_deep;
mod slotted;

/// Scope CSS with the Vite plugin pipeline's selector model.
pub(super) fn scope_css_for_pipeline(css: &str, scope_id: &str) -> String {
    transform_css_block(css, scope_id)
}

pub(super) fn unwrap_deep_selectors(css: &str) -> String {
    unwrap_pseudo_functions(
        css,
        &[
            "::v-deep(",
            "::deep(",
            ":deep(",
            "::v-slotted(",
            ":slotted(",
            "::v-global(",
            ":global(",
        ],
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
        let rule = find_next_top_level_brace(css, cursor).and_then(|brace| {
            let end = find_matching_brace(css, brace)?;
            let header_start = find_rule_header_start(css, cursor, brace);
            Some((
                css.get(cursor..header_start)?,
                css.get(header_start..brace)?,
                css.get(brace + 1..end)?,
                end,
            ))
        });
        let Some((pending, header, body, end)) = rule else {
            declarations.push_str(css.get(cursor..).unwrap_or_default());
            break;
        };

        declarations.push_str(pending);
        flush_declarations(
            &mut output,
            parent_selectors,
            scope_id,
            declarations.as_str(),
        );
        declarations.clear();

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
    ["@container", "@layer", "@media", "@supports"]
        .contains(&statement.split_whitespace().next().unwrap_or(""))
}

fn find_rule_header_start(css: &str, start: usize, brace: usize) -> usize {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut in_comment = false;
    let mut header_start = start;
    let mut iter = css
        .get(start..brace)
        .unwrap_or_default()
        .char_indices()
        .peekable();

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
    let mut iter = css
        .get(start..)
        .unwrap_or_default()
        .char_indices()
        .peekable();

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
    let mut iter = css
        .get(start..)
        .unwrap_or_default()
        .char_indices()
        .peekable();

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
                selectors.push(selector_list.get(start..index).unwrap_or_default());
                start = index + 1;
            }
            _ => {}
        }
    }

    selectors.push(selector_list.get(start..).unwrap_or_default());
    selectors
}

fn scope_selector(selector: &str, scope_id: &str) -> String {
    let Some(leading_length) = first_non_ws(selector) else {
        return String::from(selector);
    };

    let body_end = trailing_trim_end(selector);
    let (Some(leading), Some(body), Some(trailing)) = (
        selector.get(..leading_length),
        selector.get(leading_length..body_end),
        selector.get(body_end..),
    ) else {
        return String::from(selector);
    };
    let mut body = legacy_deep::normalize_scoped_selector_body(body);

    if let Some(slotted) = find_pseudo_function_any(body.as_str(), &["::v-slotted(", ":slotted("]) {
        body = slotted::scope_slotted_selector(body.as_str(), &slotted, scope_id);
    } else if let Some(deep) =
        find_pseudo_function_any(body.as_str(), &["::v-deep(", "::deep(", ":deep("])
    {
        let (before, inner, after) = deep.parts(body.as_str());
        let before = before.trim_end();
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
        let Some((_, rest)) = value.split_at_checked(cursor) else {
            return ("", value);
        };
        cursor += rest
            .char_indices()
            .find(|(_, char)| !char.is_whitespace())
            .map_or(rest.len(), |(index, _)| index);
        let Some((leading, rest)) = value.split_at_checked(cursor) else {
            return ("", value);
        };

        let Some(end) = rest
            .strip_prefix("/*")
            .and_then(|comment| comment.find("*/"))
        else {
            return (leading, rest);
        };
        cursor += 2 + end + 2;
    }
}

fn add_scope_before_trailing_combinator(selector: &str, scope_id: &str) -> String {
    let trimmed = selector.trim_end();
    let Some(combinator_start) = trailing_combinator_start(trimmed) else {
        return add_scope_to_selector_end(selector, scope_id);
    };

    let target = selector
        .get(..combinator_start)
        .unwrap_or_default()
        .trim_end();
    let mut output = if target.is_empty() {
        scope_attr(scope_id)
    } else {
        add_scope_to_selector_end(target, scope_id)
    };
    output.push_str(selector.get(target.len()..).unwrap_or_default());
    output
}

fn trailing_combinator_start(selector: &str) -> Option<usize> {
    selector
        .strip_suffix("||")
        .or_else(|| selector.strip_suffix(['>', '+', '~']))
        .map(str::len)
}

fn add_scope_to_selector_end(selector: &str, scope_id: &str) -> String {
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

impl PseudoFunction {
    /// The text before the function, its argument, and the text after `)`.
    fn parts<'s>(&self, input: &'s str) -> (&'s str, &'s str, &'s str) {
        (
            input.get(..self.start).unwrap_or_default(),
            input
                .get(self.inner_start..self.inner_end)
                .unwrap_or_default(),
            input.get(self.end..).unwrap_or_default(),
        )
    }
}

fn unwrap_pseudo_functions(input: &str, markers: &[&str]) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while let Some(function) = find_pseudo_function_any_from(input, markers, cursor) {
        if function.start < cursor {
            break;
        }

        output.push_str(input.get(cursor..function.start).unwrap_or_default());
        output.push_str(function.parts(input).1);
        cursor = function.end;
        changed = true;
    }

    if !changed {
        return String::from(input);
    }

    output.push_str(input.get(cursor..).unwrap_or_default());
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
            let start = cursor + input.get(cursor..)?.find(marker)?;
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
    let mut iter = input.get(open_paren..)?.char_indices().peekable();

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
mod tests;
