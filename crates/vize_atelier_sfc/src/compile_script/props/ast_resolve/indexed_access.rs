use oxc_allocator::Allocator;
use oxc_ast::ast::{Statement, TSIndexedAccessType, TSSignature, TSType, TSTypeName};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashMap, String, ToCompactString};

use super::runtime_type_combine::combine_runtime_js_types;
use super::{
    collect_props_from_ts_type, finish_resolved_type_reference, literal_values_from_ts_type,
    property_key_name, resolve_type_reference_text, simple_type_name,
    ts_type_to_js_type_from_ast_inner, wrap_type_alias_source,
};

pub(super) fn resolve(
    indexed: &TSIndexedAccessType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> String {
    resolve_inner(indexed, source, interfaces, type_aliases, seen).unwrap_or_else(|| "null".into())
}

fn resolve_inner(
    indexed: &TSIndexedAccessType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<String> {
    let keys = literal_values_from_ts_type(
        &indexed.index_type,
        source,
        interfaces,
        type_aliases,
        None,
        seen,
    )?;
    let mut js_types = Vec::new();
    for key in keys {
        js_types.push(resolve_property_runtime_type(
            &indexed.object_type,
            key.as_str(),
            source,
            interfaces,
            type_aliases,
            seen,
        )?);
    }

    Some(combine_runtime_js_types(js_types))
}

fn resolve_property_runtime_type(
    ts_type: &TSType<'_>,
    key: &str,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<String> {
    match ts_type {
        TSType::TSTypeLiteral(type_lit) => {
            for member in type_lit.members.iter() {
                match member {
                    TSSignature::TSPropertySignature(prop) => {
                        let Some(name) = property_key_name(&prop.key) else {
                            continue;
                        };
                        if name.as_str() != key {
                            continue;
                        }
                        return Some(
                            prop.type_annotation
                                .as_ref()
                                .map(|type_ann| {
                                    ts_type_to_js_type_from_ast_inner(
                                        &type_ann.type_annotation,
                                        source,
                                        interfaces,
                                        type_aliases,
                                        seen,
                                    )
                                })
                                .unwrap_or_else(|| "null".to_compact_string()),
                        );
                    }
                    TSSignature::TSMethodSignature(method) => {
                        let Some(name) = property_key_name(&method.key) else {
                            continue;
                        };
                        if name.as_str() == key {
                            return Some("Function".to_compact_string());
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        TSType::TSParenthesizedType(paren) => resolve_property_runtime_type(
            &paren.type_annotation,
            key,
            source,
            interfaces,
            type_aliases,
            seen,
        ),
        TSType::TSTypeReference(type_ref) => resolve_type_reference_property(
            &type_ref.type_name,
            key,
            interfaces,
            type_aliases,
            seen,
        ),
        _ => resolve_property_runtime_type_by_collecting(
            ts_type,
            key,
            source,
            interfaces,
            type_aliases,
            seen,
        ),
    }
}

fn resolve_type_reference_property(
    type_name: &TSTypeName<'_>,
    key: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<String> {
    let name = simple_type_name(type_name)?;
    let resolved = resolve_type_reference_text(name, interfaces, type_aliases, seen)?;
    if !is_indexable_object_source(&resolved) {
        finish_resolved_type_reference(name, seen);
        return None;
    }

    let resolved_source = wrap_type_alias_source(&resolved);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &resolved_source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        finish_resolved_type_reference(name, seen);
        return None;
    }

    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        finish_resolved_type_reference(name, seen);
        return None;
    };

    let resolved = resolve_property_runtime_type(
        &alias.type_annotation,
        key,
        &resolved_source,
        interfaces,
        type_aliases,
        seen,
    );
    finish_resolved_type_reference(name, seen);
    resolved
}

fn is_indexable_object_source(type_source: &str) -> bool {
    type_source.trim().starts_with('{')
}

fn resolve_property_runtime_type_by_collecting(
    ts_type: &TSType<'_>,
    key: &str,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<String> {
    let mut props = Vec::new();
    if !collect_props_from_ts_type(ts_type, source, interfaces, type_aliases, seen, &mut props) {
        return None;
    }
    let (_, prop) = props.iter().find(|(name, _)| name.as_str() == key)?;
    Some(prop.js_type.clone())
}

#[cfg(test)]
mod tests {
    use crate::compile_script::props::text_resolve::extract_prop_types_from_type_with_context;
    use vize_carton::{FxHashMap, ToCompactString};

    #[test]
    fn resolves_object_literal_alias_runtime_type() {
        let mut type_aliases: FxHashMap<vize_carton::String, vize_carton::String> =
            FxHashMap::default();
        type_aliases.insert(
            "ButtonProps".to_compact_string(),
            "{ color?: 'primary' | 'neutral'; ui?: { base?: string } }".to_compact_string(),
        );

        let props = extract_prop_types_from_type_with_context(
            "{ color?: ButtonProps['color']; ui?: ButtonProps['ui'] }",
            None,
            Some(&type_aliases),
        );

        let color = props
            .iter()
            .find(|(name, _)| name == "color")
            .expect("color prop should be extracted");
        assert_eq!(color.1.js_type.as_str(), "String");
        let ui = props
            .iter()
            .find(|(name, _)| name == "ui")
            .expect("ui prop should be extracted");
        assert_eq!(ui.1.js_type.as_str(), "Object");
    }

    #[test]
    fn does_not_follow_non_literal_alias_runtime_type() {
        let mut type_aliases: FxHashMap<vize_carton::String, vize_carton::String> =
            FxHashMap::default();
        type_aliases.insert(
            "Button".to_compact_string(),
            "ComponentConfig<typeof theme, AppConfig, 'button'>".to_compact_string(),
        );
        type_aliases.insert(
            "ComponentConfig".to_compact_string(),
            "{ variants?: { size?: 'sm' | 'md' }; slots?: { base?: string } }".to_compact_string(),
        );

        let props = extract_prop_types_from_type_with_context(
            "{ size?: Button['variants']['size']; ui?: Button['slots'] }",
            None,
            Some(&type_aliases),
        );

        let size = props
            .iter()
            .find(|(name, _)| name == "size")
            .expect("size prop should be extracted");
        assert_eq!(size.1.js_type.as_str(), "null");
        let ui = props
            .iter()
            .find(|(name, _)| name == "ui")
            .expect("ui prop should be extracted");
        assert_eq!(ui.1.js_type.as_str(), "null");
    }
}
