use std::{cell::Cell, convert::Infallible};

use serde_json::json;

use super::execute_cached_capability_measured_with_clock;
use crate::{
    CapabilityCacheIdentity, CapabilitySnapshot, CapabilitySnapshotCache, ContentFingerprint,
    DoctorFinding, MemoryCapabilitySnapshotCache,
};

fn identity() -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        "template-semantics",
        ContentFingerprint::digest("implementation"),
        ContentFingerprint::digest("configuration"),
        [("src/App.vue", ContentFingerprint::digest("source"))],
    )
    .unwrap()
}

fn empty_findings(_identity: &CapabilityCacheIdentity) -> Result<Vec<DoctorFinding>, Infallible> {
    Ok(Vec::new())
}

#[test]
fn cache_miss_records_exact_stage_and_total_latency() {
    let ticks = [100, 110, 140, 150, 210, 220, 250, 280];
    let next_tick = Cell::new(0);
    let clock = || {
        let index = next_tick.get();
        next_tick.set(index + 1);
        ticks[index]
    };
    let mut cache = MemoryCapabilitySnapshotCache::new();

    let measured = execute_cached_capability_measured_with_clock(
        &mut cache,
        identity(),
        empty_findings,
        clock,
    )
    .unwrap();

    assert!(measured.outcome().is_cache_miss());
    assert_eq!(measured.latency().schema_version(), 1);
    assert_eq!(measured.latency().cache_load_time_ns(), 30);
    assert_eq!(measured.latency().analysis_time_ns(), Some(60));
    assert_eq!(measured.latency().cache_store_time_ns(), Some(30));
    assert_eq!(measured.latency().total_time_ns(), 180);
    assert_eq!(next_tick.get(), ticks.len());
}

#[test]
fn cache_hit_omits_analysis_and_store_latency() {
    let requested = identity();
    let snapshot = CapabilitySnapshot::try_new(requested.clone(), []).unwrap();
    let mut cache = MemoryCapabilitySnapshotCache::new();
    cache.store_snapshot(&snapshot).unwrap();
    let ticks = [1_000, 1_010, 1_035, 1_050];
    let next_tick = Cell::new(0);
    let clock = || {
        let index = next_tick.get();
        next_tick.set(index + 1);
        ticks[index]
    };

    let measured = execute_cached_capability_measured_with_clock(
        &mut cache,
        requested,
        |_| -> Result<Vec<DoctorFinding>, Infallible> {
            panic!("analysis must not run on a trusted cache hit")
        },
        clock,
    )
    .unwrap();

    assert!(measured.outcome().is_cache_hit());
    assert_eq!(measured.outcome().snapshot(), &snapshot);
    assert_eq!(measured.latency().cache_load_time_ns(), 25);
    assert_eq!(measured.latency().analysis_time_ns(), None);
    assert_eq!(measured.latency().cache_store_time_ns(), None);
    assert_eq!(measured.latency().total_time_ns(), 50);
    assert_eq!(next_tick.get(), ticks.len());
}

#[test]
fn latency_wire_contract_names_units_and_absent_stages() {
    let requested = identity();
    let snapshot = CapabilitySnapshot::try_new(requested.clone(), []).unwrap();
    let mut cache = MemoryCapabilitySnapshotCache::new();
    cache.store_snapshot(&snapshot).unwrap();
    let ticks = [0, 10, 30, 40];
    let next_tick = Cell::new(0);

    let measured = execute_cached_capability_measured_with_clock(
        &mut cache,
        requested,
        empty_findings,
        || {
            let index = next_tick.get();
            next_tick.set(index + 1);
            ticks[index]
        },
    )
    .unwrap();

    assert_eq!(
        serde_json::to_value(measured.latency()).unwrap(),
        json!({
            "schemaVersion": 1,
            "cacheLoadTimeNs": 20,
            "analysisTimeNs": null,
            "cacheStoreTimeNs": null,
            "totalTimeNs": 40,
        })
    );
}

#[test]
fn measurement_preserves_deterministic_outcome_telemetry() {
    let requested = identity();
    let ticks = [0, 1, 2, 3, 4, 5, 6, 7];
    let next_tick = Cell::new(0);
    let mut cache = MemoryCapabilitySnapshotCache::new();

    let measured = execute_cached_capability_measured_with_clock(
        &mut cache,
        requested.clone(),
        empty_findings,
        || {
            let index = next_tick.get();
            next_tick.set(index + 1);
            ticks[index]
        },
    )
    .unwrap();

    let telemetry = measured.outcome().telemetry();
    assert_eq!(telemetry.capability(), "template-semantics");
    assert_eq!(telemetry.cache_key(), requested.cache_key());
    assert_eq!(telemetry.finding_count(), 0);
    assert_eq!(measured.into_outcome().snapshot().identity(), &requested);
}
