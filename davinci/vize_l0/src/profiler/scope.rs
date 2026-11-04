//! Thread-local profiler scope.
//!
//! DOM compile accounting reads [`with_active_profiler`]. A test installs its
//! own [`Profiler`] around one compile so that compile's counters cannot see,
//! and cannot be seen by, other tests in the same process. [`global_profiler`]
//! stays process-global for callers that do not install a scope.

use std::cell::Cell;
use std::marker::PhantomData;

use super::core::Profiler;
use super::global_profiler;

thread_local! {
    static ACTIVE: Cell<*const Profiler> = const { Cell::new(std::ptr::null()) };
}

/// Restores the previous thread profiler when dropped.
///
/// The guard is `!Send`: the installed pointer is only valid on the installing
/// thread, and dropping it on another thread would restore the wrong slot.
#[must_use = "the previous profiler is restored when this guard drops"]
pub struct ProfilerScope<'a> {
    previous: *const Profiler,
    _borrow: PhantomData<&'a Profiler>,
}

impl<'a> ProfilerScope<'a> {
    /// Install `profiler` as this thread's active profiler until the guard drops.
    ///
    /// Nested scopes restore the outer profiler. Threads without a scope keep
    /// using the process-global profiler.
    pub fn install(profiler: &'a Profiler) -> Self {
        let ptr = &raw const *profiler;
        let previous = ACTIVE.with(|slot| slot.replace(ptr));
        Self {
            previous,
            _borrow: PhantomData,
        }
    }
}

impl Drop for ProfilerScope<'_> {
    fn drop(&mut self) {
        ACTIVE.with(|slot| slot.set(self.previous));
    }
}

/// Run `body` with this thread's installed profiler, or the process-global one.
#[inline]
pub fn with_active_profiler<R>(body: impl FnOnce(&Profiler) -> R) -> R {
    ACTIVE.with(|slot| {
        let ptr = slot.get();
        if ptr.is_null() {
            body(global_profiler())
        } else {
            // SAFETY: `ProfilerScope::install` stored this pointer from a live
            // `&Profiler`, and its `Drop` clears the slot before that borrow
            // ends. The guard is `!Send`, so only this thread can observe it.
            body(unsafe { &*ptr })
        }
    })
}

/// Whether the active profiler will record counters for this compile.
#[inline]
pub fn active_profiler_is_enabled() -> bool {
    with_active_profiler(Profiler::is_enabled)
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::{ProfilerScope, active_profiler_is_enabled, global_profiler, with_active_profiler};
    use crate::profiler::{CounterSummary, Profiler};

    fn total(profiler: &Profiler, name: &str) -> Option<u64> {
        counter_total(&profiler.counter_summary(), name)
    }

    fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
        counters
            .entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.total)
    }

    #[test]
    fn scoped_counter_stays_on_the_installed_profiler() {
        let local = Profiler::new();
        local.set_enabled(true);
        let other = Profiler::new();
        other.set_enabled(true);

        {
            let _scope = ProfilerScope::install(&local);
            assert!(with_active_profiler(|profiler| ptr::eq(profiler, &local)));
            with_active_profiler(|profiler| {
                profiler.record_counter_enabled("profiler.scope.probe", 1);
            });
        }

        assert_eq!(total(&local, "profiler.scope.probe"), Some(1));
        assert_eq!(total(&other, "profiler.scope.probe"), None);
        assert!(with_active_profiler(|profiler| ptr::eq(
            profiler,
            global_profiler()
        )));

        {
            let _scope = ProfilerScope::install(&other);
            with_active_profiler(|profiler| {
                profiler.record_counter_enabled("profiler.scope.probe", 1);
                profiler.record_counter_enabled("profiler.scope.probe", 1);
            });
        }

        assert_eq!(total(&local, "profiler.scope.probe"), Some(1));
        assert_eq!(total(&other, "profiler.scope.probe"), Some(2));
    }

    #[test]
    fn nested_scope_restores_the_outer_profiler() {
        let outer = Profiler::new();
        outer.set_enabled(true);
        let inner = Profiler::new();
        inner.set_enabled(true);
        let _outer_scope = ProfilerScope::install(&outer);
        with_active_profiler(|profiler| profiler.record_counter_enabled("profiler.scope.probe", 1));
        {
            let _inner_scope = ProfilerScope::install(&inner);
            assert!(with_active_profiler(|profiler| ptr::eq(profiler, &inner)));
            with_active_profiler(|profiler| {
                profiler.record_counter_enabled("profiler.scope.probe", 1);
            });
        }
        with_active_profiler(|profiler| profiler.record_counter_enabled("profiler.scope.probe", 1));
        assert!(with_active_profiler(|profiler| ptr::eq(profiler, &outer)));
        assert_eq!(total(&outer, "profiler.scope.probe"), Some(2));
        assert_eq!(total(&inner, "profiler.scope.probe"), Some(1));
    }

    #[test]
    fn disabled_scope_is_local_to_this_thread() {
        let local = Profiler::new();
        let _scope = ProfilerScope::install(&local);
        assert!(!active_profiler_is_enabled());
        assert!(with_active_profiler(|profiler| ptr::eq(profiler, &local)));

        let threaded = std::thread::spawn(|| {
            let profiler = Profiler::new();
            profiler.set_enabled(true);
            let _scope = ProfilerScope::install(&profiler);
            assert!(active_profiler_is_enabled());
            with_active_profiler(|active| {
                active.record_counter_enabled("profiler.scope.probe", 1);
            });
            total(&profiler, "profiler.scope.probe")
        });

        assert!(matches!(threaded.join(), Ok(Some(1))));
        assert_eq!(total(&local, "profiler.scope.probe"), None);
    }
}
