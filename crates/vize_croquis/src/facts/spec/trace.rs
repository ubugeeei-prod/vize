//! The `UndefinedRefs` spec's input relation: every template expression the
//! drawer checks for unresolved reads, recorded where it is checked.
//!
//! Recording is compiled only into debug builds and armed per thread by
//! [`record`], so release builds carry no trace code and an unarmed debug
//! build pays one thread-local flag read per checked expression. This is
//! the Polonius shape: the front end emits input facts, and the naive
//! evaluator ([`super::undefined_refs::evaluate`]) re-derives the relation
//! from them without the production walk's identifier cache, its walk-time
//! binding snapshot or its scope-chain cursor.

use vize_carton::CompactString;

use crate::scope::ScopeId;

/// One checked template expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedExpression {
    /// The expression text as the drawer read it.
    pub content: CompactString,
    /// Template offset of the expression's first byte.
    pub base_offset: u32,
    /// Names the enclosing element structure binds (`v-for`, `v-slot`).
    pub scope_vars: Vec<CompactString>,
    /// The scope the drawer resolved names from.
    pub scope: ScopeId,
}

#[cfg(debug_assertions)]
mod recorder {
    use core::cell::{Cell, RefCell};

    use super::CheckedExpression;

    std::thread_local! {
        static ARMED: Cell<bool> = const { Cell::new(false) };
        static TRACE: RefCell<Vec<CheckedExpression>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn armed() -> bool {
        ARMED.with(Cell::get)
    }

    pub(crate) fn push(expression: CheckedExpression) {
        TRACE.with(|trace| trace.borrow_mut().push(expression));
    }

    pub(super) fn arm(on: bool) {
        ARMED.with(|armed| armed.set(on));
        TRACE.with(|trace| trace.borrow_mut().clear());
    }

    pub(super) fn take() -> Vec<CheckedExpression> {
        TRACE.with(|trace| core::mem::take(&mut *trace.borrow_mut()))
    }
}

#[cfg(debug_assertions)]
pub(crate) use recorder::{armed, push};

/// Run `analysis` with the trace armed on this thread and return its result
/// with every expression it checked, in check order.
///
/// Debug builds only: in a release build the drawer carries no recorder, so
/// the returned trace is `None` and a spec run must not claim a comparison.
pub fn record<R>(analysis: impl FnOnce() -> R) -> (R, Option<Vec<CheckedExpression>>) {
    #[cfg(debug_assertions)]
    {
        recorder::arm(true);
        let result = analysis();
        let trace = recorder::take();
        recorder::arm(false);
        (result, Some(trace))
    }
    #[cfg(not(debug_assertions))]
    {
        (analysis(), None)
    }
}
