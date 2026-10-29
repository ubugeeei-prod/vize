//! AST ownership of native component arrows; callback roots stay expressions.

use super::error;
use crate::finder::signature::{
    destructured_prop_names, formal_parameters_range, type_parameters_range,
};
use crate::{ComponentSetupSpan, JsxDiagnostic};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, Expression, Function, ReturnStatement, Statement,
    VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};

pub(super) struct BindingOrigin {
    pub(super) name: vize_l0::String,
    pub(super) span: Span,
}

pub(super) struct Plan {
    pub(super) binding: Option<BindingOrigin>,
    pub(super) span: Span,
    pub(super) setup: ComponentSetupSpan,
}

pub(super) fn collect(
    program: &oxc_ast::ast::Program<'_>,
    roots: &[(u32, u32)],
) -> Result<Vec<Plan>, JsxDiagnostic> {
    if roots.is_empty() {
        return Ok(Vec::new());
    }
    let mut visitor = Plans {
        roots,
        plans: Vec::new(),
        error: None,
    };
    visitor.visit_program(program);
    if let Some(error) = visitor.error {
        return Err(error);
    }
    Ok(visitor.plans)
}

struct Plans<'s> {
    roots: &'s [(u32, u32)],
    plans: Vec<Plan>,
    error: Option<JsxDiagnostic>,
}

fn jsx_span(expression: &Expression<'_>) -> Option<Span> {
    let inner = expression.get_inner_expression();
    matches!(
        inner,
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    )
    .then(|| inner.span())
}

impl Plans<'_> {
    fn arrow(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        let (root, setup_start, setup_end) = if arrow.expression {
            let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.first()
            else {
                return;
            };
            let Some(span) = jsx_span(&statement.expression) else {
                return;
            };
            (span, 0, 0)
        } else {
            let Some(Statement::ReturnStatement(statement)) = arrow.body.statements.last() else {
                return;
            };
            let Some(span) = statement.argument.as_ref().and_then(jsx_span) else {
                return;
            };
            let mut returns = Returns(0);
            returns.visit_function_body(&arrow.body);
            if returns.0 != 1 {
                return;
            }
            (span, arrow.body.span.start + 1, statement.span.start)
        };
        if !self.roots.contains(&(root.start, root.end)) {
            return;
        }
        let params = &arrow.params;
        let props = destructured_prop_names(params);
        if params.rest.is_some()
            || params.items.len() > 2
            || params.items.first().is_some_and(|parameter| {
                !matches!(parameter.pattern, BindingPattern::BindingIdentifier(_))
                    || props.is_empty()
            })
        {
            self.error = Some(error(
                arrow.span.start,
                arrow.span.end,
                "Vapor/SSR component props require an identifier with an explicit nonempty inline object type; destructured, inferred and aliased props need a reactive binding projection",
            ));
            return;
        }
        let (params_start, params_end) = formal_parameters_range(params);
        let (type_params_start, type_params_end) =
            type_parameters_range(arrow.type_parameters.as_deref());
        self.plans.push(Plan {
            binding: None,
            span: arrow.span,
            setup: ComponentSetupSpan {
                declaration_start: arrow.span.start,
                declaration_end: arrow.span.end,
                params_start,
                params_end,
                type_params_start,
                type_params_end,
                is_async: arrow.r#async,
                destructured_props: props,
                setup_start,
                setup_end,
                render_start: root.start,
                render_end: root.end,
            },
        });
    }
}

impl<'a> Visit<'a> for Plans<'_> {
    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if let Some(name) = declaration.id.get_identifier_name()
            && let Some(Expression::ArrowFunctionExpression(arrow)) = declaration
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
        {
            let previous = self.plans.len();
            self.arrow(arrow);
            if self.plans.len() > previous
                && let Some(plan) = self.plans.last_mut()
            {
                plan.binding = Some(BindingOrigin {
                    name: name.as_str().into(),
                    span: declaration.id.span(),
                });
            }
        }
        walk::walk_variable_declarator(self, declaration);
    }
    fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) =
            &declaration.declaration
        {
            self.arrow(arrow);
        }
        walk::walk_export_default_declaration(self, declaration);
    }
}

struct Returns(usize);
impl<'a> Visit<'a> for Returns {
    fn visit_return_statement(&mut self, _: &ReturnStatement<'a>) {
        self.0 += 1;
    }
    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}
