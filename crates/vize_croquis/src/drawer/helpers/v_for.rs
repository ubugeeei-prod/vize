//! v-for expression parsing.
//!
//! Parses `v-for` directive values like `"item in items"` or
//! `"(item, index) in items"` into separate variable bindings
//! and the iterable source expression.
//!
//! Splits the Vue-specific `in`/`of` boundary, then normalizes aliases into
//! JavaScript binding patterns and walks the OXC AST for binding names.

use vize_carton::{CompactString, SmallVec, profile, smallvec};

/// Parsed aliases for a v-for scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VForScopeAliases {
    /// Pattern used for the value parameter, e.g. `item` or `{ id, name }`.
    pub value_pattern: CompactString,
    /// Bindings introduced by the value pattern.
    pub value_bindings: SmallVec<[CompactString; 4]>,
    /// Optional key alias from tuple syntax.
    pub key_alias: Option<CompactString>,
    /// Optional index alias from tuple syntax.
    pub index_alias: Option<CompactString>,
    /// Iterable source expression.
    pub source: CompactString,
}

/// Parse v-for expression into variables and source
#[inline]
pub fn parse_v_for_expression(expr: &str) -> (SmallVec<[CompactString; 3]>, CompactString) {
    let expr = expr.trim();
    parse_first_v_for_candidate(expr, |alias_part, source_part| {
        let source = CompactString::new(source_part);
        let (bindings, source) = profile!(
            "croquis.helpers.v_for.oxc",
            oxc::parse_v_for_with_oxc(alias_part, source)
        );

        (!bindings.is_empty()).then_some((bindings, source))
    })
    .unwrap_or_else(|| (smallvec![], CompactString::new(expr)))
}

/// Parse v-for expression into structured scope aliases.
#[inline]
pub fn parse_v_for_scope_expression(expr: &str) -> Option<VForScopeAliases> {
    parse_first_v_for_candidate(expr.trim(), |alias_part, source_part| {
        let source = CompactString::new(source_part);

        profile!(
            "croquis.helpers.v_for.scope_oxc",
            oxc::parse_v_for_scope_aliases(alias_part.trim_start_matches("const ").trim(), source)
        )
    })
}

fn parse_first_v_for_candidate<T>(
    expr: &str,
    mut parse: impl FnMut(&str, &str) -> Option<T>,
) -> Option<T> {
    let mut previous_char = None;

    for (keyword_start, ch) in expr.char_indices() {
        let keyword_len = match ch {
            'i' if expr[keyword_start..].starts_with("in") => 2,
            'o' if expr[keyword_start..].starts_with("of") => 2,
            _ => {
                previous_char = Some(ch);
                continue;
            }
        };

        if !previous_char.is_some_and(char::is_whitespace) {
            previous_char = Some(ch);
            continue;
        }

        let keyword_end = keyword_start + keyword_len;
        if !expr[keyword_end..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            previous_char = Some(ch);
            continue;
        }

        let alias_part = expr[..keyword_start].trim();
        let source_part = expr[keyword_end..].trim();
        if !alias_part.is_empty()
            && !source_part.is_empty()
            && let Some(parsed) = parse(alias_part, source_part)
        {
            return Some(parsed);
        }

        previous_char = Some(ch);
    }

    None
}

mod oxc;

#[cfg(test)]
mod tests;
