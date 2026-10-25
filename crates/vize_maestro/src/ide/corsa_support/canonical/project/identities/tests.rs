use super::{FileIdentity, OperandOrder, UriMembership, location_matches_uri_cached};
use crate::server::ServerState;
use std::path::Path;
use tower_lsp::lsp_types::Url;
use vize_l0::{String, cstr};

#[cfg(unix)]
mod physical;

fn file_uri(path: &Path) -> String {
    Url::from_file_path(path).expect("file URI").as_str().into()
}

fn assert_sequence(
    legacy_state: &ServerState,
    indexed_state: &ServerState,
    seed: &[&str],
    sequence: &[&str],
) {
    for order in [OperandOrder::CandidateFirst, OperandOrder::AcceptedFirst] {
        legacy_state.invalidate_canonical_editor_paths();
        indexed_state.invalidate_canonical_editor_paths();
        let mut legacy = seed.to_vec();
        let mut indexed = UriMembership::with_order(seed.iter().copied(), order);
        for raw in sequence {
            let accepted = !legacy.iter().any(|prior| match order {
                OperandOrder::CandidateFirst => {
                    location_matches_uri_cached(legacy_state, raw, prior)
                }
                OperandOrder::AcceptedFirst => {
                    location_matches_uri_cached(legacy_state, prior, raw)
                }
            });
            if accepted {
                legacy.push(raw);
            }
            assert_eq!(
                indexed.insert_if_absent(indexed_state, raw),
                accepted,
                "seed={seed:?}, sequence={sequence:?}, current={raw:?}",
            );
            assert_eq!(
                indexed
                    .accepted
                    .iter()
                    .map(|entry| entry.raw.as_str())
                    .collect::<Vec<_>>(),
                legacy,
                "whole retained payload order",
            );
        }
    }
}

#[test]
fn physical_observations_keep_the_callers_operand_order() {
    let directory = tempfile::tempdir().expect("directory");
    let first = directory.path().join("first.ts");
    let second = directory.path().join("second.ts");
    let candidate = directory.path().join("candidate.ts");
    let first_uri = file_uri(&first);
    let second_uri = file_uri(&second);
    let candidate_uri = file_uri(&candidate);
    for (order, expected) in [
        (
            OperandOrder::CandidateFirst,
            vec![candidate.clone(), first.clone(), second.clone()],
        ),
        (
            OperandOrder::AcceptedFirst,
            vec![first.clone(), candidate.clone(), second.clone()],
        ),
    ] {
        let state = ServerState::new();
        let mut membership =
            UriMembership::with_order([first_uri.as_str(), second_uri.as_str()], order);
        let candidate = FileIdentity::parse(&candidate_uri).expect("file identity");
        let mut observations = Vec::new();
        assert_eq!(
            membership.contains_file(&candidate, 2, 0, |path| {
                observations.push(path.to_path_buf());
                Some(state.canonical_editor_path(path))
            }),
            Some(false)
        );
        assert_eq!(observations, expected);
    }
}

#[test]
fn uncommitted_physical_observation_cannot_produce_an_indexed_decision() {
    let directory = tempfile::tempdir().expect("directory");
    let seed = file_uri(&directory.path().join("seed.ts"));
    let candidate = file_uri(&directory.path().join("candidate.ts"));
    let mut membership = UriMembership::candidate_first([seed.as_str()]);
    let candidate = FileIdentity::parse(&candidate).expect("file identity");
    assert_eq!(membership.contains_file(&candidate, 1, 0, |_| None), None);
    assert!(membership.first_physical.is_empty());
}

#[test]
fn real_legacy_relation_matches_every_uri_pair_and_ordered_triple() {
    let directory = tempfile::tempdir().expect("directory");
    let real = file_uri(&directory.path().join("real.ts"));
    let quoted = file_uri(&directory.path().join("a space \"🧭.ts"));
    let real_path = Url::parse(&real).expect("URI").path().to_owned();
    let corpus: Vec<String> = vec![
        real.clone(),
        cstr!("{real}?query=one"),
        cstr!("{real}#fragment"),
        cstr!("file://localhost{real_path}"),
        cstr!("file://remote.example{real_path}"),
        cstr!("https://first.example{real_path}"),
        cstr!("https://second.example{real_path}?different#fragment"),
        real.replace("real.ts", "%72eal.ts").as_str().into(),
        quoted.clone(),
        quoted.replace("%F0", "%f0").as_str().into(),
        file_uri(&directory.path().join("Case.ts")),
        file_uri(&directory.path().join("case.ts")),
        "custom:opaque".into(),
        "vize-virtual://http://invalid host/source.ts".into(),
        "/unparseable/absolute.ts".into(),
        "unparseable relative.ts".into(),
        "".into(),
    ];
    let legacy = ServerState::new();
    let indexed = ServerState::new();
    for first in &corpus {
        for second in &corpus {
            let pair = [first.as_str(), second.as_str()];
            assert_sequence(&legacy, &indexed, &[], &pair);
            for third in &corpus {
                assert_sequence(
                    &legacy,
                    &indexed,
                    &[],
                    &[first.as_str(), second.as_str(), third.as_str()],
                );
            }
        }
    }
    assert_sequence(
        &legacy,
        &indexed,
        &[real.as_str(), real.as_str(), quoted.as_str()],
        &[quoted.as_str(), real.as_str(), "another invalid URI"],
    );
}

#[test]
fn independent_vectors_keep_their_existing_payload_and_filter_denominators() {
    let state = ServerState::new();
    let primary = "file:///project/Primary.vue.ts";
    let dependency = "file:///project/Dependency.vue.ts";
    let extra = "file:///project/Extra.vue.ts";
    let mut dependencies = UriMembership::candidate_first([primary, dependency]);
    let mut materialized = UriMembership::accepted_first([dependency, dependency]);
    assert!(!dependencies.insert_if_absent(&state, primary));
    assert!(!dependencies.insert_if_absent(&state, dependency));
    assert_eq!(materialized.accepted.len(), 2, "existing payloads survive");
    assert!(materialized.insert_if_absent(&state, extra));
    assert!(!materialized.insert_if_absent(&state, extra));
    // Final project filtering excludes host/dependencies only. Remaining
    // materialized duplicates retain their original order and multiplicity.
    let remaining = [dependency, extra, extra]
        .into_iter()
        .filter(|uri| !dependencies.contains(&state, uri))
        .collect::<Vec<_>>();
    assert_eq!(remaining, vec![extra, extra]);
}

#[test]
fn dependency_primary_keeps_its_separate_candidate_first_comparison() {
    let directory = tempfile::tempdir().expect("directory");
    let first = file_uri(&directory.path().join("first.ts"));
    let second = file_uri(&directory.path().join("second.ts"));
    let corpus = [
        first.as_str(),
        second.as_str(),
        "https://remote.example/first.ts",
        "invalid URI",
        "",
    ];
    let legacy = ServerState::new();
    let indexed = ServerState::new();
    for primary in corpus {
        for owner in corpus {
            for candidate in corpus {
                legacy.invalidate_canonical_editor_paths();
                indexed.invalidate_canonical_editor_paths();
                let expected = !location_matches_uri_cached(&legacy, candidate, primary)
                    && !location_matches_uri_cached(&legacy, owner, candidate);
                let mut membership = UriMembership::accepted_first([owner]);
                assert_eq!(
                    membership.insert_unless_primary(&indexed, candidate, primary),
                    expected
                );
                assert_eq!(membership.work.primary_comparisons, 1);
            }
        }
    }
}

#[test]
fn actual_identity_work_is_linear_for_repeated_file_membership() {
    for count in [128_usize, 256, 512] {
        let directory = tempfile::tempdir().expect("directory");
        let uris = (0..count)
            .map(|index| file_uri(&directory.path().join(cstr!("Source{index}.ts").as_str())))
            .collect::<Vec<_>>();
        let indexed_state = ServerState::new();
        let legacy_state = ServerState::new();
        let mut indexed = UriMembership::candidate_first(uris.iter().map(String::as_str));
        let mut legacy_comparisons = 0_usize;
        for uri in &uris {
            let original = uris.iter().any(|prior| {
                legacy_comparisons += 1;
                location_matches_uri_cached(&legacy_state, uri, prior)
            });
            assert_eq!(indexed.contains(&indexed_state, uri), original);
        }
        assert_eq!(legacy_comparisons, count * (count + 1) / 2);
        assert_eq!(indexed.work.parses, count * 2);
        assert_eq!(indexed.work.probes, count);
        assert_eq!(indexed.work.physical_resolutions, (count - 1) * 2);
        assert_eq!(indexed.work.legacy_comparisons, 0);
        assert_eq!(indexed.work.physical_resets, 1);
        println!(
            "file rows={count}, legacy pairs={legacy_comparisons}, index parses={}, physical resolutions={}",
            indexed.work.parses, indexed.work.physical_resolutions,
        );
    }
}
