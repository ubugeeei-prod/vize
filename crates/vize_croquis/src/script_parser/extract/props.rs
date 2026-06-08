use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey, TSType};
use oxc_span::{GetSpan, Span};

use crate::macros::PropDefinition;
use vize_carton::{CompactString, String};
use vize_relief::BindingType;

use super::super::ScriptParseResult;

pub fn extract_props_from_type(
    result: &mut ScriptParseResult,
    type_params: &oxc_allocator::Vec<'_, TSType<'_>>,
    _source: &str,
) {
    for tp in type_params.iter() {
        if let TSType::TSTypeLiteral(lit) = tp {
            for member in lit.members.iter() {
                if let oxc_ast::ast::TSSignature::TSPropertySignature(prop) = member
                    && let PropertyKey::StaticIdentifier(id) = &prop.key
                {
                    let name = id.name.as_str();
                    result.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required: !prop.optional,
                        prop_type: None,
                        default_value: None,
                    });
                    result.bindings.add(name, BindingType::Props);
                }
            }
        }
    }
}

/// Extract props from runtime arguments (array or object)
pub fn extract_props_from_runtime(
    result: &mut ScriptParseResult,
    arg: &Argument<'_>,
    source: &str,
) {
    match arg {
        // Array syntax: ['prop1', 'prop2']
        Argument::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                if let oxc_ast::ast::ArrayExpressionElement::StringLiteral(s) = elem {
                    let name = s.value.as_str();
                    result.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required: false,
                        prop_type: None,
                        default_value: None,
                    });
                    result.bindings.add(name, BindingType::Props);
                }
            }
        }

        // Object syntax: { prop1: Type, prop2: { type: Type, required: true } }
        Argument::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                if let ObjectPropertyKind::ObjectProperty(p) = prop {
                    let name = match &p.key {
                        PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                        PropertyKey::StringLiteral(s) => s.value.as_str(),
                        _ => continue,
                    };
                    let required = detect_required_prop(&p.value);
                    let prop_type = extract_runtime_prop_type(&p.value, source);
                    let default_value = extract_runtime_prop_default(&p.value, source);
                    result.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required,
                        prop_type,
                        default_value,
                    });
                    result.bindings.add(name, BindingType::Props);
                }
            }
        }

        _ => {}
    }
}

fn extract_runtime_prop_type(value: &Expression<'_>, source: &str) -> Option<CompactString> {
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
                union.push_str(prop_type.as_str());
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
            extract_runtime_prop_type_from_annotation(source, ts_as.type_annotation.span())
                .or_else(|| extract_runtime_prop_type(&ts_as.expression, source))
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_runtime_prop_type_from_annotation(source, ts_satisfies.type_annotation.span())
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
            extract_runtime_prop_type_from_annotation(source, ts_as.type_annotation.span())
                .or_else(|| extract_runtime_prop_type(&ts_as.expression, source))
        }
        oxc_ast::ast::ArrayExpressionElement::TSSatisfiesExpression(ts_satisfies) => {
            extract_runtime_prop_type_from_annotation(source, ts_satisfies.type_annotation.span())
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

fn extract_runtime_prop_type_from_annotation(source: &str, span: Span) -> Option<CompactString> {
    let annotation = source.get(span.start as usize..span.end as usize)?.trim();
    extract_prop_type_generic(annotation, "PropType")
        .or_else(|| extract_prop_type_generic(annotation, "ReadonlyArray"))
}

fn extract_prop_type_generic(annotation: &str, type_name: &str) -> Option<CompactString> {
    let mut marker = String::default();
    marker.push_str(type_name);
    marker.push('<');
    let start = annotation.find(marker.as_str())? + marker.len();
    let mut depth = 1i32;

    for (idx, ch) in annotation[start..].char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    let inner = annotation[start..start + idx].trim();
                    return (!inner.is_empty()).then(|| CompactString::new(inner));
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_runtime_prop_default(value: &Expression<'_>, source: &str) -> Option<CompactString> {
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
fn detect_required_prop(value: &Expression<'_>) -> bool {
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
