//! Structural inspection of an authored prop type's textual shape.
//!
//! Deciding whether the template needs per-key prop bindings depends on the
//! *shape* of the authored type, not on its resolved members: a top-level union
//! or intersection, or a named type that is not a plain inline literal, cannot
//! be indexed member-by-member. Both checks scan the type text, so they live
//! next to their unit tests rather than in the model itself.

pub(super) fn is_plain_inline_type_literal(type_name: &str) -> bool {
    let type_name = type_name.trim();
    if !type_name.starts_with('{') {
        return false;
    }

    let mut depth = 0i32;
    for (idx, c) in type_name.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return type_name[idx + c.len_utf8()..].trim().is_empty();
                }
            }
            _ => {}
        }
    }
    false
}

pub(super) fn has_top_level_type_operator(type_name: &str) -> bool {
    let mut angle_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;

    // An arrow's `>` closes nothing: counting it would drive the angle depth
    // negative for a function-valued member, so every operator after it would
    // look nested and a union such as
    // `{ onPick: (v: string) => void } | { onPick: null }` would be missed.
    // Closing delimiters clamp at zero for the same reason: an unbalanced input
    // must not hide a later top-level operator.
    let mut prev = '\0';
    for c in type_name.chars() {
        match c {
            '<' => angle_depth += 1,
            '>' if prev != '=' => angle_depth = (angle_depth - 1).max(0),
            '{' => brace_depth += 1,
            '}' => brace_depth = (brace_depth - 1).max(0),
            '(' => paren_depth += 1,
            ')' => paren_depth = (paren_depth - 1).max(0),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = (bracket_depth - 1).max(0),
            '&' | '|'
                if angle_depth == 0
                    && brace_depth == 0
                    && paren_depth == 0
                    && bracket_depth == 0 =>
            {
                return true;
            }
            _ => {}
        }
        prev = c;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{has_top_level_type_operator, is_plain_inline_type_literal};

    #[test]
    fn a_plain_inline_literal_is_recognized() {
        assert!(is_plain_inline_type_literal("{ a: string }"));
        assert!(is_plain_inline_type_literal("  { a: { b: number } }  "));
    }

    #[test]
    fn a_named_or_suffixed_type_is_not_a_plain_literal() {
        assert!(!is_plain_inline_type_literal("Props"));
        assert!(!is_plain_inline_type_literal("{ a: string }[]"));
        assert!(!is_plain_inline_type_literal("{ a: string } & Props"));
    }

    #[test]
    fn a_brace_inside_a_string_member_falls_back_to_keyed_bindings() {
        // The scan is textual, so a `}` inside a string literal closes the
        // literal early and leaves a trailing suffix. Reporting "not plain"
        // there is the safe direction: the caller then emits keyed bindings,
        // which resolve for any shape.
        assert!(!is_plain_inline_type_literal("{ a: '}' }"));
    }

    #[test]
    fn top_level_union_survives_a_function_valued_member() {
        assert!(has_top_level_type_operator(
            "{ onPick: (v: string) => void } | { onPick: null }"
        ));
        assert!(has_top_level_type_operator("(() => void) | null"));
        assert!(has_top_level_type_operator("{ a: string } & { b: number }"));
    }

    #[test]
    fn nested_operators_are_not_top_level() {
        assert!(!has_top_level_type_operator(
            "{ onPick: (v: string) => void }"
        ));
        assert!(!has_top_level_type_operator("Array<string | number>"));
    }

    #[test]
    fn an_unbalanced_closer_does_not_hide_a_later_operator() {
        assert!(has_top_level_type_operator("Unexpected } | string"));
    }
}
