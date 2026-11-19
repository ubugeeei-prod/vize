#![expect(
    clippy::disallowed_types,
    reason = "diagnostic JSON and environment transport occurs outside measured windows"
)]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use serde::Serialize;
use std::sync::OnceLock;

mod ledger;

static PATH: OnceLock<std::string::String> = OnceLock::new();
static CONTEXT: AtomicU64 = AtomicU64::new(0);
static WINDOW_COUNT: AtomicU64 = AtomicU64::new(0);
static WINDOW_OVERFLOW: AtomicBool = AtomicBool::new(false);

struct WindowSlot {
    context: AtomicU64,
    start: AtomicU64,
    end: AtomicU64,
    tid: AtomicU64,
    ready: AtomicBool,
}

impl WindowSlot {
    const fn new() -> Self {
        Self {
            context: AtomicU64::new(0),
            start: AtomicU64::new(0),
            end: AtomicU64::new(0),
            tid: AtomicU64::new(0),
            ready: AtomicBool::new(false),
        }
    }
}

static WINDOWS: [WindowSlot; 32] = [const { WindowSlot::new() }; 32];

/// Prepare observation before warmup; no ledger clear or heap work in windows.
pub fn begin_session() {
    let Ok(path) = std::env::var("VIZE_ALLOCATION_TRACE_PATH") else {
        return;
    };
    assert!(PATH.set(path).is_ok(), "one diagnostic session per process");
    assert_ne!(
        ledger::tid(),
        0,
        "positive actual Linux measuring thread identity"
    );
    ledger::begin();
}

pub fn set_context(context: u64) {
    CONTEXT.store(context, Ordering::Relaxed);
}

pub(in crate::alloc) fn record(ordinal: u64, size: usize, operation: u8) {
    ledger::record(ordinal, size, operation);
}

/// Endpoints are the existing global counter snapshots, not a new window.
pub(in crate::alloc) fn window(start: u64, end: u64) {
    if !ledger::active() {
        return;
    }
    let index = WINDOW_COUNT.fetch_add(1, Ordering::Relaxed) as usize;
    let Some(slot) = WINDOWS.get(index) else {
        WINDOW_OVERFLOW.store(true, Ordering::Relaxed);
        return;
    };
    slot.context
        .store(CONTEXT.load(Ordering::Relaxed), Ordering::Relaxed);
    slot.start.store(start, Ordering::Relaxed);
    slot.end.store(end, Ordering::Relaxed);
    slot.tid.store(ledger::tid(), Ordering::Relaxed);
    slot.ready.store(true, Ordering::Release);
}

#[derive(Serialize)]
struct Window {
    context: u64,
    calls_start: u64,
    calls_end: u64,
    measuring_tid: u64,
}

#[derive(Serialize)]
struct Receipt {
    schema_version: u32,
    status: &'static str,
    process_pid: u32,
    ledger_capacity: usize,
    overflow: bool,
    unpublished_events: u64,
    in_flight_events: u64,
    errors: std::vec::Vec<std::string::String>,
    windows: std::vec::Vec<Window>,
    events: std::vec::Vec<ledger::Event>,
}

/// Serialize after all original rows, before their original cap assertion.
pub fn finish_session() {
    let Some(path) = PATH.get() else {
        return;
    };
    let (events, event_overflow, unpublished_events, in_flight_events) = ledger::snapshot();
    let window_count = WINDOW_COUNT.load(Ordering::Acquire) as usize;
    let overflow = event_overflow || WINDOW_OVERFLOW.load(Ordering::Relaxed);
    let mut errors = std::vec::Vec::new();
    let mut windows = std::vec::Vec::new();
    for slot in WINDOWS.iter().take(window_count.min(WINDOWS.len())) {
        if !slot.ready.load(Ordering::Acquire) {
            errors.push("unpublished window".into());
            continue;
        }
        let window = Window {
            context: slot.context.load(Ordering::Relaxed),
            calls_start: slot.start.load(Ordering::Relaxed),
            calls_end: slot.end.load(Ordering::Relaxed),
            measuring_tid: slot.tid.load(Ordering::Relaxed),
        };
        if window.measuring_tid == 0 || window.calls_end < window.calls_start {
            errors.push("invalid measuring thread or counter endpoints".into());
            continue;
        }
        let mut ordinals: std::vec::Vec<_> = events
            .iter()
            .filter(|event| event.ordinal > window.calls_start && event.ordinal <= window.calls_end)
            .map(|event| event.ordinal)
            .collect();
        ordinals.sort_unstable();
        let expected: std::vec::Vec<_> = (window.calls_start + 1..=window.calls_end).collect();
        if ordinals != expected {
            errors.push("missing or duplicate allocation ordinal inside exact window".into());
        }
        windows.push(window);
    }
    if overflow || unpublished_events != 0 || in_flight_events != 0 {
        errors.push("fixed ledger overflow, unpublished or in-flight event".into());
    }
    if events
        .iter()
        .any(|event| event.tid == 0 || !(1..=3).contains(&event.operation))
    {
        errors.push("invalid allocation identity or operation".into());
    }
    let valid = errors.is_empty();
    let receipt = Receipt {
        schema_version: 1,
        status: if valid {
            "diagnostic-observed"
        } else {
            "diagnostic-invalid"
        },
        process_pid: std::process::id(),
        ledger_capacity: ledger::CAPACITY,
        overflow,
        unpublished_events,
        in_flight_events,
        errors,
        windows,
        events,
    };
    let bytes = serde_json::to_vec(&receipt).expect("diagnostic serialization");
    std::fs::write(path, bytes).expect("preserve diagnostic before cap assertion");
    assert!(
        valid,
        "allocation diagnostic must be complete; no truncation credit"
    );
}
