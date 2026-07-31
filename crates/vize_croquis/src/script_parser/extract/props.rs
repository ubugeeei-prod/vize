use oxc_ast::ast::{
    Argument, ArrayExpression, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    TSSignature, TSType,
};
use oxc_span::GetSpan;

use crate::macros::PropDefinition;
use vize_carton::{CompactString, String};
use vize_relief::BindingType;

use super::super::ScriptParseResult;
use super::props_type::runtime_prop_type_from_ts_type;

pub fn extract_props_from_type(
    result: &mut ScriptParseResult,
    type_params: &oxc_allocator::Vec<'_, TSType<'_>>,
    source: &str,
) {
    for tp in type_params.iter() {
        let type_source = tp.span().source_text(source);
        for prop in result.types.extract_properties(type_source) {
            let declaration = type_prop_declaration(tp, prop.name.as_str());
            let definition = PropDefinition {
                name: prop.name.clone(),
                required: !prop.optional,
                prop_type: prop.prop_type,
                default_value: None,
            };
            if let Some((start, end)) = declaration {
                result
                    .macros
                    .add_prop_with_declaration(definition, start, end);
            } else {
                result.macros.add_prop(definition);
            }
            result.bindings.add(prop.name.as_str(), BindingType::Props);
        }
    }
}

pub fn extract_props_from_runtime(
    result: &mut ScriptParseResult,
    arg: &Argument<'_>,
    source: &str,
) {
    match arg {
        Argument::ArrayExpression(arr) => extract_props_from_array(result, arr),

        Argument::ObjectExpression(obj) => {
            extract_props_from_object(result, obj, source);
        }

        Argument::TSAsExpression(ts_as) => {
            extract_props_from_runtime_expression(result, &ts_as.expression, source);
        }
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            extract_props_from_runtime_expression(result, &ts_satisfies.expression, source);
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            extract_props_from_runtime_expression(result, &ts_non_null.expression, source);
        }
        Argument::ParenthesizedExpression(paren) => {
            extract_props_from_runtime_expression(result, &paren.expression, source);
        }

        _ => {}
    }
}

fn extract_props_from_runtime_expression(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) {
    match expr {
        Expression::ArrayExpression(arr) => extract_props_from_array(result, arr),
        Expression::ObjectExpression(obj) => extract_props_from_object(result, obj, source),
        Expression::TSAsExpression(ts_as) => {
            extract_props_from_runtime_expression(result, &ts_as.expression, source);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_props_from_runtime_expression(result, &ts_satisfies.expression, source);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_props_from_runtime_expression(result, &ts_non_null.expression, source);
        }
        Expression::ParenthesizedExpression(paren) => {
            extract_props_from_runtime_expression(result, &paren.expression, source);
        }
        _ => {}
    }
}

fn extract_props_from_array(result: &mut ScriptParseResult, arr: &ArrayExpression<'_>) {
    for elem in arr.elements.iter() {
        if let oxc_ast::ast::ArrayExpressionElement::StringLiteral(s) = elem {
            let name = s.value.as_str();
            result.macros.add_prop_with_declaration(
                PropDefinition {
                    name: CompactString::new(name),
                    required: false,
                    prop_type: None,
                    default_value: None,
                },
                s.span.start,
                s.span.end,
            );
            result.bindings.add(name, BindingType::Props);
        }
    }
}

fn extract_props_from_object(
    result: &mut ScriptParseResult,
    obj: &ObjectExpression<'_>,
    source: &str,
) {
    for prop in obj.properties.iter() {
        match prop {
            ObjectPropertyKind::ObjectProperty(p) => {
                let Some(name) = runtime_object_property_name(&p.key) else {
                    continue;
                };
                add_runtime_prop_with_declaration(
                    result,
                    PropDefinition {
                        name: CompactString::new(name),
                        required: detect_required_prop(&p.value),
                        prop_type: extract_runtime_prop_type(&p.value, source),
                        default_value: extract_runtime_prop_default(&p.value, source),
                    },
                    p.key.span().start,
                    p.span.end,
                );
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let Expression::Identifier(identifier) = &spread.argument else {
                    continue;
                };
                let Some(props) = result
                    .runtime_object_literals
                    .get(identifier.name.as_str())
                    .map(|literal| literal.props.clone())
                else {
                    continue;
                };
                for prop in props {
                    add_runtime_prop(result, prop);
                }
            }
        }
    }
}

/// Locate an inline type-literal prop at its written name and cover the rest of
/// its property signature. Referenced types do not declare a name inside the
/// `defineProps` call, so callers fall back to the macro span for those forms.
fn type_prop_declaration(type_param: &TSType<'_>, name: &str) -> Option<(u32, u32)> {
    let TSType::TSTypeLiteral(literal) = type_param else {
        return None;
    };
    literal.members.iter().find_map(|member| {
        let TSSignature::TSPropertySignature(property) = member else {
            return None;
        };
        (runtime_object_property_name(&property.key) == Some(name))
            .then(|| (property.key.span().start, property.span.end))
    })
}

fn add_runtime_prop(result: &mut ScriptParseResult, prop: PropDefinition) {
    result.bindings.add(prop.name.as_str(), BindingType::Props);
    result.macros.add_prop(prop);
}

fn add_runtime_prop_with_declaration(
    result: &mut ScriptParseResult,
    prop: PropDefinition,
    start: u32,
    end: u32,
) {
    result.bindings.add(prop.name.as_str(), BindingType::Props);
    result.macros.add_prop_with_declaration(prop, start, end);
}

pub(super) fn runtime_object_property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

pub(in crate::script_parser) fn extract_runtime_prop_type(
    value: &Expression<'_>,
    source: &str,
) -> Option<CompactString> {
    match value {
        Expression::Identifier(id) => runtime_ctor_type(id.name.as_str()).map(CompactString::new),
        Expression::ArrayExpression(arr) => {
            let mut union = String::default();
            let mut has_type = false;
            for elem in arr.elements.iter() {
                let Some(prop_type) = extract_runtime_prop_type_from_array_element(elem, source)
                else {
                    continue;
                };
                if has_type {
                    union.push_str(" | ");
                }
                if arr.elements.len() > 1 && prop_type.contains("=>") {
                    union.push('(');
                    union.push_str(prop_type.as_str());
                    union.push(')');
                } else {
                    union.push_str(prop_type.as_str());
                }
                has_type = true;
            }
            has_type.then(|| CompactString::new(union.as_str()))
        }
        Expression::ObjectExpression(obj) => obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                return None;
            };
            let PropertyKey::StaticIdentifier(id) = &prop.key else {
                return None;
            };
            (id.name.as_str() == "type")
                .then(|| extract_runtime_prop_type(&prop.value, source))
                .flatten()
        }),
        Expression::TSAsExpression(ts_as) => {
            runtime_prop_type_from_ts_type(&ts_as.type_annotation, source)
                .or_else(|| extract_runtime_prop_type(&ts_as.expression, source))
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            runtime_prop_type_from_ts_type(&ts_satisfies.type_annotation, source)
                .or_else(|| extract_runtime_prop_type(&ts_satisfies.expression, source))
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_runtime_prop_type(&ts_non_null.expression, source)
        }
        Expression::ParenthesizedExpression(paren) => {
            extract_runtime_prop_type(&paren.expression, source)
        }
        _ => None,
    }
}

fn extract_runtime_prop_type_from_array_element(
    value: &oxc_ast::ast::ArrayExpressionElement<'_>,
    source: &str,
) -> Option<CompactString> {
    match value {
        oxc_ast::ast::ArrayExpressionElement::Identifier(id) => {
            runtime_ctor_type(id.name.as_str()).map(CompactString::new)
        }
        oxc_ast::ast::ArrayExpressionElement::StringLiteral(_) => {
            Some(CompactString::new("string"))
        }
        oxc_ast::ast::ArrayExpressionElement::NumericLiteral(_) => {
            Some(CompactString::new("number"))
        }
        oxc_ast::ast::ArrayExpressionElement::BooleanLiteral(_) => {
            Some(CompactString::new("boolean"))
        }
        oxc_ast::ast::ArrayExpressionElement::ObjectExpression(_) => {
            Some(CompactString::new("Record<string, unknown>"))
        }
        oxc_ast::ast::ArrayExpressionElement::ArrayExpression(_) => {
            Some(CompactString::new("unknown[]"))
        }
        oxc_ast::ast::ArrayExpressionElement::TSAsExpression(ts_as) => {
            runtime_prop_type_from_ts_type(&ts_as.type_annotation, source)
                .or_else(|| extract_runtime_prop_type(&ts_as.expression, source))
        }
        oxc_ast::ast::ArrayExpressionElement::TSSatisfiesExpression(ts_satisfies) => {
            runtime_prop_type_from_ts_type(&ts_satisfies.type_annotation, source)
                .or_else(|| extract_runtime_prop_type(&ts_satisfies.expression, source))
        }
        oxc_ast::ast::ArrayExpressionElement::TSNonNullExpression(ts_non_null) => {
            extract_runtime_prop_type(&ts_non_null.expression, source)
        }
        oxc_ast::ast::ArrayExpressionElement::ParenthesizedExpression(paren) => {
            extract_runtime_prop_type(&paren.expression, source)
        }
        _ => None,
    }
}

pub(in crate::script_parser) fn extract_runtime_prop_default(
    value: &Expression<'_>,
    source: &str,
) -> Option<CompactString> {
    let Expression::ObjectExpression(obj) = value else {
        return None;
    };

    obj.properties.iter().find_map(|prop| {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            return None;
        };
        let PropertyKey::StaticIdentifier(id) = &prop.key else {
            return None;
        };
        if id.name.as_str() != "default" {
            return None;
        }

        source
            .get(prop.value.span().start as usize..prop.value.span().end as usize)
            .map(CompactString::new)
    })
}

fn runtime_ctor_type(name: &str) -> Option<&'static str> {
    match name {
        "String" => Some("string"),
        "Number" => Some("number"),
        "Boolean" => Some("boolean"),
        "Array" => Some("unknown[]"),
        "Object" => Some("Record<string, unknown>"),
        "Date" => Some("Date"),
        "Function" => Some("(...args: any[]) => any"),
        _ => None,
    }
}

/// Detect if a prop has required: true
pub(in crate::script_parser) fn detect_required_prop(value: &Expression<'_>) -> bool {
    if let Expression::ObjectExpression(obj) = value {
        for prop in obj.properties.iter() {
            if let ObjectPropertyKind::ObjectProperty(p) = prop
                && let PropertyKey::StaticIdentifier(id) = &p.key
                && id.name.as_str() == "required"
                && let Expression::BooleanLiteral(b) = &p.value
            {
                return b.value;
            }
        }
    }
    false
}
