use core::cell::Cell;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use serde::Serialize;

pub(super) const CAPACITY: usize = 8192;
const CLOSED: u64 = 1 << 63;
static WRITERS: AtomicU64 = AtomicU64::new(CLOSED);
static CURSOR: AtomicU64 = AtomicU64::new(0);
static OVERFLOW: AtomicBool = AtomicBool::new(false);

struct Slot {
    ordinal: AtomicU64,
    tid: AtomicU64,
    size: AtomicU64,
    operation: AtomicU64,
}

impl Slot {
    const fn new() -> Self {
        Self {
            ordinal: AtomicU64::new(0),
            tid: AtomicU64::new(0),
            size: AtomicU64::new(0),
            operation: AtomicU64::new(0),
        }
    }
}

// Fresh process BSS supplies the ledger; no array clear precedes a window.
static SLOTS: [Slot; CAPACITY] = [const { Slot::new() }; CAPACITY];

std::thread_local! {
    // Constant, destructor-free native TLS; identity lookup never allocates.
    static TID: Cell<u64> = const { Cell::new(0) };
}

pub(super) fn tid() -> u64 {
    TID.try_with(|cache| {
        let cached = cache.get();
        if cached != 0 {
            return cached;
        }
        #[cfg(target_os = "linux")]
        // SAFETY: gettid has no pointer arguments and performs no allocation.
        let value = unsafe { libc::syscall(libc::SYS_gettid) };
        #[cfg(not(target_os = "linux"))]
        let value = 0;
        let value = if value > 0 { value as u64 } else { 0 };
        cache.set(value);
        value
    })
    .unwrap_or(0)
}

pub(super) fn record(ordinal: u64, size: usize, operation: u8) {
    // Atomic registration/close gate: a callback cannot pass the open check
    // then reserve an event after snapshot has declared all writers complete.
    let mut writers = WRITERS.load(Ordering::Acquire);
    loop {
        if writers & CLOSED != 0 {
            return;
        }
        match WRITERS.compare_exchange_weak(
            writers,
            writers + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => break,
            Err(current) => writers = current,
        }
    }
    let index = CURSOR.fetch_add(1, Ordering::Relaxed) as usize;
    let Some(slot) = SLOTS.get(index) else {
        OVERFLOW.store(true, Ordering::Relaxed);
        WRITERS.fetch_sub(1, Ordering::Release);
        return;
    };
    slot.tid.store(tid(), Ordering::Relaxed);
    slot.size.store(size as u64, Ordering::Relaxed);
    slot.operation
        .store(u64::from(operation), Ordering::Relaxed);
    // Release publishes the complete allocation-free event.
    slot.ordinal.store(ordinal, Ordering::Release);
    WRITERS.fetch_sub(1, Ordering::Release);
}

pub(super) fn begin() {
    WRITERS.store(0, Ordering::Release);
}

pub(super) fn active() -> bool {
    WRITERS.load(Ordering::Acquire) & CLOSED == 0
}

#[derive(Serialize)]
pub(super) struct Event {
    pub ordinal: u64,
    pub tid: u64,
    pub size: u64,
    pub operation: u64,
}

pub(super) fn snapshot() -> (std::vec::Vec<Event>, bool, u64, u64) {
    let writers = WRITERS.fetch_or(CLOSED, Ordering::AcqRel);
    // Any accepted recorder still publishing makes the receipt fatal. No wait
    // or allocation in the callback, and no accepted late reservation.
    let in_flight = writers & !CLOSED;
    let count = CURSOR.load(Ordering::Acquire);
    let mut unpublished = 0;
    let events = SLOTS
        .iter()
        .take((count as usize).min(CAPACITY))
        .filter_map(|slot| {
            let ordinal = slot.ordinal.load(Ordering::Acquire);
            if ordinal == 0 {
                unpublished += 1;
                return None;
            }
            Some(Event {
                ordinal,
                tid: slot.tid.load(Ordering::Relaxed),
                size: slot.size.load(Ordering::Relaxed),
                operation: slot.operation.load(Ordering::Relaxed),
            })
        })
        .collect();
    (
        events,
        OVERFLOW.load(Ordering::Relaxed),
        unpublished,
        in_flight,
    )
}
