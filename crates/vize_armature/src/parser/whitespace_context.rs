//! Scoped whitespace strategy for FFI compile entry points.
//!
//! The Rust compiler option structs remain constructible for existing callers.
//! A synchronous native compile can instead scope a strategy across the DOM,
//! SSR, or Vapor parser it invokes. Batch workers each own their thread-local
//! scope, and nested compiles restore the previous value on exit.

use std::cell::Cell;
use vize_relief::WhitespaceStrategy;

std::thread_local! {
    static OVERRIDE: Cell<Option<WhitespaceStrategy>> = const { Cell::new(None) };
}

/// Run a synchronous compilation with a parser whitespace strategy.
pub fn with_whitespace_strategy<R>(strategy: WhitespaceStrategy, compile: impl FnOnce() -> R) -> R {
    struct Reset(Option<WhitespaceStrategy>);
    impl Drop for Reset {
        fn drop(&mut self) {
            OVERRIDE.with(|cell| cell.set(self.0));
        }
    }
    let _reset = Reset(OVERRIDE.with(|cell| cell.replace(Some(strategy))));
    compile()
}

/// Return the active scoped strategy, or the caller's normal parser option.
#[inline]
pub fn current_whitespace_strategy(default: WhitespaceStrategy) -> WhitespaceStrategy {
    OVERRIDE.with(|cell| cell.get().unwrap_or(default))
}
