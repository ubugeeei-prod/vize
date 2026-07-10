//! Emit name extraction from `defineEmits` type definitions.

use oxc_allocator::Allocator;
use oxc_ast::ast::{PropertyKey, Statement, TSLiteral, TSSignature, TSType, TSTypeLiteral};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

const TYPE_ALIAS_PREFIX: &str = "type __VizeEmits = ";

/// Extract emit names from TypeScript type definition.
pub fn extract_emit_names_from_type(type_args: &str) -> Vec<String> {
    let source = wrap_type_alias_source(type_args);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }

    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        return Vec::new();
    };

    let mut emits = Vec::new();
    collect_emit_names_from_ts_type(&alias.type_annotation, &mut emits);
    emits
}

fn wrap_type_alias_source(type_args: &str) -> String {
    let trimmed = type_args.trim();
    let mut source = String::with_capacity(TYPE_ALIAS_PREFIX.len() + trimmed.len() + 1);
    source.push_str(TYPE_ALIAS_PREFIX);
    source.push_str(trimmed);
    source.push(';');
    source
}

fn collect_emit_names_from_ts_type(ts_type: &TSType<'_>, emits: &mut Vec<String>) {
    match ts_type {
        TSType::TSFunctionType(function) => {
            if let Some(annotation) = function
                .params
                .items
                .first()
                .and_then(|param| param.type_annotation.as_ref())
            {
                collect_literal_event_names(&annotation.type_annotation, emits);
            }
        }
        TSType::TSTypeLiteral(literal) => collect_emit_names_from_type_literal(literal, emits),
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_emit_names_from_ts_type(ty, emits);
            }
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                collect_emit_names_from_ts_type(ty, emits);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            collect_emit_names_from_ts_type(&parenthesized.type_annotation, emits);
        }
        _ => {}
    }
}

fn collect_emit_names_from_type_literal(type_lit: &TSTypeLiteral<'_>, emits: &mut Vec<String>) {
    let mut has_property = false;

    for member in &type_lit.members {
        if let TSSignature::TSPropertySignature(property) = member
            && let Some(name) = property_key_name(&property.key)
        {
            has_property = true;
            emits.push(name);
        }
    }

    if has_property {
        return;
    }

    for member in &type_lit.members {
        if let TSSignature::TSCallSignatureDeclaration(call) = member
            && let Some(annotation) = call
                .params
                .items
                .first()
                .and_then(|param| param.type_annotation.as_ref())
        {
            collect_literal_event_names(&annotation.type_annotation, emits);
        }
    }
}

fn collect_literal_event_names(ts_type: &TSType<'_>, emits: &mut Vec<String>) {
    match ts_type {
        TSType::TSLiteralType(literal_type) => match &literal_type.literal {
            TSLiteral::StringLiteral(literal) => emits.push(String::from(literal.value.as_str())),
            TSLiteral::NumericLiteral(literal) => emits.push(literal.value.to_compact_string()),
            _ => {}
        },
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_literal_event_names(ty, emits);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            collect_literal_event_names(&parenthesized.type_annotation, emits);
        }
        _ => {}
    }
}

fn property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(String::from(identifier.name.as_str())),
        PropertyKey::StringLiteral(literal) => Some(String::from(literal.value.as_str())),
        PropertyKey::NumericLiteral(literal) => Some(literal.value.to_compact_string()),
        _ => None,
    }
}
