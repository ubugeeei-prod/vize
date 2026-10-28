//! Heap membership, slot lifetime and source-model parity laws.

use super::super::pending::{
    DiagnosticsJobCause, InitialDiagnosticsJob, PendingInitialDiagnostics,
};
use std::time::{Duration, Instant};
use tower_lsp::lsp_types::Url;
use vize_l0::FxHashMap;

#[expect(
    clippy::disallowed_macros,
    reason = "test-only canonical URI generation"
)]
fn uri(index: u64) -> Url {
    Url::parse(&format!("file:///scheduler/Component{index}.vue")).unwrap()
}

fn job(version: i32, sequence: u64, now: Instant, open: bool) -> InitialDiagnosticsJob {
    InitialDiagnosticsJob {
        cause: if open {
            DiagnosticsJobCause::Open
        } else {
            DiagnosticsJobCause::Edit
        },
        version,
        sequence,
        not_before: now,
        sync_pending: open,
        sync_running: false,
        preparation_pending: open,
        sync: None,
        continuation: None,
    }
}

fn identity(
    value: &Option<(Url, InitialDiagnosticsJob)>,
) -> Option<(Url, i32, u64, bool, bool, bool)> {
    value.as_ref().map(|(uri, job)| {
        (
            uri.clone(),
            job.version,
            job.sequence,
            job.sync_pending,
            job.sync_running,
            job.preparation_pending,
        )
    })
}

fn assert_state(
    pending: &PendingInitialDiagnostics,
    model: &FxHashMap<Url, InitialDiagnosticsJob>,
) {
    pending.jobs.verify();
    assert_eq!(pending.jobs.len(), model.len());
    for (uri, expected) in model {
        let actual = pending.jobs.get(uri).unwrap();
        assert_eq!(
            (
                actual.version,
                actual.sequence,
                actual.not_before,
                actual.sync_pending,
                actual.sync_running,
                actual.preparation_pending
            ),
            (
                expected.version,
                expected.sequence,
                expected.not_before,
                expected.sync_pending,
                expected.sync_running,
                expected.preparation_pending
            )
        );
    }
}

#[test]
fn indexed_heaps_match_original_scans_with_explicit_deadline_tie_refinement() {
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    let mut model = FxHashMap::<Url, InitialDiagnosticsJob>::default();
    let mut sequence = 0;
    let mut random = 0x1d6c_7403_815e_a5d7_u64;
    let mut callbacks = Vec::<(Url, u64)>::new();
    for step in 0..50000_u64 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let uri = uri((random >> 20) % 129);
        let operation = (random >> 50) % 11;
        match operation {
            0 | 1 => {
                let version = ((random >> 32) % 31) as i32;
                let deadline = now + Duration::from_nanos(step + 1);
                let open = operation == 0;
                pending.insert_with_continuation(
                    uri.clone(),
                    version,
                    deadline,
                    if open {
                        DiagnosticsJobCause::Open
                    } else {
                        DiagnosticsJobCause::Edit
                    },
                    None,
                    None,
                );
                if !model.get(&uri).is_some_and(|job| job.version > version) {
                    model.insert(uri, job(version, sequence, deadline, open));
                    sequence += 1;
                }
            }
            2 => {
                let selected = model
                    .iter()
                    .filter(|(_, job)| job.sync_pending)
                    .min_by_key(|(_, job)| job.sequence)
                    .map(|(uri, _)| uri.clone());
                let expected = selected.map(|uri| {
                    let job = model.get_mut(&uri).unwrap();
                    job.sync_pending = false;
                    job.sync_running = true;
                    (uri, job.clone())
                });
                let actual = pending.take_sync();
                assert_eq!(identity(&actual), identity(&expected));
                if let Some((uri, job)) = actual {
                    callbacks.push((uri, job.sequence));
                }
            }
            3 => {
                if let Some((uri, sequence)) = callbacks.pop() {
                    pending.complete_sync(&uri, sequence);
                    if let Some(job) = model.get_mut(&uri).filter(|job| job.sequence == sequence) {
                        job.sync_running = false;
                    }
                }
            }
            4 => {
                let selected = model
                    .iter()
                    .filter(|(_, job)| {
                        !job.sync_pending && !job.sync_running && job.preparation_pending
                    })
                    .min_by_key(|(_, job)| job.sequence)
                    .map(|(uri, _)| uri.clone());
                let expected = selected.map(|uri| {
                    let job = model.get_mut(&uri).unwrap();
                    job.preparation_pending = false;
                    (uri, job.clone())
                });
                assert_eq!(identity(&pending.take_preparation()), identity(&expected));
            }
            5 => {
                let expected = model
                    .values()
                    .filter(|job| !job.sync_pending && !job.sync_running)
                    .map(|job| job.not_before)
                    .min();
                assert_eq!(pending.next_not_before(), expected);
            }
            6 => {
                let when = now + Duration::from_nanos(step.saturating_sub(20));
                // This is the original scan key. Equal deadlines had no
                // ordering contract because FxHashMap iteration broke ties.
                let selected = model
                    .iter()
                    .filter(|(_, job)| {
                        !job.sync_pending && !job.sync_running && job.not_before <= when
                    })
                    .min_by_key(|(_, job)| job.not_before)
                    .map(|(uri, job)| (uri.clone(), job.clone()));
                let actual = pending.take_ready(when);
                match (&actual, &selected) {
                    (None, None) => {}
                    (Some((uri, actual_job)), Some((_, minimum))) => {
                        let expected = &model[uri];
                        assert!(!expected.sync_pending && !expected.sync_running);
                        assert_eq!(expected.not_before, minimum.not_before);
                        assert_eq!(actual_job.not_before, minimum.not_before);
                        assert_eq!(
                            identity(&actual),
                            identity(&Some((uri.clone(), expected.clone())))
                        );
                        let tied = model
                            .values()
                            .filter(|job| {
                                !job.sync_pending
                                    && !job.sync_running
                                    && job.not_before == minimum.not_before
                            })
                            .count();
                        if tied == 1 {
                            assert_eq!(identity(&actual), identity(&selected));
                        }
                        model.remove(uri);
                    }
                    _ => panic!("original scan and indexed queue disagree about ready work"),
                }
            }
            7 => {
                let version = ((random >> 32) % 31) as i32;
                pending.complete(&uri, version);
                if model.get(&uri).is_some_and(|job| job.version <= version) {
                    model.remove(&uri);
                }
            }
            8 => {
                pending.jobs.remove(&uri);
                model.remove(&uri);
            }
            9 => {
                pending.jobs.skip_sync(&uri);
                if let Some(job) = model.get_mut(&uri) {
                    job.sync_pending = false;
                }
            }
            _ => {
                if step % 257 == 0 {
                    pending.jobs.clear();
                    model.clear();
                }
            }
        }
        assert_eq!(pending.next_sequence, sequence);
        assert_state(&pending, &model);
    }
}

#[test]
fn reusable_slots_do_not_retain_replaced_closed_or_completed_job_history() {
    let mut pending = PendingInitialDiagnostics::default();
    let now = Instant::now();
    let uri = uri(0);
    for version in 1..=10000 {
        pending.insert(uri.clone(), version, now);
        pending.jobs.verify();
        assert_eq!(pending.jobs.len(), 1);
    }
    let old = pending.take_sync().unwrap().1;
    pending.jobs.remove(&uri);
    pending.insert(uri.clone(), 1, now);
    pending.complete_sync(&uri, old.sequence);
    assert!(pending.take_ready(now).is_none());
    let new = pending.take_sync().unwrap().1;
    pending.complete_sync(&uri, new.sequence);
    assert_eq!(pending.take_ready(now).unwrap().1.version, 1);
    for index in 1..=10000 {
        let next = self::uri(index);
        pending.insert(next.clone(), 1, now);
        pending.jobs.remove(&next);
        pending.jobs.verify();
        assert!(pending.jobs.is_empty());
    }
    // One high-water live URI needs one slot, with no stale heap history.
    pending.jobs.assert_slots(1);
}

#[test]
fn equal_deadline_native_work_is_ordered_by_acceptance_without_overtaking_sync() {
    let mut pending = PendingInitialDiagnostics::default();
    let now = Instant::now();
    for index in 0..129 {
        pending.insert(uri(index), 1, now);
    }
    assert!(pending.next_not_before().is_none());
    while let Some((uri, job)) = pending.take_sync() {
        pending.complete_sync(&uri, job.sequence);
    }
    for index in 0..129 {
        assert_eq!(pending.take_ready(now).unwrap().0, uri(index));
        pending.jobs.verify();
    }
    assert!(pending.jobs.is_empty());
}

#[test]
fn all_three_indexes_clear_and_rebuild_when_closed_slot_ids_are_reused() {
    let mut pending = PendingInitialDiagnostics::default();
    let now = Instant::now();
    for round in 0..8 {
        for index in 0..2500 {
            pending.insert(uri(index), round, now);
        }
        while let Some((uri, job)) = pending.take_sync() {
            pending.complete_sync(&uri, job.sequence);
        }
        for index in (0..2500).step_by(2) {
            pending.jobs.remove(&uri(index));
        }
        pending.jobs.verify();
        for index in (0..2500).step_by(2) {
            pending.insert(uri(index), round + 1, now);
        }
        pending.jobs.verify();
        pending.jobs.assert_slots(2500);
        pending.jobs.clear();
        pending.jobs.verify();
        pending.jobs.assert_slots(0);
    }
}
