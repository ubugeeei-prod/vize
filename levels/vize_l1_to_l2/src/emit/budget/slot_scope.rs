//! Invocation-local SFC slot-scope policy without changing public option literals.

use super::{
    DomEmit, DomEmitOptions, EmitError, EmitInvocation, LegacyCaps, NoObserver, ObservedDomEmit,
    emit_dom_source_with_options_and_observer_captured,
};
use vize_l0::Allocator;
use vize_l0::dump::capture::CaptureSink;

/// Observe SFC emission with its own slot-scope policy.
#[doc(hidden)]
pub fn emit_dom_source_observed_with_options_captured_and_slot_scope<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    capture: &mut C,
) -> Result<ObservedDomEmit, EmitError> {
    emit_dom_source_observed_with_options_captured_slot_scope_and_environment(
        allocator,
        source,
        caps,
        options,
        strict_slot_params,
        no_slotted,
        true,
        capture,
    )
}

/// Observe SFC emission with its build environment outside published options.
#[doc(hidden)]
#[expect(clippy::too_many_arguments, reason = "SFC emission inputs")]
pub fn emit_dom_source_observed_with_options_captured_slot_scope_and_environment<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    development: bool,
    capture: &mut C,
) -> Result<ObservedDomEmit, EmitError> {
    let mut transform = vize_l0::pass::BudgetObserver::new();
    let observed = emit_dom_source_with_options_and_observer_captured(
        allocator,
        source,
        caps,
        EmitInvocation {
            options,
            strict_slot_params,
            no_slotted,
            development,
            preflight: None,
        },
        &mut transform,
        capture,
    )?;
    Ok(ObservedDomEmit {
        emit: observed.emit,
        budget: super::DomEmitBudget {
            transform,
            emit_walks: 1,
            emit_visits: observed.emit_visits,
        },
    })
}

/// Emit with SFC slot scope outside the published `DomEmitOptions` literal.
#[doc(hidden)]
pub fn emit_dom_source_with_options_captured_and_slot_scope<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    capture: &mut C,
) -> Result<DomEmit, EmitError> {
    emit_dom_source_with_options_captured_slot_scope_and_environment(
        allocator,
        source,
        caps,
        options,
        strict_slot_params,
        no_slotted,
        true,
        capture,
    )
}

/// Emit with SFC slot scope and build environment outside published options.
#[doc(hidden)]
#[expect(clippy::too_many_arguments, reason = "SFC emission inputs")]
pub fn emit_dom_source_with_options_captured_slot_scope_and_environment<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    development: bool,
    capture: &mut C,
) -> Result<DomEmit, EmitError> {
    emit_dom_source_with_options_and_observer_captured(
        allocator,
        source,
        caps,
        EmitInvocation {
            options,
            strict_slot_params,
            no_slotted,
            development,
            preflight: None,
        },
        &mut NoObserver,
        capture,
    )
    .map(|observed| observed.emit)
}
