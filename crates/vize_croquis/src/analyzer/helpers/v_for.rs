//! v-for expression parsing.
//!
//! Parses `v-for` directive values like `"item in items"` or
//! `"(item, index) in items"` into separate variable bindings
//! and the iterable source expression.
//!
//! Uses fast-path string scanning for simple patterns and falls
//! back to OXC parsing for destructured bindings.

use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, SmallVec, String, profile, smallvec};

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
        parse_v_for_with_oxc(alias_part, source)
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
    let value_bindings = extract_binding_names_from_pattern(value_pattern);
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

/// Parse complex v-for alias using OXC
#[cold]
fn parse_v_for_with_oxc(
    alias: &str,
    source: CompactString,
) -> (SmallVec<[CompactString; 3]>, CompactString) {
    let mut buffer = [0u8; 256];
    let prefix = b"let [";
    let suffix = b"] = x";

    let inner = if alias.starts_with('(') && alias.ends_with(')') {
        &alias[1..alias.len() - 1]
    } else {
        alias
    };

    let total_len = prefix.len() + inner.len() + suffix.len();
    if total_len > buffer.len() {
        #[allow(clippy::disallowed_macros)]
        let pattern_str = format!("let [{inner}] = x");
        return profile!(
            "croquis.helpers.v_for.parse_pattern",
            parse_v_for_pattern(&pattern_str, source)
        );
    }

    buffer[..prefix.len()].copy_from_slice(prefix);
    buffer[prefix.len()..prefix.len() + inner.len()].copy_from_slice(inner.as_bytes());
    buffer[prefix.len() + inner.len()..total_len].copy_from_slice(suffix);

    match std::str::from_utf8(&buffer[..total_len]) {
        Ok(pattern_str) => profile!(
            "croquis.helpers.v_for.parse_pattern",
            parse_v_for_pattern(pattern_str, source)
        ),
        Err(_) => (SmallVec::new(), source),
    }
}

/// Parse v-for pattern using OXC
fn parse_v_for_pattern(
    pattern_str: &str,
    source: CompactString,
) -> (SmallVec<[CompactString; 3]>, CompactString) {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = profile!(
        "croquis.helpers.v_for.oxc_parse",
        Parser::new(&allocator, pattern_str, source_type).parse()
    );

    let mut vars = SmallVec::new();

    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first()
        && let Some(declarator) = var_decl.declarations.first()
    {
        extract_binding_names(&declarator.id, &mut vars);
    }

    (vars, source)
}

fn extract_binding_names_from_pattern(pattern: &str) -> SmallVec<[CompactString; 4]> {
    if is_valid_identifier_fast(pattern.as_bytes()) {
        return smallvec![CompactString::new(pattern)];
    }

    let pattern_str = format_binding_pattern(pattern);
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = profile!(
        "croquis.helpers.v_for.oxc_scope_parse",
        Parser::new(&allocator, &pattern_str, source_type).parse()
    );

    let mut vars = SmallVec::new();
    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first()
        && let Some(declarator) = var_decl.declarations.first()
    {
        extract_binding_names4(&declarator.id, &mut vars);
    }
    vars
}

fn format_binding_pattern(pattern: &str) -> String {
    let mut formatted = String::with_capacity("let [] = x".len() + pattern.len());
    formatted.push_str("let [");
    formatted.push_str(pattern);
    formatted.push_str("] = x");
    formatted
}

/// Extract binding names from a binding pattern
pub(crate) fn extract_binding_names(
    pattern: &BindingPattern<'_>,
    names: &mut SmallVec<[CompactString; 3]>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.push(CompactString::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                extract_binding_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                extract_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                extract_binding_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                extract_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            extract_binding_names(&assign.left, names);
        }
    }
}

fn extract_binding_names4(pattern: &BindingPattern<'_>, names: &mut SmallVec<[CompactString; 4]>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.push(CompactString::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                extract_binding_names4(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                extract_binding_names4(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                extract_binding_names4(elem, names);
            }
            if let Some(rest) = &arr.rest {
                extract_binding_names4(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            extract_binding_names4(&assign.left, names);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_v_for_scope_expression;

    #[test]
    fn parse_scope_expression_keeps_destructured_value_pattern() {
        let aliases = parse_v_for_scope_expression("{ id, name: label } in items").unwrap();

        assert_eq!(aliases.value_pattern.as_str(), "{ id, name: label }");
        assert_eq!(
            aliases
                .value_bindings
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            vec!["id", "label"]
        );
        assert!(aliases.key_alias.is_none());
        assert!(aliases.index_alias.is_none());
    }

    #[test]
    fn parse_scope_expression_splits_tuple_around_destructured_value() {
        let aliases =
            parse_v_for_scope_expression("({ id, meta: { slug } }, index) in items").unwrap();

        assert_eq!(aliases.value_pattern.as_str(), "{ id, meta: { slug } }");
        assert_eq!(
            aliases
                .value_bindings
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            vec!["id", "slug"]
        );
        assert_eq!(aliases.key_alias.as_deref(), Some("index"));
        assert!(aliases.index_alias.is_none());
    }
}
