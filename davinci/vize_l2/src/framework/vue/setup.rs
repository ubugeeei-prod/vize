//! `<script setup>` facts, observed inside the single L2 Program walk.
//!
//! Macro calls: Vue reads macros only in top-level expression statements and in the
//! initializers of top-level, non-`declare` variable declarators, after
//! removing TypeScript expression wrappers and parentheses. A `withDefaults`
//! call also records the `defineProps` call it wraps.

mod bindings;
mod props;
pub use bindings::{SetupBinding, SetupBindingKind};
pub use props::{PropsDestructure, PropsField};

use super::{MacroKind, lookup};
use crate::file::{ScopeId, ScriptUnit};
use crate::lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
use crate::resolution::ResolutionErrorKind;
use alloc::vec::Vec;
use oxc_ast::ast::{Argument, CallExpression, Expression, Statement};
use oxc_span::GetSpan;
use vize_l0::Span;

/// One top-level macro call. Spans are authored file coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroCall {
    pub kind: MacroKind,
    /// The whole call expression.
    pub span: Span,
    pub callee: Span,
    /// The type argument list, angle brackets included.
    pub type_arguments: Option<Span>,
    /// From the first argument's start to the last argument's end.
    pub arguments: Option<Span>,
    /// The declarator pattern the call initializes. A `defineProps` call
    /// wrapped by `withDefaults` carries the wrapper's pattern.
    pub declarator: Option<Span>,
}

/// The facts of one `<script setup>` Program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupFacts {
    macro_calls: Vec<MacroCall>,
    bindings: Vec<SetupBinding>,
    props_destructure: Option<PropsDestructure>,
    unclassified: Vec<Span>,
}

impl SetupFacts {
    /// Top-level macro calls in source order.
    #[must_use]
    pub fn macro_calls(&self) -> &[MacroCall] {
        &self.macro_calls
    }

    /// Bindings introduced by top-level identifiers, imports, functions,
    /// classes, enums and `defineProps` destructure locals, in source order.
    /// Props read without a local belong to the props contract.
    #[must_use]
    pub fn bindings(&self) -> &[SetupBinding] {
        &self.bindings
    }

    /// The `defineProps` destructure, when there is one.
    #[must_use]
    pub fn props_destructure(&self) -> Option<&PropsDestructure> {
        self.props_destructure.as_ref()
    }

    /// `defineProps` destructure patterns whose bindings are not classified:
    /// a second destructure, or one Vue rejects. A consumer that needs every
    /// binding refuses a non-empty list.
    #[must_use]
    pub fn unclassified(&self) -> &[Span] {
        &self.unclassified
    }
}

/// Why the observed facts do not cover the whole Program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupFactsIncomplete {
    /// The walk never started a Program.
    NoProgram,
    /// The observer saw a second Program; it observes exactly one.
    SecondProgram,
    /// Some top-level statement produced no event.
    MissingStatements { observed: usize, expected: usize },
    /// A span left the 32-bit file coordinate space.
    SpanOverflow,
}

/// Collects the facts of one `<script setup>` Program from its top-level
/// statement events. Pass it to `FileProducer::program_observed`, then
/// [`Self::finish`].
#[derive(Debug, Default)]
pub struct SetupObserver<'a> {
    program: Option<Program>,
    second_program: bool,
    overflow: bool,
    observed: usize,
    calls: Vec<MacroCall>,
    bindings: bindings::Bindings<'a>,
}

#[derive(Debug, Clone, Copy)]
struct Program {
    scope: ScopeId,
    base: u32,
    statements: usize,
}

impl SetupObserver<'_> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The facts, once every top-level statement was seen.
    pub fn finish(self) -> Result<SetupFacts, SetupFactsIncomplete> {
        let Some(program) = self.program else {
            return Err(SetupFactsIncomplete::NoProgram);
        };
        if self.second_program {
            return Err(SetupFactsIncomplete::SecondProgram);
        }
        if self.overflow || self.bindings.overflow {
            return Err(SetupFactsIncomplete::SpanOverflow);
        }
        if self.observed != program.statements {
            return Err(SetupFactsIncomplete::MissingStatements {
                observed: self.observed,
                expected: program.statements,
            });
        }
        Ok(SetupFacts {
            macro_calls: self.calls,
            bindings: self.bindings.rows,
            props_destructure: self.bindings.props,
            unclassified: self.bindings.unclassified,
        })
    }

    fn authored(&mut self, base: u32, span: oxc_span::Span) -> Span {
        match (base.checked_add(span.start), base.checked_add(span.end)) {
            (Some(start), Some(end)) => Span::new(start, end),
            _ => {
                self.overflow = true;
                Span::new(0, 0)
            }
        }
    }

    fn expression(&mut self, base: u32, expression: &Expression<'_>, declarator: Option<Span>) {
        if let Expression::CallExpression(call) = unwrap(expression) {
            self.call(base, call, declarator);
        }
    }

    fn call(&mut self, base: u32, call: &CallExpression<'_>, declarator: Option<Span>) {
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        let Some(row) = lookup(callee.name.as_str()) else {
            return;
        };
        let arguments = match (call.arguments.first(), call.arguments.last()) {
            (Some(first), Some(last)) => {
                Some(oxc_span::Span::new(first.span().start, last.span().end))
            }
            _ => None,
        };
        let record = MacroCall {
            kind: row.kind,
            span: self.authored(base, call.span),
            callee: self.authored(base, callee.span),
            type_arguments: call
                .type_arguments
                .as_ref()
                .map(|arguments| self.authored(base, arguments.span)),
            arguments: arguments.map(|span| self.authored(base, span)),
            declarator,
        };
        self.calls.push(record);
        if row.kind == MacroKind::WithDefaults
            && let Some(Argument::CallExpression(inner)) = call.arguments.first()
            && matches!(&inner.callee, Expression::Identifier(id) if id.name == super::DEFINE_PROPS)
        {
            self.call(base, inner, declarator);
        }
    }
}

/// Vue's `unwrapTSNode`, plus the parentheses Babel does not retain.
fn unwrap<'e, 'a>(expression: &'e Expression<'a>) -> &'e Expression<'a> {
    match expression {
        Expression::TSAsExpression(inner) => unwrap(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrap(&inner.expression),
        Expression::TSTypeAssertion(inner) => unwrap(&inner.expression),
        Expression::TSNonNullExpression(inner) => unwrap(&inner.expression),
        Expression::TSInstantiationExpression(inner) => unwrap(&inner.expression),
        Expression::ParenthesizedExpression(inner) => unwrap(&inner.expression),
        other => other,
    }
}

impl<'a> FileObserver<'a> for SetupObserver<'a> {
    type Checkpoint = SetupCheckpoint;

    fn unit(&mut self, unit: &ScriptUnit) {
        if self.program.is_some() {
            self.second_program = true;
            return;
        }
        self.program = Some(Program {
            scope: unit.scope,
            base: unit.span.start,
            statements: unit.origin.length,
        });
    }

    fn statement(&mut self, event: StatementEvent<'_, 'a>) {
        let Some(program) = self.program else {
            return;
        };
        if self.second_program || event.scope != program.scope {
            return;
        }
        self.observed += 1;
        match event.statement {
            Statement::ExpressionStatement(statement) => {
                self.expression(program.base, &statement.expression, None);
            }
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    if let Some(init) = &declarator.init {
                        let pattern = self.authored(program.base, declarator.id.span());
                        self.expression(program.base, init, Some(pattern));
                    }
                }
            }
            _ => {}
        }
        self.bindings.statement(program.base, event.statement);
    }

    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}

    fn checkpoint(&self) -> SetupCheckpoint {
        SetupCheckpoint {
            calls: self.calls.len(),
            observed: self.observed,
            bindings: self.bindings.rows.len(),
            props: self.bindings.props.is_some(),
            unclassified: self.bindings.unclassified.len(),
        }
    }

    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }

    /// A rolled-back statement no longer counts as observed.
    fn rollback(&mut self, checkpoint: SetupCheckpoint) {
        self.calls.truncate(checkpoint.calls);
        self.observed = checkpoint.observed;
        self.bindings.rows.truncate(checkpoint.bindings);
        self.bindings.unclassified.truncate(checkpoint.unclassified);
        if !checkpoint.props {
            self.bindings.props = None;
        }
    }
}

/// Row counts before a walk subtree; statement facts never straddle one.
#[derive(Debug, Clone, Copy)]
pub struct SetupCheckpoint {
    calls: usize,
    observed: usize,
    bindings: usize,
    props: bool,
    unclassified: usize,
}
