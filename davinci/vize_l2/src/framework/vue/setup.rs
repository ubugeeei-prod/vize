//! `<script setup>` facts, observed inside the single L2 Program walk.
//!
//! Macro calls: Vue reads macros only in top-level expression statements and in the
//! initializers of top-level, non-`declare` variable declarators, after
//! removing TypeScript expression wrappers and parentheses. A `withDefaults`
//! call also records the `defineProps` call it wraps, and each `defineProps`
//! call records the prop keys its runtime argument declares. The first
//! `defineProps` call declares the component's props contract.

mod bindings;
mod props;
pub use bindings::{ReactiveSource, SetupBinding, SetupBindingKind};
pub use props::{PropsDestructure, PropsField};

use super::contract::{self, ComponentContract, LocalTypes, PropsDraft};
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
    runtime_prop_keys: Vec<Span>,
    bindings: Vec<SetupBinding>,
    props_destructure: Option<PropsDestructure>,
    unclassified: Vec<Span>,
    contract: ComponentContract,
}

impl SetupFacts {
    /// Top-level macro calls in source order.
    #[must_use]
    pub fn macro_calls(&self) -> &[MacroCall] {
        &self.macro_calls
    }

    /// The string-named keys `defineProps` runtime arguments declare, in
    /// source order: an identifier key, or a string literal's contents.
    /// Type-argument props belong to the props contract.
    #[must_use]
    pub fn runtime_prop_keys(&self) -> &[Span] {
        &self.runtime_prop_keys
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

    /// The component contract the macros declare.
    #[must_use]
    pub fn contract(&self) -> &ComponentContract {
        &self.contract
    }

    /// `defineProps` destructure patterns whose bindings are not classified
    /// (a second destructure, or one Vue rejects), and runtime prop keys
    /// whose text is not their source text. A consumer that needs every
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
    runtime_prop_keys: Vec<Span>,
    bindings: bindings::Bindings<'a>,
    types: LocalTypes<'a>,
    props_contract: Option<PropsDraft<'a>>,
    duplicates: Vec<Span>,
}

#[derive(Debug, Clone, Copy)]
struct Program {
    scope: ScopeId,
    base: u32,
    statements: usize,
}

impl<'a> SetupObserver<'a> {
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
        let props = self.props_contract.map(|mut props| {
            for (key, value) in self.bindings.defaults {
                props.destructure_default(key, value);
            }
            props.finish(&self.types)
        });
        Ok(SetupFacts {
            macro_calls: self.calls,
            runtime_prop_keys: self.runtime_prop_keys,
            bindings: self.bindings.rows,
            props_destructure: self.bindings.props,
            unclassified: self.bindings.unclassified,
            contract: ComponentContract {
                props,
                duplicates: self.duplicates,
            },
        })
    }

    fn authored(&mut self, base: u32, span: oxc_span::Span) -> Span {
        authored(base, span, &mut self.overflow)
    }

    fn expression(&mut self, base: u32, expression: &Expression<'a>, declarator: Option<Span>) {
        if let Expression::CallExpression(call) = unwrap(expression) {
            self.call(base, call, declarator);
        }
    }

    fn call(&mut self, base: u32, call: &CallExpression<'a>, declarator: Option<Span>) {
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
        if row.kind == MacroKind::DefineProps {
            self.define_props(base, record.span, call);
        }
        if row.kind == MacroKind::WithDefaults
            && let Some(Argument::CallExpression(inner)) = call.arguments.first()
            && matches!(&inner.callee, Expression::Identifier(id) if id.name == super::DEFINE_PROPS)
        {
            self.call(base, inner, declarator);
            let inner = self.authored(base, inner.span);
            let overflow = &mut self.overflow;
            if let Some(defaults) = call.arguments.get(1)
                && let Some(props) = self
                    .props_contract
                    .as_mut()
                    .filter(|props| props.declaration() == inner)
            {
                props.defaults(defaults, &mut |span| authored(base, span, overflow));
            }
        }
    }

    /// The runtime keys every call declares; the first call's contract.
    fn define_props(&mut self, base: u32, span: Span, call: &CallExpression<'a>) {
        let overflow = &mut self.overflow;
        let authored = &mut |span| authored(base, span, overflow);
        let runtime = call.arguments.first().map(|argument| {
            let read = contract::runtime_props(argument, &mut *authored);
            (authored(argument.span()), read)
        });
        if let Some((_, read)) = &runtime {
            self.runtime_prop_keys
                .extend(read.props.iter().map(|named| named.prop.name));
            self.bindings
                .unclassified
                .extend(read.escaped.iter().copied());
        }
        if self.props_contract.is_some() {
            self.duplicates.push(span);
            return;
        }
        let type_argument = call
            .type_arguments
            .as_ref()
            .and_then(|arguments| arguments.params.first());
        self.props_contract = Some(PropsDraft::new(span, type_argument, runtime, authored));
    }
}

fn authored(base: u32, span: oxc_span::Span, overflow: &mut bool) -> Span {
    match (base.checked_add(span.start), base.checked_add(span.end)) {
        (Some(start), Some(end)) => Span::new(start, end),
        _ => {
            *overflow = true;
            Span::new(0, 0)
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
        let overflow = &mut self.overflow;
        self.types.statement(event.statement, &mut |span| {
            authored(program.base, span, overflow)
        });
    }

    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}

    fn checkpoint(&self) -> SetupCheckpoint {
        SetupCheckpoint {
            calls: self.calls.len(),
            runtime_prop_keys: self.runtime_prop_keys.len(),
            observed: self.observed,
            bindings: self.bindings.rows.len(),
            props: self.bindings.props.is_some(),
            unclassified: self.bindings.unclassified.len(),
            defaults: self.bindings.defaults.len(),
            types: self.types.len(),
            contract: self.props_contract.is_some(),
            duplicates: self.duplicates.len(),
        }
    }

    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }

    /// A rolled-back statement no longer counts as observed.
    fn rollback(&mut self, checkpoint: SetupCheckpoint) {
        self.calls.truncate(checkpoint.calls);
        self.runtime_prop_keys
            .truncate(checkpoint.runtime_prop_keys);
        self.observed = checkpoint.observed;
        self.bindings.rows.truncate(checkpoint.bindings);
        self.bindings.unclassified.truncate(checkpoint.unclassified);
        if !checkpoint.props {
            self.bindings.props = None;
        }
        self.bindings.defaults.truncate(checkpoint.defaults);
        self.types.truncate(checkpoint.types);
        if !checkpoint.contract {
            self.props_contract = None;
        }
        self.duplicates.truncate(checkpoint.duplicates);
    }
}

/// Row counts before a walk subtree; statement facts never straddle one.
#[derive(Debug, Clone, Copy)]
pub struct SetupCheckpoint {
    calls: usize,
    runtime_prop_keys: usize,
    observed: usize,
    bindings: usize,
    props: bool,
    unclassified: usize,
    defaults: usize,
    types: usize,
    contract: bool,
    duplicates: usize,
}
