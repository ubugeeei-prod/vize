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
    let Some((alias_part, source_part)) = split_v_for_expression(expr) else {
        return (smallvec![], CompactString::new(expr.trim()));
    };
    let source = CompactString::new(source_part);

    profile!(
        "croquis.helpers.v_for.oxc",
        oxc::parse_v_for_with_oxc(alias_part, source)
    )
}

/// Parse v-for expression into structured scope aliases.
#[inline]
pub fn parse_v_for_scope_expression(expr: &str) -> Option<VForScopeAliases> {
    let (alias_part, source_part) = split_v_for_expression(expr)?;
    let source = CompactString::new(source_part);

    profile!(
        "croquis.helpers.v_for.scope_oxc",
        oxc::parse_v_for_scope_aliases(alias_part.trim_start_matches("const ").trim(), source)
    )
}

fn split_v_for_expression(expr: &str) -> Option<(&str, &str)> {
    let expr = expr.trim();
    let chars: Vec<_> = expr.char_indices().collect();

    for keyword_idx in 0..chars.len().saturating_sub(1) {
        match (chars[keyword_idx].1, chars[keyword_idx + 1].1) {
            ('i', 'n') | ('o', 'f') => {}
            _ => continue,
        }

        let has_space_before = keyword_idx > 0 && chars[keyword_idx - 1].1.is_whitespace();
        let after_keyword_idx = keyword_idx + 2;
        let has_space_after = chars
            .get(after_keyword_idx)
            .is_some_and(|(_, ch)| ch.is_whitespace());
        if !has_space_before || !has_space_after {
            continue;
        }

        let keyword_start = chars[keyword_idx].0;
        let keyword_end = chars
            .get(after_keyword_idx)
            .map_or(expr.len(), |(byte_idx, _)| *byte_idx);
        let alias_part = expr[..keyword_start].trim();
        let source_part = expr[keyword_end..].trim();

        if !alias_part.is_empty()
            && !source_part.is_empty()
            && oxc::is_valid_v_for_alias(alias_part)
            && oxc::is_valid_expression(source_part)
        {
            return Some((alias_part, source_part));
        }
    }

    None
}

mod oxc;

#[cfg(test)]
mod tests;
