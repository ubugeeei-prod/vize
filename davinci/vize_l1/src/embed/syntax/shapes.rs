//! Internal parser wrappers and views over their authored syntax only.

use oxc_ast::ast::{
    ArrowFunctionExpression, AwaitExpression, Directive, Expression, FormalParameter,
    FormalParameterRest, FormalParameters, FunctionBody, ModuleDeclaration, Program, Statement,
    TSImportEqualsDeclaration,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};

use super::{EmbedHole, Shape, coordinates::Coordinates};

#[derive(Clone, Copy)]
pub(super) struct Wrapper {
    pub prefix: &'static str,
    pub suffix: &'static str,
}

impl Wrapper {
    pub const fn for_shape(shape: Shape) -> Option<Self> {
        let (prefix, suffix) = match shape {
            Shape::Program => ("", ""),
            Shape::Expr => ("(\n", "\n)"),
            Shape::HandlerBody => ("()=>{\n", "\n}"),
            Shape::SlotParams => ("(\n", "\n)=>{}"),
            Shape::ForHead | Shape::FilterChain => return None,
        };
        Some(Self { prefix, suffix })
    }
}

/// Authored handler syntax in a non-async lexical arrow context.
///
/// The generated arrow, its empty parameters and enclosing braces stay private.
/// Node spans use the owning `NativeSyntax`'s checked coordinate projections.
#[derive(Debug, Clone, Copy)]
pub struct HandlerBodyView<'s, 'a> {
    directives: &'s [Directive<'a>],
    statements: &'s [Statement<'a>],
}

impl<'s, 'a> HandlerBodyView<'s, 'a> {
    #[must_use]
    pub const fn directives(self) -> &'s [Directive<'a>] {
        self.directives
    }

    #[must_use]
    pub const fn statements(self) -> &'s [Statement<'a>] {
        self.statements
    }
}

/// Authored slot formal parameters, including patterns, defaults and rest.
///
/// The generated arrow and enclosing parameter-list span stay private. Node
/// spans use the owning `NativeSyntax`'s checked coordinate projections.
#[derive(Debug, Clone, Copy)]
pub struct SlotParamsView<'s, 'a> {
    parameters: &'s [FormalParameter<'a>],
    rest: Option<&'s FormalParameterRest<'a>>,
}

impl<'s, 'a> SlotParamsView<'s, 'a> {
    #[must_use]
    pub const fn parameters(self) -> &'s [FormalParameter<'a>] {
        self.parameters
    }

    #[must_use]
    pub const fn rest(self) -> Option<&'s FormalParameterRest<'a>> {
        self.rest
    }
}

fn arrow<'p, 'a>(program: &'p Program<'a>) -> Option<&'p ArrowFunctionExpression<'a>> {
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

pub(super) fn handler_body<'p, 'a>(program: &'p Program<'a>) -> Option<HandlerBodyView<'p, 'a>> {
    let arrow = arrow(program)?;
    Some(HandlerBodyView {
        directives: arrow.body.directives.as_slice(),
        statements: arrow.body.statements.as_slice(),
    })
}

pub(super) fn slot_params<'p, 'a>(program: &'p Program<'a>) -> Option<SlotParamsView<'p, 'a>> {
    let arrow = arrow(program)?;
    Some(SlotParamsView {
        parameters: arrow.params.items.as_slice(),
        rest: arrow.params.rest.as_deref(),
    })
}

/// Require the generated enclosing syntax exactly where it was inserted. An
/// input escaping its body/list cannot add statements, parameters or a tail.
pub(super) fn valid_wrapper(
    program: &Program<'_>,
    shape: Shape,
    coordinates: Coordinates<'_>,
) -> bool {
    let Some(arrow) = arrow(program) else {
        return false;
    };
    let source_end = coordinates.prefix + coordinates.source.text().len() as u32;
    let authored = |span| coordinates.decoded_span(span).is_ok();
    match shape {
        Shape::HandlerBody => {
            arrow.span == Span::new(0, source_end + 2)
                && arrow.params.span == Span::new(0, 2)
                && arrow.params.items.is_empty()
                && arrow.params.rest.is_none()
                && arrow.body.span == Span::new(4, source_end + 2)
                && arrow.body.directives.iter().all(|node| authored(node.span))
                && arrow
                    .body
                    .statements
                    .iter()
                    .all(|node| authored(node.span()))
        }
        Shape::SlotParams => {
            arrow.span == Span::new(0, source_end + 6)
                && arrow.params.span == Span::new(0, source_end + 2)
                && arrow.body.span == Span::new(source_end + 4, source_end + 6)
                && arrow.body.directives.is_empty()
                && arrow.body.statements.is_empty()
                && arrow.params.items.iter().all(|node| authored(node.span))
                && arrow
                    .params
                    .rest
                    .as_deref()
                    .is_none_or(|node| authored(node.span))
        }
        _ => false,
    }
}

/// OXC's syntax parser retains module declarations in function bodies and
/// normally leaves contextual rejection to its semantic builder. Reject that
/// grammar here without a second parse or a semantic artifact. The same walk
/// rejects AwaitExpression in any function's parameters. Arrow heads inherit
/// OXC's outer module Await setting, but parameter defaults cannot await. A
/// nested async function body remains its own valid Await context.
pub(super) fn context_hole(program: &Program<'_>) -> Option<EmbedHole> {
    #[derive(Default)]
    struct Context {
        module_declaration: bool,
        parameter_await: bool,
        in_parameters: bool,
    }
    impl<'a> Visit<'a> for Context {
        fn visit_module_declaration(&mut self, _declaration: &ModuleDeclaration<'a>) {
            self.module_declaration = true;
        }

        fn visit_ts_import_equals_declaration(
            &mut self,
            _declaration: &TSImportEqualsDeclaration<'a>,
        ) {
            self.module_declaration = true;
        }

        fn visit_formal_parameters(&mut self, parameters: &FormalParameters<'a>) {
            let outer = core::mem::replace(&mut self.in_parameters, true);
            walk::walk_formal_parameters(self, parameters);
            self.in_parameters = outer;
        }

        fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
            let outer = core::mem::replace(&mut self.in_parameters, false);
            walk::walk_function_body(self, body);
            self.in_parameters = outer;
        }

        fn visit_await_expression(&mut self, expression: &AwaitExpression<'a>) {
            self.parameter_await |= self.in_parameters;
            walk::walk_await_expression(self, expression);
        }
    }
    let mut visitor = Context::default();
    visitor.visit_program(program);
    if visitor.module_declaration {
        Some(EmbedHole::InvalidModuleContext)
    } else if visitor.parameter_await {
        Some(EmbedHole::InvalidParameterContext)
    } else {
        None
    }
}
