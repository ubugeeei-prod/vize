//! One-shot, benchmark-only instruction windows. Setup and the returned
//! value's destruction stay outside the window, matching allocation probes.
//! Callgrind starts with instrumentation off; each window produces one named
//! dump and resets its counters. Criterion sampling is bypassed entirely.

#[cfg(any(
    test,
    all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )
))]
use std::ffi::CStr;
use std::ffi::CString;

/// Whether the caller selected instruction measurements.
#[expect(
    clippy::panic,
    reason = "benchmark tooling rejects an invalid measurement mode"
)]
pub fn requested() -> bool {
    match std::env::var("VIZE_INSTRUCTION_COUNTS") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "1" => true,
        _ => panic!("VIZE_INSTRUCTION_COUNTS must be absent or 1"),
    }
}

/// Reject a requested measurement without real instrumentation.
#[cfg_attr(
    not(all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )),
    expect(
        clippy::panic,
        reason = "benchmark tooling fails unsupported measurement configurations"
    )
)]
pub fn initialize() {
    if !requested() {
        return;
    }
    #[cfg(all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    ))]
    {
        assert!(
            valgrind_requests::valgrind::running_on_valgrind() > 0,
            "instruction measurements must run under Valgrind"
        );
        // Reserve is configured before process startup by the driver. Register
        // this allocator-only range before any window so its lazy page-map
        // submaps do not move between product probes with randomized addresses.
        // No product routine is run as warmup, and the allocation is dropped
        // while instrumentation is still off.
        const PREINITIALIZED_BYTES: usize = 64 * 1024 * 1024;
        let buffer = Vec::<u8>::with_capacity(PREINITIALIZED_BYTES);
        assert_eq!(buffer.capacity(), PREINITIALIZED_BYTES);
        drop(core::hint::black_box(buffer));
        eprintln!(
            "VIZE_INSTRUCTION_ALLOCATOR {}",
            serde_json::json!({"preinitialized_bytes": PREINITIALIZED_BYTES})
        );
    }
    #[cfg(not(all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )))]
    panic!("instruction measurements require instruction-counts on linux x86_64");
}

#[cfg(any(
    test,
    all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )
))]
trait Client {
    fn start(&self);
    fn stop(&self);
    fn dump(&self, id: &CStr);
}

#[cfg(any(
    test,
    all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )
))]
struct Window<'a, C: Client>(&'a C);

#[cfg(any(
    test,
    all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )
))]
impl<C: Client> Drop for Window<'_, C> {
    fn drop(&mut self) {
        self.0.stop();
    }
}

#[cfg(any(
    test,
    all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )
))]
fn measure_with<T, C: Client>(client: &C, id: &CStr, routine: impl FnOnce() -> T) -> T {
    client.start();
    let window = Window(client);
    let value = run_once(routine);
    drop(window);
    client.dump(id);
    value
}

// Starting instrumentation does not rewrite the basic block already running.
// A real call boundary makes even a constant-folded routine enter a newly
// instrumented block. This fixed benchmark-only overhead is in every budget.
#[cfg(any(
    test,
    all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )
))]
#[inline(never)]
fn run_once<T>(routine: impl FnOnce() -> T) -> T {
    core::hint::black_box(routine())
}

#[cfg(all(
    feature = "instruction-counts",
    target_os = "linux",
    target_arch = "x86_64"
))]
struct Callgrind;

#[cfg(all(
    feature = "instruction-counts",
    target_os = "linux",
    target_arch = "x86_64"
))]
impl Client for Callgrind {
    fn start(&self) {
        valgrind_requests::callgrind::start_instrumentation();
    }
    fn stop(&self) {
        valgrind_requests::callgrind::stop_instrumentation();
    }
    fn dump(&self, id: &CStr) {
        valgrind_requests::callgrind::dump_stats_at(id);
    }
}

/// Run exactly one routine between client start/stop requests.
#[cfg_attr(
    not(all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )),
    expect(
        clippy::panic,
        reason = "benchmark tooling cannot silently skip unsupported measurement windows"
    )
)]
pub fn measure<T>(bench_id: &str, routine: impl FnOnce() -> T) -> T {
    let id = CString::new(bench_id).expect("validated bench ids contain no NUL");
    #[cfg(all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    ))]
    {
        measure_with(&Callgrind, &id, routine)
    }
    #[cfg(not(all(
        feature = "instruction-counts",
        target_os = "linux",
        target_arch = "x86_64"
    )))]
    {
        let _ = (id, routine);
        panic!("instruction measurement unavailable on this build");
    }
}

/// Emit identity outside the measured window for the strict dump reconciler.
pub fn record(bench_id: &str, fixture: &str, window: &str) {
    eprintln!(
        "VIZE_INSTRUCTION_BENCH {}",
        serde_json::json!({
            "bench_id": bench_id, "fixture": fixture, "window": window,
        })
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Fake<'a>(&'a RefCell<Vec<&'static str>>);
    impl Client for Fake<'_> {
        fn start(&self) {
            self.0.borrow_mut().push("start");
        }
        fn stop(&self) {
            self.0.borrow_mut().push("stop");
        }
        fn dump(&self, _: &CStr) {
            self.0.borrow_mut().push("dump");
        }
    }
    struct ResultValue<'a>(&'a RefCell<Vec<&'static str>>);
    impl Drop for ResultValue<'_> {
        fn drop(&mut self) {
            self.0.borrow_mut().push("drop-result");
        }
    }

    #[test]
    fn setup_and_returned_value_drop_stay_outside_the_window() {
        let events = RefCell::new(vec!["setup"]);
        let client = Fake(&events);
        let result = measure_with(&client, c"fixture", || {
            events.borrow_mut().push("stage");
            ResultValue(&events)
        });
        assert_eq!(
            *events.borrow(),
            ["setup", "start", "stage", "stop", "dump"]
        );
        drop(result);
        assert_eq!(
            *events.borrow(),
            ["setup", "start", "stage", "stop", "dump", "drop-result"]
        );
    }

    #[test]
    fn a_panicking_stage_stops_collection_without_publishing_a_dump() {
        let events = RefCell::new(Vec::new());
        let client = Fake(&events);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            measure_with(&client, c"fixture", || panic!("broken stage"));
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["start", "stop"]);
    }
}
