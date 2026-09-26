//! Scoped retained selection for same-process production benchmark pairs.
//! Compiled out unless the explicit `davinci-benchmark` feature is enabled.

use super::VaporCompilerOptions;
use core::cell::Cell;

std::thread_local! {
    static RETAINED: Cell<bool> = const { Cell::new(false) };
}

/// Pin every Vapor compile on this thread to the retained lane. Nested calls
/// and unwinding restore the caller's selection without changing any options.
pub fn with_retained_lane<R>(f: impl FnOnce() -> R) -> R {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            RETAINED.with(|flag| flag.set(self.0));
        }
    }
    let _reset = Reset(RETAINED.with(|flag| flag.replace(true)));
    f()
}

pub(super) fn apply(mut options: VaporCompilerOptions) -> VaporCompilerOptions {
    options.davinci_retained_lane |= RETAINED.with(Cell::get);
    options
}

#[cfg(test)]
mod tests {
    use super::{VaporCompilerOptions, apply, with_retained_lane};

    #[test]
    fn retained_selector_restores_nested_and_unwound_scopes() {
        let selected = || apply(VaporCompilerOptions::default()).davinci_retained_lane;
        assert!(!selected());
        with_retained_lane(|| {
            assert!(selected());
            with_retained_lane(|| assert!(selected()));
            assert!(selected());
        });
        assert!(!selected());
        let unwound = std::panic::catch_unwind(|| {
            with_retained_lane(|| panic!("benchmark scope unwinding"));
        });
        assert!(unwound.is_err());
        assert!(!selected());
        assert!(
            apply(VaporCompilerOptions {
                davinci_retained_lane: true,
                ..Default::default()
            })
            .davinci_retained_lane
        );
    }
}
