//! Borrowed surface validation at the actual parse boundary.

use super::{DomEmitBudget, EmitInvocation, emit_dom_source_with_options_and_observer_captured};
use crate::{DomEmit, DomEmitOptions, EmitError, LegacyCaps};
use vize_l0::{
    Allocator,
    dump::capture::CaptureSink,
    pass::{BudgetObserver, NoObserver},
};
use vize_l1::{SurfaceError, SurfaceTree};

/// Invocation-local validation owned by the consumer, not the compiler stage.
/// A false result stops before lowering; diagnostics stay with that consumer.
/// None is reserved for a consumer's actual compatibility parse entry.
/// This does not select the emitter used after that parse.
#[doc(hidden)]
pub trait SurfacePreflight {
    fn check(&mut self, surface: Option<(&SurfaceTree<'_>, &[SurfaceError])>) -> bool;
}

/// Reuse the exact parsed surface; ordinary entry points install no hook.
/// NoObserver remains the pass observer with traversal accounting disabled.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "existing source inputs and invocation-local validation"
)]
pub fn emit_dom_source_with_preflight<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    preflight: &mut dyn SurfacePreflight,
    observe: bool,
    capture: &mut C,
) -> Result<(DomEmit, Option<DomEmitBudget>), EmitError> {
    let invocation = EmitInvocation {
        options,
        strict_slot_params,
        no_slotted,
        preflight: Some(preflight),
    };
    if observe {
        let mut transform = BudgetObserver::new();
        let observed = emit_dom_source_with_options_and_observer_captured(
            allocator,
            source,
            caps,
            invocation,
            &mut transform,
            capture,
        )?;
        Ok((
            observed.emit,
            Some(DomEmitBudget {
                transform,
                emit_walks: 1,
                emit_visits: observed.emit_visits,
            }),
        ))
    } else {
        let observed = emit_dom_source_with_options_and_observer_captured(
            allocator,
            source,
            caps,
            invocation,
            &mut NoObserver,
            capture,
        )?;
        Ok((observed.emit, None))
    }
}

#[cfg(test)]
mod tests;
