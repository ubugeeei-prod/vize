//! AST-based prop type resolution.
//!
//! Parses TypeScript type definitions and walks the resulting AST to collect
//! prop types, resolving interface/type-alias references, mapped types, and
//! literal unions into runtime constructors.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, PropertyKey, Statement, TSLiteral, TSMappedTypeModifierOperator, TSSignature,
    TSType, TSTypeLiteral, TSTypeName, TSTypeOperatorOperator,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::FxHashMap;
use vize_carton::{String, ToCompactString};

use super::runtime_type::ts_type_to_js_type;
use super::types::PropTypeInfo;

pub(super) const TYPE_ALIAS_PREFIX: &str = "type __VizeProps = ";

pub(super) fn extract_prop_types_from_ast(
    type_args: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
) -> Option<Vec<(String, PropTypeInfo)>> {
    let source = wrap_type_alias_source(type_args);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return None;
    }

    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        return None;
    };

    let mut props = Vec::new();
    let mut seen = Vec::new();
    if collect_props_from_ts_type(
        &alias.type_annotation,
        &source,
        interfaces,
        type_aliases,
        &mut seen,
        &mut props,
    ) {
        Some(props)
    } else {
        None
    }
}

pub(super) fn wrap_type_alias_source(type_args: &str) -> String {
    let trimmed = type_args.trim();
    let mut source = String::with_capacity(TYPE_ALIAS_PREFIX.len() + trimmed.len() + 1);
    source.push_str(TYPE_ALIAS_PREFIX);
    source.push_str(trimmed);
    source.push(';');
    source
}

fn collect_props_from_ts_type(
    ts_type: &TSType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
    props: &mut Vec<(String, PropTypeInfo)>,
) -> bool {
    match ts_type {
        TSType::TSTypeLiteral(type_lit) => {
            collect_props_from_ts_type_literal(type_lit, source, interfaces, type_aliases, props);
            true
        }
        TSType::TSMappedType(mapped) => {
            collect_props_from_mapped_type(mapped, source, interfaces, type_aliases, seen, props);
            true
        }
        TSType::TSIntersectionType(intersection) => {
            let mut handled = false;
            for ty in intersection.types.iter() {
                handled |=
                    collect_props_from_ts_type(ty, source, interfaces, type_aliases, seen, props);
            }
            handled
        }
        TSType::TSUnionType(union) => {
            let mut handled = false;
            for ty in union.types.iter() {
                handled |=
                    collect_props_from_ts_type(ty, source, interfaces, type_aliases, seen, props);
            }
            handled
        }
        TSType::TSConditionalType(conditional) => {
            let true_handled = collect_props_from_ts_type(
                &conditional.true_type,
                source,
                interfaces,
                type_aliases,
                seen,
                props,
            );
            let false_handled = collect_props_from_ts_type(
                &conditional.false_type,
                source,
                interfaces,
                type_aliases,
                seen,
                props,
            );
            true_handled || false_handled
        }
        TSType::TSParenthesizedType(paren) => collect_props_from_ts_type(
            &paren.type_annotation,
            source,
            interfaces,
            type_aliases,
            seen,
            props,
        ),
        TSType::TSTypeReference(type_ref) => {
            let Some(name) = simple_type_name(&type_ref.type_name) else {
                return false;
            };
            let Some(resolved) = resolve_type_reference_text(name, interfaces, type_aliases, seen)
            else {
                return false;
            };
            let resolved_source = wrap_type_alias_source(&resolved);
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &resolved_source, SourceType::ts()).parse();
            if parsed.panicked || !parsed.errors.is_empty() {
                finish_resolved_type_reference(name, seen);
                return false;
            }
            let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
                finish_resolved_type_reference(name, seen);
                return false;
            };
            let handled = collect_props_from_ts_type(
                &alias.type_annotation,
                &resolved_source,
                interfaces,
                type_aliases,
                seen,
                props,
            );
            finish_resolved_type_reference(name, seen);
            handled
        }
        _ => false,
    }
}

fn collect_props_from_ts_type_literal(
    type_lit: &TSTypeLiteral<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    props: &mut Vec<(String, PropTypeInfo)>,
) {
    for member in type_lit.members.iter() {
        match member {
            TSSignature::TSPropertySignature(prop) => {
                let Some(name) = property_key_name(&prop.key) else {
                    continue;
                };
                let (ts_type, js_type, nullable) = if let Some(type_ann) = &prop.type_annotation {
                    let ts_type = source_for_span(source, type_ann.type_annotation.span())
                        .unwrap_or("unknown")
                        .trim()
                        .to_compact_string();
                    let js_type = ts_type_to_js_type_from_ast(
                        &type_ann.type_annotation,
                        source,
                        interfaces,
                        type_aliases,
                    );
                    let nullable = type_includes_top_level_null_from_ast(&type_ann.type_annotation);
                    (Some(ts_type), js_type, nullable)
                } else {
                    (None, "null".to_compact_string(), false)
                };
                push_prop_type_info(
                    props,
                    name,
                    PropTypeInfo {
                        js_type,
                        ts_type,
                        optional: prop.optional,
                        nullable,
                    },
                );
            }
            TSSignature::TSMethodSignature(method) => {
                let Some(name) = property_key_name(&method.key) else {
                    continue;
                };
                push_prop_type_info(
                    props,
                    name,
                    PropTypeInfo {
                        js_type: "Function".to_compact_string(),
                        ts_type: None,
                        optional: method.optional,
                        nullable: false,
                    },
                );
            }
            _ => {}
        }
    }
}

fn collect_props_from_mapped_type(
    mapped: &oxc_ast::ast::TSMappedType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
    props: &mut Vec<(String, PropTypeInfo)>,
) {
    let Some(keys) = literal_values_from_ts_type(
        &mapped.constraint,
        source,
        interfaces,
        type_aliases,
        None,
        seen,
    ) else {
        return;
    };

    let key_name = mapped.key.name.as_str();
    for key in keys {
        let prop_name = if let Some(name_type) = &mapped.name_type {
            let mut values = literal_values_from_ts_type(
                name_type,
                source,
                interfaces,
                type_aliases,
                Some((key_name, key.as_str())),
                seen,
            )
            .unwrap_or_default();
            if values.is_empty() {
                continue;
            }
            values.remove(0)
        } else {
            key
        };

        let (ts_type, js_type, nullable) = if let Some(type_ann) = &mapped.type_annotation {
            let ts_type = source_for_span(source, type_ann.span())
                .unwrap_or("unknown")
                .trim()
                .to_compact_string();
            let js_type = ts_type_to_js_type_from_ast(type_ann, source, interfaces, type_aliases);
            let nullable = type_includes_top_level_null_from_ast(type_ann);
            (Some(ts_type), js_type, nullable)
        } else {
            (None, "null".to_compact_string(), false)
        };

        push_prop_type_info(
            props,
            prop_name,
            PropTypeInfo {
                js_type,
                ts_type,
                optional: matches!(
                    mapped.optional,
                    Some(TSMappedTypeModifierOperator::True | TSMappedTypeModifierOperator::Plus)
                ),
                nullable,
            },
        );
    }
}

fn push_prop_type_info(props: &mut Vec<(String, PropTypeInfo)>, name: String, info: PropTypeInfo) {
    if !props
        .iter()
        .any(|(existing, _)| existing.as_str() == name.as_str())
    {
        props.push((name, info));
    }
}

fn ts_type_to_js_type_from_ast(
    ts_type: &TSType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
) -> String {
    let mut seen = Vec::new();
    ts_type_to_js_type_from_ast_inner(ts_type, source, interfaces, type_aliases, &mut seen)
}

/// Inner implementation of [`ts_type_to_js_type_from_ast`] that threads a
/// `seen` stack of type-alias names currently being resolved. Self-referential
/// or mutually recursive type aliases (e.g. `type NestedItem<T> = T extends
/// Array<infer I> ? NestedItem<I> : T`, or a recursive `DotPathKeys`) otherwise
/// re-parse and re-enter the same alias body forever, overflowing the native
/// stack and aborting the whole `vize build`. The guard stops following an
/// alias reference once its name is already on the stack and falls back to a
/// safe `null` runtime type, mirroring the cycle protection already used by
/// `collect_props_from_ts_type` / `resolve_type_reference_text`.
fn ts_type_to_js_type_from_ast_inner(
    ts_type: &TSType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> String {
    match ts_type {
        TSType::TSStringKeyword(_) => "String".to_compact_string(),
        TSType::TSNumberKeyword(_) => "Number".to_compact_string(),
        TSType::TSBooleanKeyword(_) => "Boolean".to_compact_string(),
        TSType::TSSymbolKeyword(_) => "Symbol".to_compact_string(),
        TSType::TSBigIntKeyword(_) => "BigInt".to_compact_string(),
        TSType::TSObjectKeyword(_) | TSType::TSTypeLiteral(_) | TSType::TSMappedType(_) => {
            "Object".to_compact_string()
        }
        TSType::TSArrayType(_) | TSType::TSTupleType(_) => "Array".to_compact_string(),
        TSType::TSFunctionType(_) | TSType::TSConstructorType(_) => "Function".to_compact_string(),
        TSType::TSLiteralType(lit) => js_type_for_ts_literal(&lit.literal),
        TSType::TSUnionType(union) => {
            combine_runtime_js_types(union.types.iter().filter_map(|ty| {
                if matches!(
                    ty,
                    TSType::TSUndefinedKeyword(_)
                        | TSType::TSNullKeyword(_)
                        | TSType::TSNeverKeyword(_)
                ) {
                    None
                } else {
                    Some(ts_type_to_js_type_from_ast_inner(
                        ty,
                        source,
                        interfaces,
                        type_aliases,
                        seen,
                    ))
                }
            }))
        }
        TSType::TSConditionalType(conditional) => combine_runtime_js_types([
            ts_type_to_js_type_from_ast_inner(
                &conditional.true_type,
                source,
                interfaces,
                type_aliases,
                seen,
            ),
            ts_type_to_js_type_from_ast_inner(
                &conditional.false_type,
                source,
                interfaces,
                type_aliases,
                seen,
            ),
        ]),
        TSType::TSIntersectionType(intersection) => {
            if intersection.types.iter().any(|ty| {
                matches!(
                    ty,
                    TSType::TSTypeLiteral(_) | TSType::TSMappedType(_) | TSType::TSTypeReference(_)
                )
            }) {
                "Object".to_compact_string()
            } else {
                "null".to_compact_string()
            }
        }
        TSType::TSParenthesizedType(paren) => ts_type_to_js_type_from_ast_inner(
            &paren.type_annotation,
            source,
            interfaces,
            type_aliases,
            seen,
        ),
        TSType::TSTypeOperatorType(op) => match op.operator {
            TSTypeOperatorOperator::Readonly => ts_type_to_js_type_from_ast_inner(
                &op.type_annotation,
                source,
                interfaces,
                type_aliases,
                seen,
            ),
            TSTypeOperatorOperator::Keyof => "String".to_compact_string(),
            TSTypeOperatorOperator::Unique => "null".to_compact_string(),
        },
        TSType::TSTypeReference(type_ref) => {
            let source_type = source_for_span(source, type_ref.span()).unwrap_or_default();
            let js_type = ts_type_to_js_type(source_type);
            if js_type != "null" {
                return js_type;
            }

            let Some(name) = simple_type_name(&type_ref.type_name) else {
                return "null".to_compact_string();
            };
            if interfaces.is_some_and(|interfaces| interfaces.contains_key(name)) {
                return "Object".to_compact_string();
            }
            if let Some(type_aliases) = type_aliases
                && let Some(alias) = type_aliases.get(name)
            {
                // Stop following recursive / mutually recursive aliases. Without
                // this guard a self-referential alias body re-parses and
                // re-enters here forever, overflowing the stack. (nuxt-ui
                // `CheckboxGroup`/`RadioGroup` hit this via `NestedItem` /
                // `DotPathKeys`.)
                if seen.iter().any(|seen_name| seen_name == name) {
                    return "null".to_compact_string();
                }
                let alias_source = wrap_type_alias_source(alias);
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, &alias_source, SourceType::ts()).parse();
                if !parsed.panicked
                    && parsed.errors.is_empty()
                    && let Some(Statement::TSTypeAliasDeclaration(alias_decl)) =
                        parsed.program.body.first()
                {
                    seen.push(name.to_compact_string());
                    let resolved = ts_type_to_js_type_from_ast_inner(
                        &alias_decl.type_annotation,
                        &alias_source,
                        interfaces,
                        Some(type_aliases),
                        seen,
                    );
                    seen.pop();
                    return resolved;
                }
            }
            "null".to_compact_string()
        }
        TSType::TSIndexedAccessType(_) => "null".to_compact_string(),
        _ => "null".to_compact_string(),
    }
}

fn js_type_for_ts_literal(literal: &TSLiteral<'_>) -> String {
    match literal {
        TSLiteral::StringLiteral(_) | TSLiteral::TemplateLiteral(_) => "String".to_compact_string(),
        TSLiteral::NumericLiteral(_) => "Number".to_compact_string(),
        TSLiteral::BigIntLiteral(_) => "BigInt".to_compact_string(),
        TSLiteral::BooleanLiteral(_) => "Boolean".to_compact_string(),
        TSLiteral::UnaryExpression(unary) => {
            if matches!(&unary.argument, Expression::NumericLiteral(_)) {
                "Number".to_compact_string()
            } else {
                "null".to_compact_string()
            }
        }
    }
}

fn combine_runtime_js_types(types: impl IntoIterator<Item = String>) -> String {
    let mut js_types: Vec<String> = Vec::new();
    for js_type in types {
        if js_type == "null" {
            return js_type;
        }
        if !js_types.contains(&js_type) {
            js_types.push(js_type);
        }
    }

    match js_types.len() {
        0 => "null".to_compact_string(),
        1 => js_types.pop().unwrap_or_else(|| "null".to_compact_string()),
        _ => {
            let joined = js_types.join(", ");
            let mut result = String::with_capacity(joined.len() + 2);
            result.push('[');
            result.push_str(&joined);
            result.push(']');
            result
        }
    }
}

fn type_includes_top_level_null_from_ast(ts_type: &TSType<'_>) -> bool {
    match ts_type {
        TSType::TSNullKeyword(_) => true,
        TSType::TSUnionType(union) => union
            .types
            .iter()
            .any(|ty| matches!(ty, TSType::TSNullKeyword(_))),
        TSType::TSParenthesizedType(paren) => {
            type_includes_top_level_null_from_ast(&paren.type_annotation)
        }
        _ => false,
    }
}

fn literal_values_from_ts_type(
    ts_type: &TSType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    mapped_key: Option<(&str, &str)>,
    seen: &mut Vec<String>,
) -> Option<Vec<String>> {
    match ts_type {
        TSType::TSLiteralType(lit) => literal_values_from_literal(&lit.literal),
        TSType::TSUnionType(union) => {
            let mut values = Vec::new();
            for ty in union.types.iter() {
                values.extend(literal_values_from_ts_type(
                    ty,
                    source,
                    interfaces,
                    type_aliases,
                    mapped_key,
                    seen,
                )?);
            }
            Some(values)
        }
        TSType::TSParenthesizedType(paren) => literal_values_from_ts_type(
            &paren.type_annotation,
            source,
            interfaces,
            type_aliases,
            mapped_key,
            seen,
        ),
        TSType::TSTemplateLiteralType(template) => literal_values_from_template_type(
            template,
            source,
            interfaces,
            type_aliases,
            mapped_key,
            seen,
        ),
        TSType::TSTypeOperatorType(op) if op.operator == TSTypeOperatorOperator::Keyof => {
            literal_values_from_keyof_type(
                &op.type_annotation,
                source,
                interfaces,
                type_aliases,
                seen,
            )
        }
        TSType::TSTypeReference(type_ref) => {
            let name = simple_type_name(&type_ref.type_name)?;
            if let Some((mapped_name, mapped_value)) = mapped_key
                && name == mapped_name
            {
                return Some(vec![mapped_value.to_compact_string()]);
            }
            let resolved = resolve_type_reference_text(name, interfaces, type_aliases, seen)?;
            let values = literal_values_from_type_text(
                &resolved,
                interfaces,
                type_aliases,
                mapped_key,
                seen,
            );
            finish_resolved_type_reference(name, seen);
            values
        }
        _ => None,
    }
}

fn literal_values_from_literal(literal: &TSLiteral<'_>) -> Option<Vec<String>> {
    match literal {
        TSLiteral::StringLiteral(lit) => Some(vec![lit.value.to_compact_string()]),
        TSLiteral::NumericLiteral(lit) => Some(vec![lit.value.to_compact_string()]),
        TSLiteral::TemplateLiteral(template) => {
            literal_value_from_template_literal(template).map(|value| vec![value])
        }
        _ => None,
    }
}

fn literal_values_from_template_type(
    template: &oxc_ast::ast::TSTemplateLiteralType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    mapped_key: Option<(&str, &str)>,
    seen: &mut Vec<String>,
) -> Option<Vec<String>> {
    let mut values = vec![String::default()];
    for (idx, quasi) in template.quasis.iter().enumerate() {
        let text = quasi
            .value
            .cooked
            .as_ref()
            .unwrap_or(&quasi.value.raw)
            .as_str();
        for value in &mut values {
            value.push_str(text);
        }
        let Some(ty) = template.types.get(idx) else {
            continue;
        };
        let parts =
            literal_values_from_ts_type(ty, source, interfaces, type_aliases, mapped_key, seen)?;
        let mut expanded = Vec::new();
        for prefix in &values {
            for part in &parts {
                let mut value = prefix.clone();
                value.push_str(part);
                expanded.push(value);
            }
        }
        values = expanded;
    }
    Some(values)
}

fn literal_values_from_keyof_type(
    ts_type: &TSType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<Vec<String>> {
    let mut props = Vec::new();
    if collect_props_from_ts_type(ts_type, source, interfaces, type_aliases, seen, &mut props) {
        Some(props.into_iter().map(|(name, _)| name).collect())
    } else {
        None
    }
}

fn literal_values_from_type_text(
    type_text: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    mapped_key: Option<(&str, &str)>,
    seen: &mut Vec<String>,
) -> Option<Vec<String>> {
    let source = wrap_type_alias_source(type_text);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return None;
    }
    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        return None;
    };
    literal_values_from_ts_type(
        &alias.type_annotation,
        &source,
        interfaces,
        type_aliases,
        mapped_key,
        seen,
    )
}

fn property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.to_compact_string()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.to_compact_string()),
        PropertyKey::NumericLiteral(lit) => Some(lit.value.to_compact_string()),
        PropertyKey::TemplateLiteral(template) => literal_value_from_template_literal(template),
        _ => None,
    }
}

fn literal_value_from_template_literal(
    template: &oxc_ast::ast::TemplateLiteral<'_>,
) -> Option<String> {
    if !template.expressions.is_empty() {
        return None;
    }
    let mut value = String::default();
    for quasi in template.quasis.iter() {
        value.push_str(
            quasi
                .value
                .cooked
                .as_ref()
                .unwrap_or(&quasi.value.raw)
                .as_str(),
        );
    }
    Some(value)
}

fn source_for_span(source: &str, span: oxc_span::Span) -> Option<&str> {
    source.get(span.start as usize..span.end as usize)
}

fn simple_type_name<'a>(type_name: &'a TSTypeName<'_>) -> Option<&'a str> {
    match type_name {
        TSTypeName::IdentifierReference(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn resolve_type_reference_text(
    name: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<String> {
    if seen.iter().any(|seen_name| seen_name == name) {
        return None;
    }

    let resolved = if let Some(type_aliases) = type_aliases
        && let Some(alias) = type_aliases.get(name)
    {
        Some(alias.clone())
    } else if let Some(interfaces) = interfaces
        && let Some(body) = interfaces.get(name)
    {
        let mut source = String::with_capacity(body.len() + 4);
        source.push_str("{ ");
        source.push_str(body);
        source.push_str(" }");
        Some(source)
    } else {
        None
    }?;

    seen.push(name.to_compact_string());
    Some(resolved)
}

fn finish_resolved_type_reference(name: &str, seen: &mut Vec<String>) {
    if seen.last().is_some_and(|seen_name| seen_name == name) {
        seen.pop();
    }
}
