//! Same-file runtime types for `defineModel<T>` and `defineProps<T>()`.
//!
//! This is the subset of eslint-plugin-vue's `inferRuntimeType` that the
//! reported defaults need: keywords, literals, unions, and a same-file alias
//! or interface. A name that is not a native constructor is dropped.

use oxc_ast::ast::{Declaration, Program, Statement, TSLiteral, TSSignature, TSType, TSTypeName};
use vize_l0::CompactString;

use super::key_name;

pub(super) fn collect_type_props(
    program: &Program<'_>,
    ty: &TSType<'_>,
) -> Vec<(CompactString, Vec<&'static str>)> {
    let mut props = Vec::new();
    let mut seen = Vec::new();
    collect_props(program, ty, 0, &mut seen, &mut props);
    props
}

pub(super) fn infer_types(program: &Program<'_>, ty: &TSType<'_>) -> Vec<&'static str> {
    let mut types = Vec::new();
    infer_into(program, ty, 0, &mut Vec::new(), &mut types);
    types.retain(|ty| super::super::NATIVE_TYPES.contains(ty));
    types
}

fn collect_props<'a>(
    program: &'a Program<'a>,
    ty: &'a TSType<'a>,
    depth: usize,
    seen: &mut Vec<&'a str>,
    props: &mut Vec<(CompactString, Vec<&'static str>)>,
) {
    if depth >= 8 {
        return;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => collect_members(&literal.members, program, props),
        TSType::TSParenthesizedType(parenthesized) => {
            collect_props(
                program,
                &parenthesized.type_annotation,
                depth + 1,
                seen,
                props,
            );
        }
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_props(program, ty, depth + 1, seen, props);
            }
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                collect_props(program, ty, depth + 1, seen, props);
            }
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return;
            };
            let name = identifier.name.as_str();
            if seen.contains(&name) {
                return;
            }
            seen.push(name);
            resolve_named_type(program, name, depth + 1, seen, props);
            seen.pop();
        }
        _ => {}
    }
}

fn collect_members(
    members: &[TSSignature<'_>],
    program: &Program<'_>,
    props: &mut Vec<(CompactString, Vec<&'static str>)>,
) {
    for member in members {
        match member {
            TSSignature::TSPropertySignature(property) => {
                let Some(name) = key_name(&property.key) else {
                    continue;
                };
                let types = property
                    .type_annotation
                    .as_ref()
                    .map(|annotation| infer_types(program, &annotation.type_annotation))
                    .unwrap_or_default();
                props.push((CompactString::new(name), types));
            }
            TSSignature::TSMethodSignature(method) => {
                let Some(name) = key_name(&method.key) else {
                    continue;
                };
                props.push((CompactString::new(name), vec!["Function"]));
            }
            _ => {}
        }
    }
}

fn resolve_named_type<'a>(
    program: &'a Program<'a>,
    name: &str,
    depth: usize,
    seen: &mut Vec<&'a str>,
    props: &mut Vec<(CompactString, Vec<&'static str>)>,
) {
    for statement in &program.body {
        let Some(declaration) = statement_declaration(statement) else {
            continue;
        };
        match declaration {
            Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                collect_members(&interface.body.body, program, props);
                return;
            }
            Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                collect_props(program, &alias.type_annotation, depth, seen, props);
                return;
            }
            _ => {}
        }
    }
}

fn infer_into<'a>(
    program: &'a Program<'a>,
    ty: &'a TSType<'a>,
    depth: usize,
    seen: &mut Vec<&'a str>,
    types: &mut Vec<&'static str>,
) {
    if depth >= 8 {
        return;
    }
    match ty {
        TSType::TSStringKeyword(_) | TSType::TSTemplateLiteralType(_) => push(types, "String"),
        TSType::TSNumberKeyword(_) => push(types, "Number"),
        TSType::TSBooleanKeyword(_) => push(types, "Boolean"),
        TSType::TSBigIntKeyword(_) => push(types, "BigInt"),
        TSType::TSSymbolKeyword(_) => push(types, "Symbol"),
        TSType::TSObjectKeyword(_) | TSType::TSTypeLiteral(_) => push(types, "Object"),
        TSType::TSFunctionType(_) | TSType::TSConstructorType(_) => push(types, "Function"),
        TSType::TSArrayType(_) | TSType::TSTupleType(_) => push(types, "Array"),
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(_) | TSLiteral::TemplateLiteral(_) => push(types, "String"),
            TSLiteral::NumericLiteral(_) | TSLiteral::BigIntLiteral(_) => push(types, "Number"),
            TSLiteral::BooleanLiteral(_) => push(types, "Boolean"),
            _ => {}
        },
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                infer_into(program, ty, depth + 1, seen, types);
            }
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                infer_into(program, ty, depth + 1, seen, types);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            infer_into(
                program,
                &parenthesized.type_annotation,
                depth + 1,
                seen,
                types,
            );
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return;
            };
            match identifier.name.as_str() {
                "Array" | "ReadonlyArray" => push(types, "Array"),
                "Function" => push(types, "Function"),
                "Object" | "Record" | "Partial" | "Readonly" | "Pick" | "Omit" | "Required" => {
                    push(types, "Object");
                }
                "String" => push(types, "String"),
                "Number" => push(types, "Number"),
                "Boolean" => push(types, "Boolean"),
                "Symbol" => push(types, "Symbol"),
                "BigInt" => push(types, "BigInt"),
                name => {
                    if seen.contains(&name) {
                        return;
                    }
                    seen.push(name);
                    if let Some(alias) = named_alias(program, name) {
                        infer_into(program, alias, depth + 1, seen, types);
                    }
                    seen.pop();
                }
            }
        }
        _ => {}
    }
}

fn named_alias<'a>(program: &'a Program<'a>, name: &str) -> Option<&'a TSType<'a>> {
    for statement in &program.body {
        let Some(declaration) = statement_declaration(statement) else {
            continue;
        };
        let Declaration::TSTypeAliasDeclaration(alias) = declaration else {
            continue;
        };
        if alias.id.name == name {
            return Some(&alias.type_annotation);
        }
    }
    None
}

fn statement_declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

fn push(types: &mut Vec<&'static str>, name: &'static str) {
    if !types.contains(&name) {
        types.push(name);
    }
}
