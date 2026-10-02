//! The native L1→L2 provenance metrics keep one stable profile wire family.
#![cfg(feature = "davinci-benchmark-profile")]

use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};
use vize_l0::Allocator;
use vize_l0::profiler::global_profiler;
use vize_l1::parse;
use vize_l1_to_l2::lower;
use vize_l2::provenance::ProvenanceRecord;

const PREFIX: &str = "l1_to_l2.provenance.";
const OLD_KEYS: [&str; 10] = [
    "davinci.lower.provenance.record",
    "davinci.lower.provenance.records",
    "davinci.lower.provenance.vector_capacity_bytes",
    "davinci.lower.provenance.record_bytes",
    "davinci.lower.provenance.rule_heap_strings",
    "davinci.lower.provenance.rule_heap_capacity_bytes",
    "davinci.lower.provenance.before_heap_strings",
    "davinci.lower.provenance.before_heap_capacity_bytes",
    "davinci.lower.provenance.after_heap_strings",
    "davinci.lower.provenance.after_heap_capacity_bytes",
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
}
