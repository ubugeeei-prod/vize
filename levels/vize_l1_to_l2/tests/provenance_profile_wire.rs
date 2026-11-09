//! The native L1→L2 provenance metrics keep one stable profile wire family.
#![cfg(feature = "level-benchmark-profile")]

use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};
use vize_l0::Allocator;
use vize_l0::profiler::global_profiler;
use vize_l1::markup::parse;
use vize_l1_to_l2::{emit_dom_source, lower};
use vize_l2::provenance::ProvenanceRecord;

const PREFIX: &str = "l1_to_l2.provenance.";
const OLD_KEYS: [&str; 10] = [
    "levels.lower.provenance.record",
    "levels.lower.provenance.records",
    "levels.lower.provenance.vector_capacity_bytes",
    "levels.lower.provenance.record_bytes",
    "levels.lower.provenance.rule_heap_strings",
    "levels.lower.provenance.rule_heap_capacity_bytes",
    "levels.lower.provenance.before_heap_strings",
    "levels.lower.provenance.before_heap_capacity_bytes",
    "levels.lower.provenance.after_heap_strings",
    "levels.lower.provenance.after_heap_capacity_bytes",
];

#[test]
fn lowering_exports_the_level_named_provenance_family_only_when_profiled() {
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, "<p v-if=\"ok\" v-for=\"i in is\">t</p>");
    let plain = lower(&allocator, &tree, &errors);
    assert!(!plain.provenance.is_empty());
    assert!(profiler.get("l1_to_l2.provenance.record").is_none());
    assert!(profiler.counter_summary().entries.is_empty());

    profiler.enable();
    let observed = lower(&allocator, &tree, &errors);
    profiler.disable();
    assert_eq!(observed.provenance, plain.provenance);
    assert_eq!(observed.op_count, plain.op_count);

    let report = export_report(
        profiler,
        &ProfileExportOptions {
            command: "dump",
            allocation: None,
            budget: ProfileExportBudget::default(),
        },
    );
    let spans: Vec<_> = report
        .spans
        .iter()
        .filter(|span| span.key.starts_with(PREFIX))
        .map(|span| (span.key, span.count))
        .collect();
    assert_eq!(
        spans,
        [(
            "l1_to_l2.provenance.record",
            observed.provenance.len() as u64
        )]
    );

    let mut keys: Vec<_> = report
        .counters
        .iter()
        .filter(|counter| counter.key.starts_with(PREFIX))
        .map(|counter| counter.key)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "l1_to_l2.provenance.after_heap_capacity_bytes",
            "l1_to_l2.provenance.after_heap_strings",
            "l1_to_l2.provenance.before_heap_capacity_bytes",
            "l1_to_l2.provenance.before_heap_strings",
            "l1_to_l2.provenance.record_bytes",
            "l1_to_l2.provenance.records",
            "l1_to_l2.provenance.rule_heap_capacity_bytes",
            "l1_to_l2.provenance.rule_heap_strings",
            "l1_to_l2.provenance.vector_capacity_bytes",
        ]
    );
    assert_eq!(
        report
            .counters
            .iter()
            .find(|counter| counter.key == "l1_to_l2.provenance.records")
            .map(|counter| counter.total),
        Some(observed.provenance.len() as u64)
    );
    assert_eq!(
        report
            .counters
            .iter()
            .find(|counter| counter.key == "l1_to_l2.provenance.record_bytes")
            .map(|counter| counter.total),
        Some(observed.provenance.len() as u64 * size_of::<ProvenanceRecord>() as u64)
    );
    let old_keys: Vec<_> = report
        .spans
        .iter()
        .map(|span| span.key)
        .chain(report.counters.iter().map(|counter| counter.key))
        .filter(|key| OLD_KEYS.iter().any(|old| old == key))
        .collect();
    assert_eq!(old_keys, Vec::<&str>::new());

    // The DOM emitter lowers into a sink that keeps nothing, so it builds no
    // record for the profile to count.
    profiler.clear();
    profiler.enable();
    let emitted = emit_dom_source(&allocator, "<p v-for=\"i in is\">{{ i }}</p>");
    profiler.disable();
    assert_eq!(emitted.map(|emit| emit.code.is_empty()), Ok(false));
    let report = export_report(
        profiler,
        &ProfileExportOptions {
            command: "dump",
            allocation: None,
            budget: ProfileExportBudget::default(),
        },
    );
    let dom_keys: Vec<_> = report
        .spans
        .iter()
        .map(|span| span.key)
        .chain(report.counters.iter().map(|counter| counter.key))
        .filter(|key| key.starts_with(PREFIX))
        .collect();
    assert_eq!(dom_keys, Vec::<&str>::new());
}
