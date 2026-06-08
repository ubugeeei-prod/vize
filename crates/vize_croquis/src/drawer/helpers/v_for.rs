//! v-for expression parsing.
//!
//! Parses `v-for` directive values like `"item in items"` or
//! `"(item, index) in items"` into separate variable bindings
//! and the iterable source expression.
//!
//! Uses fast-path string scanning for simple patterns and falls
//! back to OXC parsing for destructured bindings.

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
    let Some((alias_part, source_part)) = split_v_for_expression(expr) else {
        return (smallvec![], CompactString::new(expr.trim()));
    };
    let source = CompactString::new(source_part);

    // Fast path: simple identifier
    if !alias_part.starts_with('(')
        && !alias_part.contains('{')
        && is_valid_identifier_fast(alias_part.as_bytes())
    {
        return (smallvec![CompactString::new(alias_part)], source);
    }

    // Fast path: simple tuple (item, index)
    if alias_part.starts_with('(') && alias_part.ends_with(')') && !alias_part.contains('{') {
        let inner = &alias_part[1..alias_part.len() - 1];
        let mut vars = SmallVec::new();
        for part in inner.split(',') {
            let part = part.trim();
            if !part.is_empty() && is_valid_identifier_fast(part.as_bytes()) {
                vars.push(CompactString::new(part));
            }
        }
        if !vars.is_empty() {
            return (vars, source);
        }
    }

    // Complex case: use OXC parser
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

    let (value_pattern, key_alias, index_alias) =
        split_v_for_aliases(alias_part.trim_start_matches("const ").trim());
    let value_pattern = value_pattern.trim();
    let value_bindings = oxc::extract_binding_names_from_pattern(value_pattern);
    if value_bindings.is_empty() {
        return None;
    }

    Some(VForScopeAliases {
        value_pattern: CompactString::new(value_pattern),
        value_bindings,
        key_alias,
        index_alias,
        source,
    })
}

fn split_v_for_expression(expr: &str) -> Option<(&str, &str)> {
    let expr = expr.trim();
    let bytes = expr.as_bytes();
    let len = bytes.len();

    // Find " in " or " of " separator
    let mut split_pos = None;
    let mut i = 0;
    while i + 4 <= len {
        if bytes[i] == b' '
            && ((bytes[i + 1] == b'i' && bytes[i + 2] == b'n')
                || (bytes[i + 1] == b'o' && bytes[i + 2] == b'f'))
            && bytes[i + 3] == b' '
        {
            split_pos = Some(i);
            break;
        }
        i += 1;
    }

    let pos = split_pos?;

    let alias_part = expr[..pos].trim();
    let source_part = expr[pos + 4..].trim();
    Some((alias_part, source_part))
}

fn split_v_for_aliases(alias: &str) -> (&str, Option<CompactString>, Option<CompactString>) {
    let Some(inner) = enclosing_parens_inner(alias) else {
        return (alias, None, None);
    };
    let parts = split_top_level_commas(inner);
    if parts.len() <= 1 {
        return (parts.first().copied().unwrap_or(inner), None, None);
    }

    let key_alias = parts.get(1).and_then(|part| simple_alias(part));
    let index_alias = parts.get(2).and_then(|part| simple_alias(part));
    (parts[0], key_alias, index_alias)
}

fn simple_alias(part: &str) -> Option<CompactString> {
    let alias = part.trim();
    is_valid_identifier_fast(alias.as_bytes()).then(|| CompactString::new(alias))
}

fn enclosing_parens_inner(text: &str) -> Option<&str> {
    let text = text.trim();
    if !text.starts_with('(') || !text.ends_with(')') {
        return None;
    }

    let mut depth = 0usize;
    let last = text.len() - 1;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 && index != last {
                    return None;
                }
            }
            _ => {}
        }
    }

    (depth == 0).then_some(text[1..last].trim())
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_char {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                let part = text[start..index].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    let part = text[start..].trim();
    if !part.is_empty() {
        parts.push(part);
    }
    parts
}

mod oxc;

#[cfg(test)]
mod tests;
