//! Close demanded type reads inside the existing expression walk.

use super::facts::IdentifierWalk;
use crate::script_parser::{
    annotation_has_value_reads, arguments_have_value_reads, has_value_type_reads,
    parameters_have_value_reads,
};
use oxc_ast::ast::{Expression, TSTypeAnnotation};

#[inline]
pub(super) fn refuse_annotation(
    annotation: Option<&TSTypeAnnotation<'_>>,
    identifiers: &mut IdentifierWalk,
) {
    if identifiers.demanded && annotation_has_value_reads(annotation) {
        identifiers.refuse();
    }
}

#[inline]
pub(super) fn refuse_expression_types(
    expression: &Expression<'_>,
    identifiers: &mut IdentifierWalk,
) {
    if !identifiers.demanded {
        return;
    }
    let unmodeled = match expression {
        Expression::TSAsExpression(expression) => has_value_type_reads(&expression.type_annotation),
        Expression::TSSatisfiesExpression(expression) => {
            has_value_type_reads(&expression.type_annotation)
        }
        Expression::TSTypeAssertion(expression) => {
            has_value_type_reads(&expression.type_annotation)
        }
        Expression::ArrowFunctionExpression(function) => {
            annotation_has_value_reads(function.return_type.as_deref())
                || parameters_have_value_reads(function.type_parameters.as_deref())
        }
        Expression::FunctionExpression(function) => {
            annotation_has_value_reads(function.return_type.as_deref())
                || parameters_have_value_reads(function.type_parameters.as_deref())
                || annotation_has_value_reads(
                    function
                        .this_param
                        .as_ref()
                        .and_then(|parameter| parameter.type_annotation.as_deref()),
                )
        }
        Expression::CallExpression(call) => {
            arguments_have_value_reads(call.type_arguments.as_deref())
        }
        Expression::NewExpression(call) => {
            arguments_have_value_reads(call.type_arguments.as_deref())
        }
        Expression::TaggedTemplateExpression(template) => {
            arguments_have_value_reads(template.type_arguments.as_deref())
        }
        Expression::TSInstantiationExpression(expression) => {
            arguments_have_value_reads(Some(&expression.type_arguments))
        }
        _ => false,
    };
    if unmodeled {
        identifiers.refuse();
    }
}
