//! Demand-only refusals for unmodeled value reads inside type subtrees.

use super::super::{ScriptParseResult, typeof_refs};

impl ScriptParseResult {
    pub(crate) fn refuse_function_type_reads(&mut self, function: &oxc_ast::ast::Function<'_>) {
        self.refuse_type_reads(function.return_type.as_deref());
        self.refuse_type_parameters(function.type_parameters.as_deref());
        self.refuse_type_reads(
            function
                .this_param
                .as_ref()
                .and_then(|parameter| parameter.type_annotation.as_deref()),
        );
    }

    pub(crate) fn refuse_declarator_type_reads(
        &mut self,
        declarator: &oxc_ast::ast::VariableDeclarator<'_>,
    ) {
        self.refuse_type_reads(declarator.type_annotation.as_deref());
        if self.occurrence_capture.is_none() {
            return;
        }
        // Macro dispatch unwraps these existing expression wrappers before its
        // normal call walk. Close their type relation without parsing again.
        if let Some(initializer) = &declarator.init
            && super::super::extract::extract_call_expression(initializer).is_some()
        {
            let mut expression = initializer;
            loop {
                use oxc_ast::ast::Expression;
                refuse_expression_type_reads(self, expression);
                expression = match expression {
                    Expression::TSAsExpression(expression) => &expression.expression,
                    Expression::TSSatisfiesExpression(expression) => &expression.expression,
                    Expression::TSTypeAssertion(expression) => &expression.expression,
                    Expression::TSNonNullExpression(expression) => &expression.expression,
                    Expression::ParenthesizedExpression(expression) => &expression.expression,
                    _ => break,
                };
            }
        }
    }

    pub(crate) fn refuse_type_reads(
        &mut self,
        annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    ) {
        if self.occurrence_capture.is_some()
            && annotation.is_some_and(|annotation| {
                typeof_refs::has_value_type_reads(&annotation.type_annotation)
            })
        {
            self.refuse_occurrences();
        }
    }

    pub(crate) fn refuse_type_parameters(
        &mut self,
        parameters: Option<&oxc_ast::ast::TSTypeParameterDeclaration<'_>>,
    ) {
        if self.occurrence_capture.is_some()
            && parameters.is_some_and(|parameters| {
                parameters.params.iter().any(|parameter| {
                    parameter
                        .constraint
                        .as_ref()
                        .is_some_and(typeof_refs::has_value_type_reads)
                        || parameter
                            .default
                            .as_ref()
                            .is_some_and(typeof_refs::has_value_type_reads)
                })
            })
        {
            self.refuse_occurrences();
        }
    }

    pub(crate) fn refuse_type_arguments(
        &mut self,
        arguments: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
    ) {
        if self.occurrence_capture.is_some()
            && arguments.is_some_and(|arguments| {
                arguments
                    .params
                    .iter()
                    .any(typeof_refs::has_value_type_reads)
            })
        {
            self.refuse_occurrences();
        }
    }
}

/// Type subtrees are checked only where the existing value walk reaches them.
pub(crate) fn refuse_expression_type_reads(
    result: &mut ScriptParseResult,
    expression: &oxc_ast::ast::Expression<'_>,
) {
    use oxc_ast::ast::Expression;
    if result.occurrence_capture.is_none() {
        return;
    }
    match expression {
        Expression::ArrowFunctionExpression(function) => {
            result.refuse_type_reads(function.return_type.as_deref());
            result.refuse_type_parameters(function.type_parameters.as_deref());
        }
        Expression::FunctionExpression(function) => {
            result.refuse_function_type_reads(function);
        }
        Expression::TSAsExpression(assertion) => {
            if typeof_refs::has_value_type_reads(&assertion.type_annotation) {
                result.refuse_occurrences();
            }
        }
        Expression::TSSatisfiesExpression(assertion) => {
            if typeof_refs::has_value_type_reads(&assertion.type_annotation) {
                result.refuse_occurrences();
            }
        }
        Expression::TSTypeAssertion(assertion) => {
            if typeof_refs::has_value_type_reads(&assertion.type_annotation) {
                result.refuse_occurrences();
            }
        }
        Expression::NewExpression(expression) => {
            result.refuse_type_arguments(expression.type_arguments.as_deref());
        }
        _ => {}
    }
}
