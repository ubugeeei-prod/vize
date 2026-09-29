//! Invocation-local SFC slot-scope policy without changing public option literals.

use super::{
    DomEmit, DomEmitOptions, EmitError, LegacyCaps, NoObserver, ObservedDomEmit,
    emit_dom_source_observed_with_slot_policy_captured,
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
    emit_dom_source_observed_with_slot_policy_captured(
        allocator,
        source,
        caps,
        options,
        strict_slot_params,
        no_slotted,
        capture,
    )
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
    emit_dom_source_with_options_and_observer_captured(
        allocator,
        source,
        caps,
        options,
        &mut NoObserver,
        strict_slot_params,
        no_slotted,
        capture,
    )
    .map(|observed| observed.emit)
}
