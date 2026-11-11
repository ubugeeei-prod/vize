//! Prop names from a type literal, a same-file interface, or a type alias.

use oxc_ast::ast::{Declaration, Program, PropertyKey, Statement, TSSignature, TSType, TSTypeName};
use oxc_span::{GetSpan, Span};
use vize_l0::String;

pub(super) fn props_from_type(
    program: &Program<'_>,
    ty: &TSType<'_>,
    depth: u8,
) -> Vec<(String, Span)> {
    if depth == 0 {
        return Vec::new();
    }
    match ty {
        TSType::TSTypeLiteral(literal) => members(&literal.members),
        TSType::TSParenthesizedType(inner) => {
            props_from_type(program, &inner.type_annotation, depth)
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return Vec::new();
            };
            lookup(program, identifier.name.as_str(), depth - 1)
        }
        _ => Vec::new(),
    }
}

fn members(signatures: &[TSSignature<'_>]) -> Vec<(String, Span)> {
    let mut props = Vec::new();
    for signature in signatures {
        let key = match signature {
            TSSignature::TSPropertySignature(signature) => &signature.key,
            TSSignature::TSMethodSignature(signature) => &signature.key,
            _ => continue,
        };
        if let Some(prop) = named(key) {
            props.push(prop);
        }
    }
    props
}

fn lookup(program: &Program<'_>, name: &str, depth: u8) -> Vec<(String, Span)> {
    for statement in &program.body {
        if let Some(props) = declaration_props(program, statement, name, depth) {
            return props;
        }
    }
    Vec::new()
}

fn declaration_props(
    program: &Program<'_>,
    statement: &Statement<'_>,
    name: &str,
    depth: u8,
) -> Option<Vec<(String, Span)>> {
    match statement {
        Statement::TSInterfaceDeclaration(interface) if interface.id.name == name => {
            Some(members(&interface.body.body))
        }
        Statement::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
            Some(props_from_type(program, &alias.type_annotation, depth))
        }
        Statement::ExportNamedDeclaration(export) => {
            export
                .declaration
                .as_ref()
                .and_then(|declaration| match declaration {
                    Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                        Some(members(&interface.body.body))
                    }
                    Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                        Some(props_from_type(program, &alias.type_annotation, depth))
                    }
                    _ => None,
                })
        }
        _ => None,
    }
}

pub(super) fn named(key: &PropertyKey<'_>) -> Option<(String, Span)> {
    let name = match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str(),
        PropertyKey::StringLiteral(literal) => literal.value.as_str(),
        _ => return None,
    };
    Some((String::from(name), key.span()))
}
