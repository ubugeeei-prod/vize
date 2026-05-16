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
use vize_carton::{profile, smallvec, CompactString, SmallVec};

use super::is_valid_identifier_fast;

/// Parse v-for expression into variables and source
#[inline]
pub fn parse_v_for_expression(expr: &str) -> (SmallVec<[CompactString; 3]>, CompactString) {
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

    let Some(pos) = split_pos else {
        return (smallvec![], CompactString::new(expr.trim()));
    };

    let alias_part = expr[..pos].trim();
    let source_part = expr[pos + 4..].trim();
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

    // SAFETY: we only copy ASCII bytes
    let pattern_str = unsafe { std::str::from_utf8_unchecked(&buffer[..total_len]) };
    profile!(
        "croquis.helpers.v_for.parse_pattern",
        parse_v_for_pattern(pattern_str, source)
    )
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

    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first() {
        if let Some(declarator) = var_decl.declarations.first() {
            extract_binding_names(&declarator.id, &mut vars);
        }
    }

    (vars, source)
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
