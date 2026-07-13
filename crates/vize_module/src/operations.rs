#[path = "operations/expression.rs"]
mod expression;
#[path = "operations/pattern.rs"]
mod pattern;

use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AwaitExpression, BindingPattern, CallExpression,
    Expression, Function, IdentifierReference, Program, ReturnStatement, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::{
    ModuleExpression, ModuleFunction, ModuleOperation, ModuleOperationKind, ModuleOperations,
    ModulePattern, ModuleSpan,
};
use expression::{call_snapshot, expression_snapshot};
use pattern::{assignment_pattern, binding_kind, binding_pattern, formal_parameters};

pub(crate) fn collect(program: &Program<'_>, source: &str, base: u32) -> ModuleOperations {
    let mut collector = OperationCollector {
        source,
        base,
        output: ModuleOperations::default(),
        functions: Vec::new(),
        after_await: Vec::new(),
    };
    collector.visit_program(program);
    for function in &mut collector.output.functions {
        function.references.sort_unstable();
        function.references.dedup();
        function.local_bindings.sort_unstable();
        function.local_bindings.dedup();
    }
    collector.output
}

struct OperationCollector<'s> {
    source: &'s str,
    base: u32,
    output: ModuleOperations,
    functions: Vec<usize>,
    after_await: Vec<bool>,
}

impl OperationCollector<'_> {
    fn push(&mut self, kind: ModuleOperationKind, span: Span) {
        self.output.operations.push(ModuleOperation {
            kind,
            span: absolute(span, self.base),
            function: self.functions.last().copied(),
            top_level: self.functions.is_empty(),
            after_await: self.after_await.last().copied().unwrap_or(false),
        });
    }

    fn enter_function(
        &mut self,
        name: Option<&str>,
        async_: bool,
        parameters: Vec<ModulePattern>,
        span: Span,
    ) {
        let id = self.output.functions.len();
        self.output.functions.push(ModuleFunction {
            id,
            parent: self.functions.last().copied(),
            name: name.map(Into::into),
            async_,
            local_bindings: pattern_names(&parameters),
            parameters,
            span: absolute(span, self.base),
            references: Vec::new(),
        });
        self.functions.push(id);
        self.after_await.push(false);
    }

    fn exit_function(&mut self) {
        self.functions.pop();
        self.after_await.pop();
    }

    fn expression(&self, expression: &Expression<'_>) -> ModuleExpression {
        expression_snapshot(expression, self.source, self.base)
    }

    fn pattern(&self, pattern: &BindingPattern<'_>) -> ModulePattern {
        binding_pattern(pattern, self.source, self.base)
    }
}

impl<'a> Visit<'a> for OperationCollector<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(function) = self.functions.last().copied() {
            collect_pattern_names(
                &self.pattern(&declarator.id),
                &mut self.output.functions[function].local_bindings,
            );
        }
        self.push(
            ModuleOperationKind::Binding {
                kind: binding_kind(declarator.kind),
                pattern: self.pattern(&declarator.id),
                initializer: declarator.init.as_ref().map(|value| self.expression(value)),
            },
            declarator.span,
        );
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        self.push(
            ModuleOperationKind::Assignment {
                target: assignment_pattern(&assignment.left, self.source, self.base),
                value: self.expression(&assignment.right),
            },
            assignment.span,
        );
        walk::walk_assignment_expression(self, assignment);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.push(
            ModuleOperationKind::Call(call_snapshot(call, self.source, self.base)),
            call.span,
        );
        walk::walk_call_expression(self, call);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        self.push(
            ModuleOperationKind::Return(
                statement
                    .argument
                    .as_ref()
                    .map(|value| self.expression(value)),
            ),
            statement.span,
        );
        walk::walk_return_statement(self, statement);
    }

    fn visit_await_expression(&mut self, expression: &AwaitExpression<'a>) {
        walk::walk_await_expression(self, expression);
        self.push(
            ModuleOperationKind::Await(self.expression(&expression.argument)),
            expression.span,
        );
        if let Some(after) = self.after_await.last_mut() {
            *after = true;
        }
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.enter_function(
            function.id.as_ref().map(|id| id.name.as_str()),
            function.r#async,
            formal_parameters(&function.params, self.source, self.base),
            function.span,
        );
        walk::walk_function(self, function, flags);
        self.exit_function();
    }

    fn visit_arrow_function_expression(&mut self, function: &ArrowFunctionExpression<'a>) {
        self.enter_function(
            None,
            function.r#async,
            formal_parameters(&function.params, self.source, self.base),
            function.span,
        );
        walk::walk_arrow_function_expression(self, function);
        self.exit_function();
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if let Some(function) = self.functions.last().copied() {
            self.output.functions[function]
                .references
                .push(identifier.name.as_str().into());
        }
    }
}

fn pattern_names(patterns: &[ModulePattern]) -> Vec<Box<str>> {
    let mut names = Vec::new();
    for pattern in patterns {
        collect_pattern_names(pattern, &mut names);
    }
    names
}

fn collect_pattern_names(pattern: &ModulePattern, names: &mut Vec<Box<str>>) {
    match pattern {
        ModulePattern::Identifier(name) => names.push(name.clone()),
        ModulePattern::Object(properties) => {
            for property in properties {
                collect_pattern_names(&property.value, names);
            }
        }
        ModulePattern::Array(items) => {
            for item in items.iter().flatten() {
                collect_pattern_names(item, names);
            }
        }
        ModulePattern::Rest(pattern)
        | ModulePattern::Assignment {
            binding: pattern, ..
        } => {
            collect_pattern_names(pattern, names);
        }
        ModulePattern::Path(_) | ModulePattern::Unknown { .. } => {}
    }
}

pub(super) fn slice(source: &str, span: Span) -> &str {
    let start = (span.start as usize).min(source.len());
    let end = (span.end as usize).min(source.len()).max(start);
    &source[start..end]
}

pub(super) const fn absolute(span: Span, base: u32) -> ModuleSpan {
    ModuleSpan::new(
        base.saturating_add(span.start),
        base.saturating_add(span.end),
    )
}
