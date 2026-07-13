//! Type-reference and generic-parameter text helpers.

use vize_carton::String;

/// Lookup key for a `defineProps<...>` type argument when resolving its fields.
pub(crate) fn type_reference_lookup_key(type_name: &str) -> &str {
    if type_name.trim_start().starts_with('{') {
        type_name
    } else {
        strip_generic_params(type_name).trim()
    }
}

/// Strip the outermost `<...>` pair, handling nested generics.
pub(crate) fn strip_outer_angle_brackets(s: &str) -> &str {
    let s = s.trim();
    if !s.starts_with('<') {
        return s;
    }
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 && i == s.len() - 1 {
                    return &s[1..i];
                }
            }
            _ => {}
        }
    }
    s
}

/// Strip generic parameters from a type name for interface lookup.
pub(super) fn strip_generic_params(type_name: &str) -> &str {
    match type_name.find('<') {
        Some(pos) => &type_name[..pos],
        None => type_name,
    }
}

fn generic_param_name(param: &str) -> &str {
    let mut tokens = param.split_whitespace();
    match tokens.next() {
        Some("const") => tokens.next().unwrap_or(param),
        Some(token) => token,
        None => param,
    }
}

/// Extract parameter names from a full generic declaration.
pub(crate) fn extract_generic_names(generic_param: &str) -> String {
    let mut names = String::default();
    let mut depth = 0i32;
    let mut current_name = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                push_generic_name(&mut names, current_name.trim());
                current_name = String::default();
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            current_name.push(ch);
        }
    }
    push_generic_name(&mut names, current_name.trim());
    names
}

fn push_generic_name(names: &mut String, parameter: &str) {
    if parameter.is_empty() {
        return;
    }
    if !names.is_empty() {
        names.push_str(", ");
    }
    names.push_str(generic_param_name(parameter));
}

/// Drop TS 5.0 `const` modifiers from a generic parameter list.
pub(crate) fn strip_const_modifiers(generic_param: &str) -> String {
    let mut result = String::default();
    let mut depth = 0i32;
    let mut current_param = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                append_param_without_const(&mut result, current_param.trim());
                result.push_str(", ");
                current_param = String::default();
                continue;
            }
            _ => {}
        }
        current_param.push(ch);
    }
    let trimmed = current_param.trim();
    if !trimmed.is_empty() {
        append_param_without_const(&mut result, trimmed);
    }
    result
}

fn append_param_without_const(result: &mut String, param: &str) {
    let stripped = param
        .strip_prefix("const")
        .filter(|rest| rest.starts_with(|ch: char| ch.is_ascii_whitespace()))
        .map(str::trim_start)
        .unwrap_or(param);
    result.push_str(stripped);
}

/// Add `= any` defaults to generic parameters without an existing default.
pub(crate) fn add_generic_defaults(generic_param: &str) -> String {
    let mut result = String::default();
    let mut depth = 0i32;
    let mut current_param = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => {
                depth += 1;
                current_param.push(ch);
            }
            '>' => {
                depth -= 1;
                current_param.push(ch);
            }
            ',' if depth == 0 => {
                append_param_with_default(&mut result, current_param.trim());
                result.push_str(", ");
                current_param = String::default();
            }
            _ => current_param.push(ch),
        }
    }
    let trimmed = current_param.trim();
    if !trimmed.is_empty() {
        append_param_with_default(&mut result, trimmed);
    }
    result
}

fn append_param_with_default(result: &mut String, param: &str) {
    result.push_str(param);
    let mut depth = 0i32;
    let has_default = param.chars().any(|ch| {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            '=' if depth == 0 => return true,
            _ => {}
        }
        false
    });
    if !has_default {
        result.push_str(" = any");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_generic_names_skipping_const_modifier() {
        assert_eq!(
            extract_generic_names("T extends Foo, P extends Bar"),
            "T, P"
        );
        assert_eq!(extract_generic_names("const T extends Tab"), "T");
        assert_eq!(
            extract_generic_names("const T extends Record<string, any>, U"),
            "T, U"
        );
        assert_eq!(extract_generic_names("constant extends Foo"), "constant");
    }

    #[test]
    fn strips_const_modifiers_for_type_declarations() {
        assert_eq!(
            strip_const_modifiers("const T extends Tab = any").as_str(),
            "T extends Tab = any"
        );
        assert_eq!(
            strip_const_modifiers("const T extends Record<string, any>, const U = any").as_str(),
            "T extends Record<string, any>, U = any"
        );
        assert_eq!(
            strip_const_modifiers(add_generic_defaults("const T extends Tab").as_str()).as_str(),
            "T extends Tab = any"
        );
    }

    #[test]
    fn lookup_key_strips_generics_but_preserves_inline_literals() {
        assert_eq!(type_reference_lookup_key("Props"), "Props");
        assert_eq!(type_reference_lookup_key("Foo<T>"), "Foo");
        assert_eq!(
            type_reference_lookup_key("ContextMenuContentProps<T, U>"),
            "ContextMenuContentProps"
        );
        assert_eq!(
            type_reference_lookup_key("{ items: Array<{ id: string }> }"),
            "{ items: Array<{ id: string }> }"
        );
        assert_eq!(
            type_reference_lookup_key("  { msg: string }"),
            "  { msg: string }"
        );
    }
}
