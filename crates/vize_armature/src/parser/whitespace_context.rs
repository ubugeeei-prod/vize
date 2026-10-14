//! Scoped whitespace strategy for FFI compile entry points.
//!
//! The Rust compiler option structs remain constructible for existing callers.
//! A synchronous native compile can instead scope a strategy across the DOM,
//! SSR, or Vapor parser it invokes. Batch workers each own their thread-local
//! scope, and nested compiles restore the previous value on exit.

use std::cell::Cell;
use vize_relief::WhitespaceStrategy;

std::thread_local! {
    static OVERRIDE: Cell<Option<WhitespaceMode>> = const { Cell::new(None) };
}

#[derive(Clone, Copy)]
struct WhitespaceMode {
    strategy: WhitespaceStrategy,
    legacy_line_breaks: bool,
    vue2_preserve: bool,
}

/// Run a synchronous compilation with a parser whitespace strategy.
pub fn with_whitespace_strategy<R>(strategy: WhitespaceStrategy, compile: impl FnOnce() -> R) -> R {
    with_whitespace_mode(strategy, false, compile)
}

/// Run a synchronous compilation with an opt-in Vue 2 migration line break mode.
/// This mode only changes whitespace-only segments after text/interpolations.
pub fn with_whitespace_mode<R>(
    strategy: WhitespaceStrategy,
    legacy_line_breaks: bool,
    compile: impl FnOnce() -> R,
) -> R {
    with_scoped_mode(
        WhitespaceMode {
            strategy,
            legacy_line_breaks,
            vue2_preserve: false,
        },
        compile,
    )
}

/// Run a synchronous compilation with Vue 2.7's `whitespace: "preserve"`.
///
/// Vue 2 keeps text and interpolations in one text node, so whitespace next to
/// an interpolation survives verbatim; only whitespace-only runs between
/// elements are condensed (or dropped at the edges of a child list). The
/// strategy reported to later stages is `Preserve`, so every stage that
/// special-cases preserve keeps doing so.
pub fn with_vue2_preserve_whitespace<R>(compile: impl FnOnce() -> R) -> R {
    with_scoped_mode(
        WhitespaceMode {
            strategy: WhitespaceStrategy::Preserve,
            legacy_line_breaks: false,
            vue2_preserve: true,
        },
        compile,
    )
}

fn with_scoped_mode<R>(mode: WhitespaceMode, compile: impl FnOnce() -> R) -> R {
    struct Reset(Option<WhitespaceMode>);
    impl Drop for Reset {
        fn drop(&mut self) {
            OVERRIDE.with(|cell| cell.set(self.0));
        }
    }
    let _reset = Reset(OVERRIDE.with(|cell| cell.replace(Some(mode))));
    compile()
}

/// Return the active scoped strategy, or the caller's normal parser option.
#[inline]
pub fn current_whitespace_strategy(default: WhitespaceStrategy) -> WhitespaceStrategy {
    OVERRIDE.with(|cell| cell.get().map_or(default, |mode| mode.strategy))
}

/// Whether the current compile preserves authored line breaks following text.
#[inline]
pub fn current_legacy_line_breaks() -> bool {
    OVERRIDE.with(|cell| cell.get().is_some_and(|mode| mode.legacy_line_breaks))
}

/// Whether the current compile follows Vue 2.7's `preserve` text rules.
#[inline]
pub fn current_vue2_preserve() -> bool {
    OVERRIDE.with(|cell| cell.get().is_some_and(|mode| mode.vue2_preserve))
}
