use oxc_ast::ast::TSType;
use oxc_span::GetSpan;

use crate::macros::SlotsDefinition;
use vize_carton::CompactString;

use super::super::ScriptParseResult;
use super::common::static_property_name;

pub(super) fn extract_slots_from_type(
    result: &mut ScriptParseResult,
    type_params: &oxc_allocator::Vec<'_, TSType<'_>>,
    source: &str,
) {
    for type_param in type_params {
        extract_slots_from_ts_type(result, type_param, source);
    }
}

fn extract_slots_from_ts_type(result: &mut ScriptParseResult, ty: &TSType<'_>, source: &str) {
    match ty {
        TSType::TSTypeLiteral(literal) => {
            for member in &literal.members {
                match member {
                    oxc_ast::ast::TSSignature::TSPropertySignature(property) => {
                        let Some(name) = static_property_name(&property.key) else {
                            continue;
                        };
                        let props_type = property.type_annotation.as_ref().and_then(|annotation| {
                            slot_props_type_from_ts_type(&annotation.type_annotation, source)
                        });
                        result.macros.add_slot(SlotsDefinition {
                            name: CompactString::new(name),
                            props_type,
                        });
                    }
                    oxc_ast::ast::TSSignature::TSMethodSignature(method) => {
                        let Some(name) = static_property_name(&method.key) else {
                            continue;
                        };
                        result.macros.add_slot(SlotsDefinition {
                            name: CompactString::new(name),
                            props_type: first_param_type(&method.params, source),
                        });
                    }
                    _ => {}
                }
            }
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                extract_slots_from_ts_type(result, ty, source);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            extract_slots_from_ts_type(result, &parenthesized.type_annotation, source);
        }
        _ => {}
    }
}

fn slot_props_type_from_ts_type(ty: &TSType<'_>, source: &str) -> Option<CompactString> {
    match ty {
        TSType::TSFunctionType(function) => first_param_type(&function.params, source),
        TSType::TSParenthesizedType(parenthesized) => {
            slot_props_type_from_ts_type(&parenthesized.type_annotation, source)
        }
        TSType::TSUnionType(union) => union
            .types
            .iter()
            .find_map(|ty| slot_props_type_from_ts_type(ty, source)),
        _ => None,
    }
}

fn first_param_type(
    params: &oxc_ast::ast::FormalParameters<'_>,
    source: &str,
) -> Option<CompactString> {
    let annotation = params.items.first()?.type_annotation.as_ref()?;
    let span = annotation.type_annotation.span();
    source
        .get(span.start as usize..span.end as usize)
        .map(str::trim)
        .filter(|text| !text.is_empty() && *text != "()")
        .map(CompactString::new)
}
