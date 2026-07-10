use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, SmallVec, String, profile};

use super::VForScopeAliases;

/// Parse complex v-for alias using OXC
pub(super) fn parse_v_for_with_oxc(
    alias: &str,
    source: CompactString,
) -> (SmallVec<[CompactString; 3]>, CompactString) {
    let mut buffer = [0u8; 256];
    let prefix = b"let [";
    let suffix = b"] = x";

    let inner = tuple_alias_inner(alias.trim_start_matches("const ").trim());

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

pub(super) fn parse_v_for_scope_aliases(
    alias: &str,
    source: CompactString,
) -> Option<VForScopeAliases> {
    let inner = tuple_alias_inner(alias);
    let pattern_str = format_binding_pattern(inner);
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = profile!(
        "croquis.helpers.v_for.scope_alias_parse",
        Parser::new(&allocator, &pattern_str, source_type).parse()
    );
    if !ret.errors.is_empty() {
        return None;
    }

    let declarator = ret.program.body.first().and_then(|statement| {
        if let oxc_ast::ast::Statement::VariableDeclaration(var_decl) = statement {
            var_decl.declarations.first()
        } else {
            None
        }
    })?;

    let BindingPattern::ArrayPattern(root) = &declarator.id else {
        return None;
    };

    let value_pattern = root.elements.first().and_then(Option::as_ref)?;
    let mut value_bindings = SmallVec::new();
    extract_binding_names4(value_pattern, &mut value_bindings);
    if value_bindings.is_empty() {
        return None;
    }

    Some(VForScopeAliases {
        value_pattern: binding_pattern_source(value_pattern, &pattern_str),
        value_bindings,
        key_alias: root
            .elements
            .get(1)
            .and_then(Option::as_ref)
            .and_then(binding_identifier_name),
        index_alias: root
            .elements
            .get(2)
            .and_then(Option::as_ref)
            .and_then(binding_identifier_name),
        source,
    })
}

fn binding_pattern_source(pattern: &BindingPattern<'_>, source: &str) -> CompactString {
    let span = pattern.span();
    CompactString::new(&source[span.start as usize..span.end as usize])
}

fn binding_identifier_name(pattern: &BindingPattern<'_>) -> Option<CompactString> {
    if let BindingPattern::BindingIdentifier(id) = pattern {
        Some(CompactString::new(id.name.as_str()))
    } else {
        None
    }
}

fn tuple_alias_inner(alias: &str) -> &str {
    let alias = alias.trim();
    if alias.starts_with('(') && alias.ends_with(')') {
        alias[1..alias.len() - 1].trim()
    } else {
        alias
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
    if !ret.errors.is_empty() {
        return (SmallVec::new(), source);
    }

    let mut vars = SmallVec::new();

    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first()
        && let Some(declarator) = var_decl.declarations.first()
    {
        extract_binding_names(&declarator.id, &mut vars);
    }

    (vars, source)
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
