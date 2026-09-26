//! Attribution of full provenance; never changes its retention or spelling.
//! Heap capacities are retained storage, not allocation traffic or peak usage.

use vize_l0::profiler::{ProfileGuard, global_profiler};
use vize_l2::provenance::ProvenanceRecord;

/// The caller has already built `after`, so its allocation is excluded here.
pub(super) fn record_guard() -> Option<ProfileGuard> {
    global_profiler().global_span("davinci.lower.provenance.record")
}

/// The artifact owns these bytes at the end of L1→L2 lowering. Later transform
/// pass records are outside this stage; inline CompactStrings own no heap.
pub(super) fn retained(records: &[ProvenanceRecord], capacity: usize) {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return;
    }
    let bytes = core::mem::size_of::<ProvenanceRecord>() as u64;
    profiler.record_counter_enabled("davinci.lower.provenance.records", records.len() as u64);
    profiler.record_counter_enabled(
        "davinci.lower.provenance.vector_capacity_bytes",
        capacity as u64 * bytes,
    );
    profiler.record_counter_enabled(
        "davinci.lower.provenance.record_bytes",
        records.len() as u64 * bytes,
    );
    for (count, capacity, field) in [
        (
            "davinci.lower.provenance.rule_heap_strings",
            "davinci.lower.provenance.rule_heap_capacity_bytes",
            0,
        ),
        (
            "davinci.lower.provenance.before_heap_strings",
            "davinci.lower.provenance.before_heap_capacity_bytes",
            1,
        ),
        (
            "davinci.lower.provenance.after_heap_strings",
            "davinci.lower.provenance.after_heap_capacity_bytes",
            2,
        ),
    ] {
        let mut strings = 0;
        let mut heap_bytes = 0;
        for record in records {
            let text = match field {
                0 => &record.rule,
                1 => &record.before,
                _ => &record.after,
            };
            if text.is_heap_allocated() {
                strings += 1;
                heap_bytes += text.capacity() as u64;
            }
        }
        profiler.record_counter_enabled(count, strings);
        profiler.record_counter_enabled(capacity, heap_bytes);
    }
}
