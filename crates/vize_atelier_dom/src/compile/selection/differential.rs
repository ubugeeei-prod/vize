use core::cell::Cell;

use vize_l4::targets::dom::artifact::EmitError;

std::thread_local! {
    static FORCE_LEGACY: Cell<bool> = const { Cell::new(false) };
    static EMIT_REFUSAL: Cell<Option<EmitError>> = const { Cell::new(None) };
}

/// Historical arm. Projectable script tables are admitted in
/// production, so this no longer changes the selector.
pub fn with_croquis_projection<R>(f: impl FnOnce() -> R) -> R {
    f()
}

/// Run `f` with every DOM compile on this thread on the legacy lane.
pub fn with_legacy_lane<R>(f: impl FnOnce() -> R) -> R {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            FORCE_LEGACY.with(|flag| flag.set(self.0));
        }
    }
    let _reset = Reset(FORCE_LEGACY.with(|flag| flag.replace(true)));
    f()
}

#[doc(hidden)]
pub fn legacy_forced() -> bool {
    FORCE_LEGACY.with(Cell::get)
}

/// Take the typed error L2 returned for this thread's latest DOM
/// compile, so a corpus sweep can attribute `emit_refused` templates to
/// their [`vize_l4::targets::dom::artifact::UnsupportedReason`] bucket. `None` when L2
/// emitted the module or the selector never ran it.
pub fn take_emit_refusal() -> Option<EmitError> {
    EMIT_REFUSAL.with(Cell::take)
}

#[doc(hidden)]
pub fn note_emit_refusal(error: EmitError) {
    EMIT_REFUSAL.with(|slot| slot.set(Some(error)));
}
