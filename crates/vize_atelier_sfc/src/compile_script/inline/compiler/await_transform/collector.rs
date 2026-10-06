//! Collect setup-owned awaits in the existing parsed setup tree.
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::{
    ArrowFunctionExpression, AwaitExpression, Expression, ExpressionStatement, Function, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;

#[derive(Clone, Copy)]
pub(super) struct AwaitRegion {
    pub(super) span: Span,
    /// Keep the established whole-statement bytes and provenance for direct awaits.
    pub(super) legacy_statement: Option<(Span, bool)>,
    pub(super) is_statement: bool,
    pub(super) needs_semicolon: bool,
    pub(super) object_argument: bool,
}

#[derive(Default)]
pub(super) struct SetupAwaits {
    pub(super) regions: Vec<AwaitRegion>,
    in_sequence: bool,
    statement_in_sequence: bool,
    expression_statement: Option<Span>,
    direct_await: Option<(Span, Span, bool)>,
}

impl<'a> Visit<'a> for SetupAwaits {
    fn visit_statements(&mut self, statements: &ArenaVec<'a, Statement<'a>>) {
        let previous = self.in_sequence;
        for statement in statements {
            self.in_sequence = true;
            self.visit_statement(statement);
        }
        self.in_sequence = previous;
    }

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        let sequence = self.in_sequence;
        let previous = (self.statement_in_sequence, self.direct_await);
        self.statement_in_sequence = sequence;
        self.direct_await = match statement {
            Statement::ExpressionStatement(expression) if sequence => {
                match &expression.expression {
                    Expression::AwaitExpression(await_expression) => {
                        Some((await_expression.span, statement.span(), false))
                    }
                    _ => None,
                }
            }
            Statement::VariableDeclaration(declaration) if declaration.declarations.len() == 1 => {
                declaration.declarations.first().and_then(|declarator| {
                    match declarator.init.as_ref()? {
                        Expression::AwaitExpression(await_expression) => {
                            Some((await_expression.span, statement.span(), true))
                        }
                        _ => None,
                    }
                })
            }
            _ => None,
        };
        self.in_sequence = false;
        walk::walk_statement(self, statement);
        self.in_sequence = sequence;
        (self.statement_in_sequence, self.direct_await) = previous;
    }

    fn visit_expression_statement(&mut self, statement: &ExpressionStatement<'a>) {
        let previous = self.expression_statement;
        self.expression_statement = Some(statement.expression.span());
        walk::walk_expression_statement(self, statement);
        self.expression_statement = previous;
    }

    fn visit_await_expression(&mut self, expression: &AwaitExpression<'a>) {
        let is_statement = self.expression_statement == Some(expression.span);
        self.regions.push(AwaitRegion {
            span: expression.span,
            legacy_statement: self.direct_await.and_then(|(span, statement, assignment)| {
                (span == expression.span).then_some((statement, assignment))
            }),
            is_statement,
            needs_semicolon: is_statement && self.statement_in_sequence,
            object_argument: matches!(&expression.argument, Expression::ObjectExpression(_)),
        });
        walk::walk_await_expression(self, expression);
    }

    // Function bodies have their own async lifetime, including methods and arrows.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}
