//! v-for expression parsing.
//!
//! Parses `v-for` directive values like `"item in items"` or
//! `"(item, index) in items"` into separate variable bindings
//! and the iterable source expression.
//!
//! Splits the Vue-specific `in`/`of` boundary, then normalizes aliases into
//! JavaScript binding patterns and walks the OXC AST for binding names.

use vize_carton::{CompactString, SmallVec, profile, smallvec};

use super::is_valid_identifier_fast;

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
    let parsed = parse_first_v_for_candidate(expr, |alias_part, source_part| {
        if let Some(bindings) = parse_simple_v_for_bindings(alias_part) {
            return Some((bindings, CompactString::new(source_part)));
        }

        let source = CompactString::new(source_part);
        let (bindings, source) = profile!(
            "croquis.helpers.v_for.oxc",
            oxc::parse_v_for_with_oxc(alias_part, source)
        );

        (!bindings.is_empty()).then_some((bindings, source))
    });
    #[cfg(any(test, feature = "davinci-differential"))]
    if parsed.is_some() {
        crate::drawer::differential::record_v_for_shape();
    }
    parsed.unwrap_or_else(|| (smallvec![], CompactString::new(expr)))
}

/// Parse v-for expression into structured scope aliases.
#[inline]
pub fn parse_v_for_scope_expression(expr: &str) -> Option<VForScopeAliases> {
    let aliases = parse_first_v_for_candidate(expr.trim(), |alias_part, source_part| {
        if let Some(aliases) = parse_simple_v_for_scope_aliases(alias_part, source_part) {
            return Some(aliases);
        }

        let source = CompactString::new(source_part);

        profile!(
            "croquis.helpers.v_for.scope_oxc",
            oxc::parse_v_for_scope_aliases(alias_part.trim_start_matches("const ").trim(), source)
        )
    });
    #[cfg(any(test, feature = "davinci-differential"))]
    if aliases.is_some() {
        crate::drawer::differential::record_v_for_shape();
    }
    aliases
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

fn parse_simple_v_for_bindings(alias: &str) -> Option<SmallVec<[CompactString; 3]>> {
    let alias = alias.trim_start_matches("const ").trim();
    if is_valid_identifier_fast(alias.as_bytes()) {
        return Some(smallvec![CompactString::new(alias)]);
    }

    let inner = simple_tuple_inner(alias)?;
    let mut bindings = SmallVec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if !is_valid_identifier_fast(part.as_bytes()) {
            return None;
        }
        bindings.push(CompactString::new(part));
    }

    (!bindings.is_empty()).then_some(bindings)
}

fn parse_simple_v_for_scope_aliases(alias: &str, source: &str) -> Option<VForScopeAliases> {
    let alias = alias.trim_start_matches("const ").trim();
    if is_valid_identifier_fast(alias.as_bytes()) {
        return Some(VForScopeAliases {
            value_pattern: CompactString::new(alias),
            value_bindings: smallvec![CompactString::new(alias)],
            key_alias: None,
            index_alias: None,
            source: CompactString::new(source),
        });
    }

    let inner = simple_tuple_inner(alias)?;
    let mut parts = inner.split(',');
    let value_pattern = parts.next()?.trim();
    if !is_valid_identifier_fast(value_pattern.as_bytes()) {
        return None;
    }

    let key_alias = simple_tuple_part(parts.next())?;
    let index_alias = simple_tuple_part(parts.next())?;
    if parts.any(|part| !part.trim().is_empty()) {
        return None;
    }

    Some(VForScopeAliases {
        value_pattern: CompactString::new(value_pattern),
        value_bindings: smallvec![CompactString::new(value_pattern)],
        key_alias,
        index_alias,
        source: CompactString::new(source),
    })
}

fn simple_tuple_part(part: Option<&str>) -> Option<Option<CompactString>> {
    let Some(part) = part else {
        return Some(None);
    };
    let part = part.trim();
    if part.is_empty() {
        return Some(None);
    }
    is_valid_identifier_fast(part.as_bytes()).then(|| Some(CompactString::new(part)))
}

fn simple_tuple_inner(alias: &str) -> Option<&str> {
    let alias = alias.trim();
    if alias.starts_with('(') && alias.ends_with(')') {
        Some(alias[1..alias.len() - 1].trim())
    } else {
        None
    }
}

mod oxc;

#[cfg(test)]
mod tests;
