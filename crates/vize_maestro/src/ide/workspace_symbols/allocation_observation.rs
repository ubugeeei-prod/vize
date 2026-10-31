//! Temporary diagnostic only: existing allocator counters for the first two scans.
//! Global snapshots include concurrent Rust work and are not atomic live-heap reads.

use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use vize_l0::profiler::{ProfileGuard, allocation_snapshot, global_profiler};

static REQUESTS: AtomicUsize = AtomicUsize::new(0);
static ORIGIN: OnceLock<Instant> = OnceLock::new();

thread_local! {
    static SCANNING: Cell<Option<Request>> = const { Cell::new(None) };
}

#[derive(Clone, Copy)]
pub(crate) enum Request {
    First,
    Second,
}

pub(crate) enum Region {
    Descriptor,
    Script,
    Style,
    Contract,
}

pub(crate) fn begin_request() -> Option<Request> {
    if !global_profiler().is_enabled() {
        return None;
    }
    let request = match REQUESTS.fetch_add(1, Ordering::Relaxed) {
        0 => Request::First,
        1 => Request::Second,
        _ => return None,
    };
    snapshot(request, false);
    Some(request)
}

pub(crate) fn finish_request(request: Option<Request>) {
    if let Some(request) = request {
        snapshot(request, true);
    }
}

struct RestoreScan(Option<Request>);

impl Drop for RestoreScan {
    fn drop(&mut self) {
        SCANNING.with(|scan| scan.set(self.0));
    }
}

pub(crate) struct WorkerScope {
    // Fields drop in declaration order: finish the span before restoring TLS.
    _span: Option<ProfileGuard>,
    _restore: RestoreScan,
}

pub(crate) fn worker_scope(request: Option<Request>) -> Option<WorkerScope> {
    let request = request?;
    let restore = RestoreScan(SCANNING.with(|scan| scan.replace(Some(request))));
    let name = match request {
        Request::First => "diagnostic.workspace_symbols.scan1.worker",
        Request::Second => "diagnostic.workspace_symbols.scan2.worker",
    };
    Some(WorkerScope {
        _span: global_profiler().global_span(name),
        _restore: restore,
    })
}

/// Existing spans count successful allocation-like requested bytes on this
/// thread only. They do not count deallocation or allocator-resident pages.
pub(crate) fn region(region: Region) -> Option<ProfileGuard> {
    let request = SCANNING.with(Cell::get)?;
    let name = match (request, region) {
        (Request::First, Region::Descriptor) => "diagnostic.workspace_symbols.scan1.descriptor",
        (Request::First, Region::Script) => "diagnostic.workspace_symbols.scan1.script",
        (Request::First, Region::Style) => "diagnostic.workspace_symbols.scan1.style",
        (Request::First, Region::Contract) => "diagnostic.workspace_symbols.scan1.contract",
        (Request::Second, Region::Descriptor) => "diagnostic.workspace_symbols.scan2.descriptor",
        (Request::Second, Region::Script) => "diagnostic.workspace_symbols.scan2.script",
        (Request::Second, Region::Style) => "diagnostic.workspace_symbols.scan2.style",
        (Request::Second, Region::Contract) => "diagnostic.workspace_symbols.scan2.contract",
    };
    global_profiler().global_span(name)
}

fn snapshot(request: Request, finished: bool) {
    let origin = ORIGIN.get_or_init(Instant::now);
    let before_ns = u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let allocation = allocation_snapshot();
    let after_ns = u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let profiler = global_profiler();
    // Every name gets exactly one sample. record_counter suppresses its own
    // profiler allocations; the relaxed snapshot fields may interleave with
    // allocations/deallocations on other threads.
    macro_rules! record {
        ($prefix:literal) => {{
            profiler.record_counter(concat!($prefix, ".read_before_ns"), before_ns);
            profiler.record_counter(concat!($prefix, ".read_after_ns"), after_ns);
            profiler.record_counter(concat!($prefix, ".alloc_calls"), allocation.alloc_calls);
            profiler.record_counter(
                concat!($prefix, ".alloc_zeroed_calls"),
                allocation.alloc_zeroed_calls,
            );
            profiler.record_counter(
                concat!($prefix, ".alloc_failures"),
                allocation.alloc_failures,
            );
            profiler.record_counter(
                concat!($prefix, ".alloc_zeroed_failures"),
                allocation.alloc_zeroed_failures,
            );
            profiler.record_counter(concat!($prefix, ".alloc_bytes"), allocation.alloc_bytes);
            profiler.record_counter(
                concat!($prefix, ".alloc_zeroed_bytes"),
                allocation.alloc_zeroed_bytes,
            );
            profiler.record_counter(concat!($prefix, ".dealloc_calls"), allocation.dealloc_calls);
            profiler.record_counter(concat!($prefix, ".dealloc_bytes"), allocation.dealloc_bytes);
            profiler.record_counter(concat!($prefix, ".realloc_calls"), allocation.realloc_calls);
            profiler.record_counter(
                concat!($prefix, ".realloc_failures"),
                allocation.realloc_failures,
            );
            profiler.record_counter(
                concat!($prefix, ".realloc_old_bytes"),
                allocation.realloc_old_bytes,
            );
            profiler.record_counter(
                concat!($prefix, ".realloc_new_bytes"),
                allocation.realloc_new_bytes,
            );
        }};
    }
    match (request, finished) {
        (Request::First, false) => record!("diagnostic.workspace_symbols.request1.before"),
        (Request::First, true) => record!("diagnostic.workspace_symbols.request1.after"),
        (Request::Second, false) => record!("diagnostic.workspace_symbols.request2.before"),
        (Request::Second, true) => record!("diagnostic.workspace_symbols.request2.after"),
    }
}
