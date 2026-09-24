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
    struct Reset(Option<WhitespaceMode>);
    impl Drop for Reset {
        fn drop(&mut self) {
            OVERRIDE.with(|cell| cell.set(self.0));
        }
    }
    let _reset = Reset(OVERRIDE.with(|cell| {
        cell.replace(Some(WhitespaceMode {
            strategy,
            legacy_line_breaks,
        }))
    }));
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
