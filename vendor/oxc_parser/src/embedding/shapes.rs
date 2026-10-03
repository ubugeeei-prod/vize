//! The existing fixed wrappers and sole context walk, at their language owner.

use oxc_ast::ast::{
    ArrowFunctionExpression, AwaitExpression, BindingIdentifier, Expression, FormalParameterKind,
    FormalParameters, FunctionBody, ModuleDeclaration, Program, Statement,
    TSImportEqualsDeclaration, TSTypeAnnotation,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};

use super::{EmbeddingGoal, EmbeddingHole};

pub(super) const fn wrapper(goal: EmbeddingGoal) -> (&'static str, &'static str) {
    match goal {
        EmbeddingGoal::Expr => ("(\n", "\n)"),
        EmbeddingGoal::HandlerBody => ("()=>{\n", "\n}"),
        EmbeddingGoal::Parameters => ("(\n", "\n)=>{}"),
    }
}

pub(super) fn expression<'p, 'a>(program: &'p Program<'a>) -> Option<&'p Expression<'a>> {
    let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
        return None;
    };
    let Expression::ParenthesizedExpression(wrapper) = &statement.expression else {
        return None;
    };
    Some(&wrapper.expression)
}

pub(super) fn arrow<'p, 'a>(program: &'p Program<'a>) -> Option<&'p ArrowFunctionExpression<'a>> {
    let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
        return None;
    };
    let Expression::ArrowFunctionExpression(arrow) = &statement.expression else {
        return None;
    };
    (!arrow.r#async
        && !arrow.expression
        && arrow.type_parameters.is_none()
        && arrow.return_type.is_none())
    .then_some(arrow)
}

pub(super) fn shape_hole(
    program: &Program<'_>,
    goal: EmbeddingGoal,
    content: Span,
) -> Option<EmbeddingHole> {
    let authored = |span: Span| {
        span.start >= content.start
            && span.end <= content.end
            && span.start <= span.end
            && program.source_text.get(span.start as usize..span.end as usize).is_some()
    };
    if goal == EmbeddingGoal::Expr {
        return expression(program)
            .is_none_or(|root| !authored(root.span()))
            .then_some(EmbeddingHole::InvalidExpressionShape);
    }
    let Some(arrow) = arrow(program) else {
        return Some(EmbeddingHole::InvalidWrappedShape);
    };
    let valid = match goal {
        EmbeddingGoal::HandlerBody => {
            arrow.span == Span::new(0, content.end + 2)
                && arrow.params.span == Span::new(0, 2)
                && arrow.params.items.is_empty()
                && arrow.params.rest.is_none()
                && arrow.body.span == Span::new(4, content.end + 2)
                && arrow.body.directives.iter().all(|node| authored(node.span))
                && arrow.body.statements.iter().all(|node| authored(node.span()))
        }
        EmbeddingGoal::Parameters => {
            arrow.span == Span::new(0, content.end + 6)
                && arrow.params.span == Span::new(0, content.end + 2)
                && arrow.body.span == Span::new(content.end + 4, content.end + 6)
                && arrow.body.directives.is_empty()
                && arrow.body.statements.is_empty()
                && arrow.params.items.iter().all(|node| authored(node.span))
                && arrow.params.rest.as_deref().is_none_or(|node| authored(node.span))
        }
        EmbeddingGoal::Expr => false,
    };
    (!valid).then_some(EmbeddingHole::InvalidWrappedShape)
}

/// One relocated context walk. A runtime formal binding is distinct from an
/// erased type-signature binding, a property key and a reference expression.
pub(super) fn context_hole(program: &Program<'_>) -> Option<EmbeddingHole> {
    #[derive(Default)]
    struct Context {
        module_declaration: bool,
        parameter_await: bool,
        invalid_runtime_binding: bool,
        in_parameters: bool,
        runtime_formals: bool,
    }
    impl<'a> Visit<'a> for Context {
        fn visit_module_declaration(&mut self, _: &ModuleDeclaration<'a>) {
            self.module_declaration = true;
        }
        fn visit_ts_import_equals_declaration(&mut self, _: &TSImportEqualsDeclaration<'a>) {
            self.module_declaration = true;
        }
        fn visit_formal_parameters(&mut self, parameters: &FormalParameters<'a>) {
            let outer = core::mem::replace(&mut self.in_parameters, true);
            let runtime = core::mem::replace(
                &mut self.runtime_formals,
                parameters.kind != FormalParameterKind::Signature,
            );
            walk::walk_formal_parameters(self, parameters);
            self.in_parameters = outer;
            self.runtime_formals = runtime;
        }
        fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
            let outer = core::mem::replace(&mut self.in_parameters, false);
            let runtime = core::mem::replace(&mut self.runtime_formals, false);
            walk::walk_function_body(self, body);
            self.in_parameters = outer;
            self.runtime_formals = runtime;
        }
        fn visit_ts_type_annotation(&mut self, annotation: &TSTypeAnnotation<'a>) {
            let runtime = core::mem::replace(&mut self.runtime_formals, false);
            walk::walk_ts_type_annotation(self, annotation);
            self.runtime_formals = runtime;
        }
        fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
            self.invalid_runtime_binding |= self.runtime_formals
                && matches!(
                    identifier.name.as_str(),
                    "eval"
                        | "arguments"
                        | "implements"
                        | "interface"
                        | "let"
                        | "package"
                        | "private"
                        | "protected"
                        | "public"
                        | "static"
                        | "yield"
                );
        }
        fn visit_await_expression(&mut self, expression: &AwaitExpression<'a>) {
            self.parameter_await |= self.in_parameters;
            walk::walk_await_expression(self, expression);
        }
    }
    let mut visitor = Context::default();
    visitor.visit_program(program);
    if visitor.module_declaration {
        Some(EmbeddingHole::InvalidModuleContext)
    } else if visitor.parameter_await || visitor.invalid_runtime_binding {
        Some(EmbeddingHole::InvalidParameterContext)
    } else {
        None
    }
}
