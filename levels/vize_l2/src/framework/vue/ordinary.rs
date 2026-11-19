//! Normal `<script>` bindings observed by the single L2 Program walk.
//!
//! These facts cover top-level runtime imports and declarations that a paired
//! `<script setup>` exposes to its template. Options API members are a separate
//! fact family because module bindings and component-instance bindings have
//! different access rules.

use super::setup::{Bindings, SetupBinding};
use crate::file::{ScopeId, ScriptUnit};
use crate::lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
use crate::resolution::ResolutionErrorKind;

/// A normal-script binding uses the same template access classification as a
/// setup binding; the alias keeps the owning script role explicit to callers.
pub type OrdinaryBinding = SetupBinding;

/// Top-level runtime bindings of one normal `<script>` Program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryFacts {
    bindings: alloc::vec::Vec<OrdinaryBinding>,
}

impl OrdinaryFacts {
    /// Runtime imports and direct top-level declarations, in source order.
    #[must_use]
    pub fn bindings(&self) -> &[OrdinaryBinding] {
        &self.bindings
    }
}

/// Why the observer cannot claim facts for the complete Program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryFactsIncomplete {
    /// The walk never started a Program.
    NoProgram,
    /// One observer saw a second Program.
    SecondProgram,
    /// Some top-level statement produced no event.
    MissingStatements { observed: usize, expected: usize },
    /// A binding span left the 32-bit file coordinate space.
    SpanOverflow,
}

#[derive(Debug, Clone, Copy)]
struct Program {
    scope: ScopeId,
    base: u32,
    statements: usize,
}

/// Collects normal-script bindings from the existing Program walk.
#[derive(Debug)]
pub struct OrdinaryObserver<'a> {
    program: Option<Program>,
    second_program: bool,
    observed: usize,
    bindings: Bindings<'a>,
}

impl Default for OrdinaryObserver<'_> {
    fn default() -> Self {
        Self {
            program: None,
            second_program: false,
            observed: 0,
            bindings: Bindings::ordinary(),
        }
    }
}

impl<'a> OrdinaryObserver<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> Result<OrdinaryFacts, OrdinaryFactsIncomplete> {
        let Some(program) = self.program else {
            return Err(OrdinaryFactsIncomplete::NoProgram);
        };
        if self.second_program {
            return Err(OrdinaryFactsIncomplete::SecondProgram);
        }
        if self.bindings.overflow {
            return Err(OrdinaryFactsIncomplete::SpanOverflow);
        }
        if self.observed != program.statements {
            return Err(OrdinaryFactsIncomplete::MissingStatements {
                observed: self.observed,
                expected: program.statements,
            });
        }
        Ok(OrdinaryFacts {
            bindings: self.bindings.rows,
        })
    }
}

impl<'a> FileObserver<'a> for OrdinaryObserver<'a> {
    type Checkpoint = OrdinaryCheckpoint;

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
        self.bindings.statement(program.base, event.statement);
    }

    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}

    fn checkpoint(&self) -> Self::Checkpoint {
        OrdinaryCheckpoint {
            observed: self.observed,
            bindings: self.bindings.rows.len(),
        }
    }

    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }

    fn rollback(&mut self, checkpoint: Self::Checkpoint) {
        self.observed = checkpoint.observed;
        self.bindings.rows.truncate(checkpoint.bindings);
    }
}

/// Row counts before a walk subtree; one statement owns every added row.
#[derive(Debug, Clone, Copy)]
pub struct OrdinaryCheckpoint {
    observed: usize,
    bindings: usize,
}
