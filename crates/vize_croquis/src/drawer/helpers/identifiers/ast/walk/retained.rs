//! Demand-only, bounded token reads. Never called by the ordinary compiler walk.
//!
//! Every recursive phase shares one budget, including binding/declaration
//! collection. Refusal unwinds with `None`; partially collected facts never
//! escape. AST spans are copied, never reconstructed from cooked name lengths.

mod assignments;
mod scopes;

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, AssignmentTarget, AssignmentTargetMaybeDefault,
    AssignmentTargetProperty, BindingPattern, CallExpression, ChainElement, Expression,
    FormalParameters, FunctionBody, ObjectPropertyKind, SimpleAssignmentTarget, Statement,
};
use oxc_span::Span;
use vize_carton::CompactString;

use super::super::super::{RetainedIdentifierRef, RetainedParameterRefs};

const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 4096;

pub(crate) fn binding_declarations(
    pattern: &BindingPattern<'_>,
) -> Option<Vec<RetainedIdentifierRef>> {
    let mut walk = TokenWalk::default();
    let mut declarations = Vec::new();
    walk.collect_bindings(pattern, &mut declarations)?;
    Some(declarations)
}

pub(crate) fn expression_refs(ast: &Expression<'_>) -> Option<Vec<RetainedIdentifierRef>> {
    let mut walk = TokenWalk::default();
    walk.expression(ast)?;
    Some(walk.references)
}

pub(crate) fn parameter_refs(params: &FormalParameters<'_>) -> Option<RetainedParameterRefs> {
    let mut walk = TokenWalk::default();
    let bindings = walk.parameters(params)?;
    Some(RetainedParameterRefs {
        bindings,
        references: walk.references,
    })
}

pub(crate) fn handler_refs(body: &FunctionBody<'_>) -> Option<Vec<RetainedIdentifierRef>> {
    let mut walk = TokenWalk::default();
    walk.body(body)?;
    Some(walk.references)
}

#[derive(Default)]
struct TokenWalk {
    references: Vec<RetainedIdentifierRef>,
    locals: Vec<CompactString>,
    nodes: usize,
    depth: usize,
}

impl TokenWalk {
    fn tick(&mut self) -> Option<()> {
        self.nodes += 1;
        (self.nodes <= MAX_NODES).then_some(())
    }

    fn within<T>(&mut self, f: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        self.tick()?;
        if self.depth >= MAX_DEPTH {
            return None;
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }

    fn reference(&mut self, name: &str, span: Span) -> Option<()> {
        // Even a locally named eval is conservatively refused. Parenthesized
        // and TS-wrapped direct eval therefore cannot bypass this boundary.
        if name == "eval" {
            return None;
        }
        if !self.locals.iter().any(|local| local == name) {
            self.references.push(RetainedIdentifierRef {
                name: CompactString::new(name),
                span,
            });
        }
        Some(())
    }

    fn arguments(&mut self, arguments: &[Argument<'_>]) -> Option<()> {
        for argument in arguments {
            self.tick()?;
            match argument {
                Argument::SpreadElement(spread) => self.expression(&spread.argument)?,
                _ => self.expression(argument.as_expression()?)?,
            }
        }
        Some(())
    }

    fn call(&mut self, call: &CallExpression<'_>) -> Option<()> {
        self.expression(&call.callee)?;
        self.arguments(&call.arguments)
    }

    fn expression(&mut self, expression: &Expression<'_>) -> Option<()> {
        self.within(|walk| match expression {
            Expression::Identifier(id) => walk.reference(id.name.as_str(), id.span),
            Expression::ThisExpression(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::RegExpLiteral(_) => Some(()),
            Expression::StaticMemberExpression(member) => walk.expression(&member.object),
            Expression::ComputedMemberExpression(member) => {
                walk.expression(&member.object)?;
                walk.expression(&member.expression)
            }
            Expression::PrivateFieldExpression(field) => walk.expression(&field.object),
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    walk.tick()?;
                    match property {
                        ObjectPropertyKind::ObjectProperty(property) => {
                            if property.computed {
                                walk.expression(property.key.as_expression()?)?;
                            }
                            // The value owns the reference, including shorthand.
                            walk.expression(&property.value)?;
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            walk.expression(&spread.argument)?;
                        }
                    }
                }
                Some(())
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    walk.tick()?;
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            walk.expression(&spread.argument)?;
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        _ => walk.expression(element.as_expression()?)?,
                    }
                }
                Some(())
            }
            Expression::BinaryExpression(binary) => {
                walk.expression(&binary.left)?;
                walk.expression(&binary.right)
            }
            Expression::LogicalExpression(logical) => {
                walk.expression(&logical.left)?;
                walk.expression(&logical.right)
            }
            Expression::ConditionalExpression(conditional) => {
                walk.expression(&conditional.test)?;
                walk.expression(&conditional.consequent)?;
                walk.expression(&conditional.alternate)
            }
            Expression::UnaryExpression(unary) => walk.expression(&unary.argument),
            Expression::UpdateExpression(update) => walk.simple_target(&update.argument),
            Expression::CallExpression(call) => walk.call(call),
            Expression::NewExpression(new) => {
                walk.expression(&new.callee)?;
                walk.arguments(&new.arguments)
            }
            Expression::ArrowFunctionExpression(arrow) => {
                let outer = walk.locals.len();
                walk.parameters(&arrow.params)?;
                walk.body(&arrow.body)?;
                walk.locals.truncate(outer);
                Some(())
            }
            Expression::FunctionExpression(function) => {
                let outer = walk.locals.len();
                walk.locals.push(CompactString::new("arguments"));
                if let Some(id) = &function.id {
                    walk.tick()?;
                    walk.locals.push(CompactString::new(id.name.as_str()));
                }
                // Decorated parameters and body-less overload-like forms are
                // not an executable function whose reads we can prove.
                walk.parameters(&function.params)?;
                walk.body(function.body.as_ref()?)?;
                walk.locals.truncate(outer);
                Some(())
            }
            Expression::SequenceExpression(sequence) => {
                for expression in &sequence.expressions {
                    walk.expression(expression)?;
                }
                Some(())
            }
            Expression::AssignmentExpression(assignment) => {
                walk.target(&assignment.left)?;
                walk.expression(&assignment.right)
            }
            Expression::TemplateLiteral(template) => {
                for _ in &template.quasis {
                    walk.tick()?;
                }
                for expression in &template.expressions {
                    walk.expression(expression)?;
                }
                Some(())
            }
            Expression::TaggedTemplateExpression(tagged) => {
                walk.expression(&tagged.tag)?;
                for _ in &tagged.quasi.quasis {
                    walk.tick()?;
                }
                for expression in &tagged.quasi.expressions {
                    walk.expression(expression)?;
                }
                Some(())
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                walk.expression(&parenthesized.expression)
            }
            Expression::AwaitExpression(awaited) => walk.expression(&awaited.argument),
            Expression::YieldExpression(yielded) => {
                if let Some(argument) = &yielded.argument {
                    walk.expression(argument)?;
                }
                Some(())
            }
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => walk.call(call),
                ChainElement::TSNonNullExpression(non_null) => {
                    walk.expression(&non_null.expression)
                }
                ChainElement::StaticMemberExpression(member) => walk.expression(&member.object),
                ChainElement::ComputedMemberExpression(member) => {
                    walk.expression(&member.object)?;
                    walk.expression(&member.expression)
                }
                ChainElement::PrivateFieldExpression(field) => walk.expression(&field.object),
            },
            // Types never execute: do not mistake type-only names (including
            // typeof queries) for runtime template references.
            Expression::TSAsExpression(as_expression) => walk.expression(&as_expression.expression),
            Expression::TSSatisfiesExpression(satisfies) => walk.expression(&satisfies.expression),
            Expression::TSNonNullExpression(non_null) => walk.expression(&non_null.expression),
            Expression::TSTypeAssertion(assertion) => walk.expression(&assertion.expression),
            Expression::TSInstantiationExpression(instantiation) => {
                walk.expression(&instantiation.expression)
            }
            _ => None,
        })
    }
}
