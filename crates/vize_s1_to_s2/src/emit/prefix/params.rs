//! Alias / slot-param name extraction, ported from
//! `vize_atelier_core::codegen::v_for::helpers` (`extract_destructure_params`)
//! and `steps::v_slot::params` (`extract_slot_prop_names`).

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::expression_guard::expression_is_safe_to_parse;
use vize_s0::{Allocator, SmallVec, String};

/// Parameter names of a destructuring pattern string, in the shipped
/// codegen's text-scanning order.
pub(super) fn extract_destructure_params(trimmed: &str, params: &mut StdVec<String>) {
    walk_destructure_params(trimmed, &mut |param| {
        params.push(String::from(param));
        false
    });
}

/// Whether the shipped scanner would list `name` among the pattern's
/// parameters. Allocation-free: the default lane asks this per dynamic
/// key without materialising the list.
pub(in crate::emit) fn destructure_params_contain(trimmed: &str, name: &str) -> bool {
    walk_destructure_params(trimmed, &mut |param| param == name)
}

/// The shipped scanner's walk; `visit` returns `true` to stop early (the
/// walk then reports `true`).
fn walk_destructure_params(trimmed: &str, visit: &mut dyn FnMut(&str) -> bool) -> bool {
    let mut pending = SmallVec::<[&str; 8]>::new();
    pending.push(trimmed);

    while let Some(trimmed) = pending.pop() {
        if trimmed.contains(',') && !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            for part in split_top_level(trimmed).into_iter().rev() {
                if part == trimmed {
                    continue;
                }
                let part = part.trim();
                if !part.is_empty() {
                    pending.push(part);
                }
            }
            continue;
        }

        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner = &trimmed[1..trimmed.len() - 1];
            for part in split_top_level(inner).into_iter().rev() {
                let part = part.trim();
                if let Some(rest) = part.strip_prefix("...") {
                    let rest = rest.trim();
                    if !rest.is_empty() {
                        pending.push(rest);
                    }
                    continue;
                }
                if let Some(colon_pos) = find_top_level_char(part, ':') {
                    let value = strip_default_value(part[colon_pos + 1..].trim());
                    if !value.is_empty() {
                        pending.push(value);
                    }
                    continue;
                }
                let part = strip_default_value(part);
                if !part.is_empty() {
                    pending.push(part);
                }
            }
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let inner = &trimmed[1..trimmed.len() - 1];
            for part in split_top_level(inner).into_iter().rev() {
                let part = part.trim();
                if let Some(rest) = part.strip_prefix("...") {
                    let rest = rest.trim();
                    if !rest.is_empty() {
                        pending.push(rest);
                    }
                    continue;
                }
                let part = strip_default_value(part);
                if !part.is_empty() {
                    pending.push(part);
                }
            }
        } else if super::super::js::is_valid_js_identifier(trimmed) && visit(trimmed) {
            return true;
        }
    }
    false
}

fn split_top_level(s: &str) -> SmallVec<[&str; 8]> {
    let mut parts = SmallVec::new();
    let mut depth = 0i32;
    let mut quote = None;
    let mut start = 0;
    let mut prev = '\0';
    for (i, ch) in s.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + ch.len_utf8();
            }
            _ => {}
        }
        prev = ch;
    }
    parts.push(&s[start..]);
    parts
}

fn strip_default_value(pattern: &str) -> &str {
    if let Some(index) = find_top_level_char(pattern, '=') {
        pattern[..index].trim()
    } else {
        pattern.trim()
    }
}

fn find_top_level_char(s: &str, needle: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';
    for (i, ch) in s.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            _ if ch == needle && depth == 0 => return Some(i),
            _ => {}
        }
        prev = ch;
    }
    None
}

/// Binding names of a slot props pattern, parsed as `let <pattern> = __slotProps`
/// in the TS dialect — the transform's `enter_v_slot_scope` names.
pub(super) fn extract_slot_prop_names(pattern: &str) -> StdVec<String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() || !expression_is_safe_to_parse(trimmed) {
        return StdVec::new();
    }
    let mut source = String::with_capacity(trimmed.len() + 18);
    source.push_str("let ");
    source.push_str(trimmed);
    source.push_str(" = __slotProps");

    let allocator = Allocator::new();
    let source_type = SourceType::default().with_typescript(true);
    let parsed = Parser::new(allocator.as_oxc(), source.as_str(), source_type).parse();
    let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = parsed.program.body.first()
    else {
        return StdVec::new();
    };
    let Some(declarator) = var_decl.declarations.first() else {
        return StdVec::new();
    };
    let mut names = StdVec::new();
    collect_slot_binding_names(&declarator.id, &mut names);
    names
}

fn collect_slot_binding_names(pattern: &BindingPattern<'_>, names: &mut StdVec<String>) {
    let mut pending = SmallVec::<[&BindingPattern<'_>; 8]>::new();
    pending.push(pattern);
    while let Some(pattern) = pending.pop() {
        match pattern {
            BindingPattern::BindingIdentifier(id) => names.push(String::from(id.name.as_str())),
            BindingPattern::ObjectPattern(obj) => {
                if let Some(rest) = &obj.rest {
                    pending.push(&rest.argument);
                }
                for prop in obj.properties.iter().rev() {
                    pending.push(&prop.value);
                }
            }
            BindingPattern::ArrayPattern(arr) => {
                if let Some(rest) = &arr.rest {
                    pending.push(&rest.argument);
                }
                for elem in arr.elements.iter().rev().flatten() {
                    pending.push(elem);
                }
            }
            BindingPattern::AssignmentPattern(assign) => pending.push(&assign.left),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        destructure_params_contain, extract_destructure_params, extract_slot_prop_names,
        split_top_level,
    };
    use alloc::vec::Vec as StdVec;

    #[test]
    fn destructure_params_match_the_shipped_scanner() {
        let mut params = StdVec::new();
        extract_destructure_params(
            r#"{ id: itemId = fallback, user: { name = "a,b" }, tags: [firstTag = "x,y"] }"#,
            &mut params,
        );
        assert_eq!(params, ["itemId", "name", "firstTag"]);
        let mut params = StdVec::new();
        extract_destructure_params("item, index", &mut params);
        assert_eq!(params, ["item", "index"]);
        let mut params = StdVec::new();
        extract_destructure_params("$data.[label, value]", &mut params);
        assert!(params.is_empty());
    }

    #[test]
    fn destructure_params_contain_matches_the_list() {
        let pattern = r#"{ id: itemId = fallback, user: { name = "a,b" }, tags: [firstTag] }"#;
        assert!(destructure_params_contain(pattern, "itemId"));
        assert!(destructure_params_contain(pattern, "firstTag"));
        assert!(!destructure_params_contain(pattern, "fallback"));
        assert!(!destructure_params_contain(pattern, "id"));
        assert!(destructure_params_contain("item", "item"));
        assert!(!destructure_params_contain("item.value", "item"));
    }

    #[test]
    fn split_top_level_ignores_commas_inside_strings() {
        assert_eq!(
            split_top_level(r#"id = "a,b", name: label, nested: { value: "c,d" }"#).as_slice(),
            [
                r#"id = "a,b""#,
                " name: label",
                r#" nested: { value: "c,d" }"#
            ]
        );
    }

    #[test]
    fn slot_prop_names_match_the_shipped_extractor() {
        assert_eq!(
            extract_slot_prop_names("{ item, index }"),
            ["item", "index"]
        );
        assert_eq!(
            extract_slot_prop_names("{ item: { id }, index = 0, ...rest }"),
            ["id", "index", "rest"]
        );
        assert_eq!(extract_slot_prop_names("props"), ["props"]);
        assert!(extract_slot_prop_names("").is_empty());
    }
}
