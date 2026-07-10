use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind};
use vize_carton::String;
use vize_croquis::{Croquis, OptionGroup};

use crate::virtual_ts::props::OptionsApiPropsSource;

pub(super) fn extend_options_api_descriptor_names<'a>(
    names: &mut Vec<&'a str>,
    summary: &'a Croquis,
) {
    let Some(descriptor) = summary.options_descriptor.as_ref() else {
        return;
    };
    names.extend(descriptor.members.iter().filter_map(|member| {
        matches!(
            member.group,
            OptionGroup::Props
                | OptionGroup::Inject
                | OptionGroup::Computed
                | OptionGroup::Methods
                | OptionGroup::Data
                | OptionGroup::Setup
        )
        .then_some(member.name.as_str())
        .filter(|name| is_safe_value_identifier(name))
    }));
}

pub(super) fn props_source_from_object(
    object: &ObjectExpression<'_>,
    source: &str,
) -> OptionsApiPropsSource {
    let source = String::from(source);
    if object_props_must_stay_in_value_scope(object) {
        OptionsApiPropsSource::DeferredObject(source)
    } else {
        OptionsApiPropsSource::Object(source)
    }
}

fn object_props_must_stay_in_value_scope(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().any(|property| match property {
        ObjectPropertyKind::SpreadProperty(_) => true,
        ObjectPropertyKind::ObjectProperty(property) => {
            property.method || expression_must_stay_in_value_scope(&property.value)
        }
    })
}

fn expression_must_stay_in_value_scope(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_)
        | Expression::CallExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::NewExpression(_)
        | Expression::TSAsExpression(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::TSNonNullExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::TSTypeAssertion(_) => true,
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| {
            matches!(element, ArrayExpressionElement::SpreadElement(_))
                || element
                    .as_expression()
                    .is_some_and(expression_must_stay_in_value_scope)
        }),
        Expression::ObjectExpression(object) => object_props_must_stay_in_value_scope(object),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_must_stay_in_value_scope(&parenthesized.expression)
        }
        _ => false,
    }
}

pub(super) fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
