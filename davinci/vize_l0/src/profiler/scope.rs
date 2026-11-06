//! Thread-local profiler scope.
//!
//! DOM compile accounting reads [`with_active_profiler`]. A test installs its
//! own [`Profiler`] around one compile so that compile's counters cannot see,
//! and cannot be seen by, other tests in the same process. [`global_profiler`]
//! stays process-global for callers that do not install a scope.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::core::Profiler;
use super::global_profiler;

struct ActiveProfiler {
    profiler: Arc<Profiler>,
    token: Rc<()>,
}

thread_local! {
    static ACTIVE: RefCell<Vec<ActiveProfiler>> = const { RefCell::new(Vec::new()) };
}

/// Removes this thread's installed profiler when dropped.
///
/// The guard and its installed entry share an `Rc` token, which keeps the guard
/// `!Send` and lets out-of-order drops remove the correct entry. The entry owns
/// an `Arc` clone, so forgetting the guard cannot leave a dangling reference.
#[must_use = "the installed profiler is removed when this guard drops"]
pub struct ProfilerScope {
    token: Rc<()>,
}

impl ProfilerScope {
    /// Install `profiler` as this thread's active profiler until the guard drops.
    ///
    /// Nested scopes restore the outer profiler. Threads without a scope keep
    /// using the process-global profiler.
    pub fn install(profiler: &Arc<Profiler>) -> Self {
        let token = Rc::new(());
        ACTIVE.with(|active| {
            active.borrow_mut().push(ActiveProfiler {
                profiler: Arc::clone(profiler),
                token: Rc::clone(&token),
            });
        });
        Self { token }
    }
}

impl Drop for ProfilerScope {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if let Some(index) = active
                .iter()
                .position(|entry| Rc::ptr_eq(&entry.token, &self.token))
            {
                active.remove(index);
            }
        });
    }
}

/// Run `body` with this thread's installed profiler, or the process-global one.
#[inline]
pub fn with_active_profiler<R>(body: impl FnOnce(&Profiler) -> R) -> R {
    let profiler = ACTIVE.with(|active| {
        active
            .borrow()
            .last()
            .map(|entry| Arc::clone(&entry.profiler))
    });
    match profiler {
        Some(profiler) => body(&profiler),
        None => body(global_profiler()),
    }
}

/// Whether the active profiler will record counters for this compile.
#[inline]
pub fn active_profiler_is_enabled() -> bool {
    with_active_profiler(Profiler::is_enabled)
}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::ptr;
    use std::sync::Arc;

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
        let local = Arc::new(Profiler::new());
        local.set_enabled(true);
        let other = Arc::new(Profiler::new());
        other.set_enabled(true);

        {
            let _scope = ProfilerScope::install(&local);
            assert!(with_active_profiler(|profiler| ptr::eq(profiler, &*local)));
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
        let outer = Arc::new(Profiler::new());
        outer.set_enabled(true);
        let inner = Arc::new(Profiler::new());
        inner.set_enabled(true);
        let _outer_scope = ProfilerScope::install(&outer);
        with_active_profiler(|profiler| profiler.record_counter_enabled("profiler.scope.probe", 1));
        {
            let _inner_scope = ProfilerScope::install(&inner);
            assert!(with_active_profiler(|profiler| ptr::eq(profiler, &*inner)));
            with_active_profiler(|profiler| {
                profiler.record_counter_enabled("profiler.scope.probe", 1);
            });
        }
        with_active_profiler(|profiler| profiler.record_counter_enabled("profiler.scope.probe", 1));
        assert!(with_active_profiler(|profiler| ptr::eq(profiler, &*outer)));
        assert_eq!(total(&outer, "profiler.scope.probe"), Some(2));
        assert_eq!(total(&inner, "profiler.scope.probe"), Some(1));
    }

    #[test]
    fn disabled_scope_is_local_to_this_thread() {
        let local = Arc::new(Profiler::new());
        let _scope = ProfilerScope::install(&local);
        assert!(!active_profiler_is_enabled());
        assert!(with_active_profiler(|profiler| ptr::eq(profiler, &*local)));

        let threaded = std::thread::spawn(|| {
            let profiler = Arc::new(Profiler::new());
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

    #[test]
    fn forgotten_guard_keeps_profiler_alive_until_thread_exit() {
        let weak = std::thread::spawn(|| {
            let profiler = Arc::new(Profiler::new());
            profiler.set_enabled(true);
            let weak = Arc::downgrade(&profiler);
            let guard = ProfilerScope::install(&profiler);

            mem::forget(guard);
            drop(profiler);

            let retained = weak.upgrade().expect("installed profiler stays alive");
            assert!(with_active_profiler(|active| ptr::eq(active, &*retained)));
            weak
        })
        .join()
        .expect("profiler thread should finish");

        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn dropping_outer_scope_first_does_not_resurrect_it() {
        let outer = Arc::new(Profiler::new());
        let outer_weak = Arc::downgrade(&outer);
        let outer_scope = ProfilerScope::install(&outer);
        let inner = Arc::new(Profiler::new());
        let inner_scope = ProfilerScope::install(&inner);

        drop(outer_scope);
        drop(outer);
        assert!(outer_weak.upgrade().is_none());
        assert!(with_active_profiler(|active| ptr::eq(active, &*inner)));

        drop(inner_scope);
        assert!(with_active_profiler(|active| ptr::eq(
            active,
            global_profiler()
        )));
        assert!(outer_weak.upgrade().is_none());
    }
}
