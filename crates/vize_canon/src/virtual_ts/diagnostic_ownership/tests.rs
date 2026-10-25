use super::{DiagnosticOwnership, InputOwner, InputRole};
use std::ops::Range;

fn owner() -> InputOwner {
    InputOwner {
        usage: 10,
        scope: 2,
        start: 20,
        end: 40,
        role: InputRole::Value,
    }
}

fn ranges(ledger: DiagnosticOwnership) -> Vec<(Range<usize>, Range<usize>)> {
    ledger
        .into_echoes()
        .into_iter()
        .map(|echo| (echo.generated, echo.authored))
        .collect()
}

#[test]
fn only_a_real_primary_of_the_same_binding_and_role_owns_an_echo() {
    let key = owner();
    let mut ledger = DiagnosticOwnership::default();
    ledger.record(key, 10..15, 22..27, true);
    ledger.record(key, 20..25, 22..27, false);
    assert_eq!(ranges(ledger), vec![(10..15, 22..27)]);
    for other in [
        InputOwner { usage: 11, ..key },
        InputOwner { scope: 3, ..key },
        InputOwner { start: 21, ..key },
        key.with_role(InputRole::Name),
    ] {
        let mut ledger = DiagnosticOwnership::default();
        ledger.record(key, 10..15, 22..27, true);
        ledger.record(other, 20..25, 22..27, false);
        assert_eq!(ranges(ledger), vec![]);
    }
}

#[test]
fn missing_empty_or_overlapping_primaries_do_not_hide_diagnostics() {
    for primary in [None, Some(10..10), Some(10..15), Some(12..14)] {
        let mut ledger = DiagnosticOwnership::default();
        ledger.record(owner(), 10..15, 22..27, true);
        if let Some(primary) = primary {
            ledger.record(owner(), primary, 22..27, false);
        }
        assert_eq!(ranges(ledger), vec![]);
    }
}

#[test]
fn separate_generations_cannot_share_an_owner_at_colliding_offsets() {
    let mut first_file = DiagnosticOwnership::default();
    let mut second_file = DiagnosticOwnership::default();
    first_file.record(owner(), 10..15, 22..27, true);
    second_file.record(owner(), 20..25, 22..27, false);
    assert_eq!(ranges(first_file), vec![]);
    assert_eq!(ranges(second_file), vec![]);
}

#[test]
fn both_proven_navigation_and_inference_copies_keep_exact_owner_ranges() {
    let mut ledger = DiagnosticOwnership::default();
    ledger.record_bare_navigation(owner(), 10..15, 22..27);
    ledger.record(owner(), 20..25, 22..27, true);
    ledger.record(owner(), 30..35, 22..27, false);
    let echoes = ledger.into_echoes();
    assert_eq!(
        echoes
            .iter()
            .map(|echo| echo.generated.clone())
            .collect::<Vec<_>>(),
        [10..15, 20..25]
    );
    let mut missing = DiagnosticOwnership::default();
    missing.record_bare_navigation(owner(), 10..15, 22..27);
    assert!(missing.into_echoes().is_empty());
}

#[test]
fn contextual_geometry_requires_its_primary_but_never_suppresses_diagnostics() {
    let mut absent = DiagnosticOwnership::default();
    absent.record_expect_geometry(owner(), 10..15, 22..27, true);
    assert!(absent.into_echoes().is_empty());
    let mut ledger = DiagnosticOwnership::default();
    ledger.record_expect_geometry(owner(), 10..15, 22..27, true);
    ledger.record_expect_geometry(owner(), 30..35, 22..27, false);
    let echoes = ledger.into_echoes();
    assert_eq!(echoes.len(), 1);
    assert_eq!(echoes[0].generated, 10..15);
    assert_eq!(echoes[0].authored, 22..27);
    assert!(!echoes[0].suppress);
}

#[test]
fn contextual_refusals_survive_deferred_buffers_and_name_role_normalization() {
    for role in [InputRole::Name, InputRole::Value] {
        let mut main = DiagnosticOwnership::default();
        main.refuse_owner(owner());
        let mut buffer = DiagnosticOwnership::default();
        buffer.record(owner().with_role(role), 2..7, 22..27, true);
        buffer.record(owner().with_role(role), 12..17, 22..27, false);
        main.append_rebased(buffer, 100);
        assert!(main.into_echoes().is_empty());
    }
}

#[test]
fn a_deferred_root_buffer_rebases_echoes_and_primaries_with_its_text() {
    let mut whole = DiagnosticOwnership::default();
    whole.record(owner(), 10..15, 22..27, true);
    let mut deferred = DiagnosticOwnership::default();
    deferred.record(owner(), 2..7, 22..27, true);
    deferred.record(owner(), 12..17, 22..27, false);
    whole.append_rebased(deferred, 100);
    assert_eq!(ranges(whole), vec![(10..15, 22..27), (102..107, 22..27)]);
}

#[test]
fn accepted_insertions_use_real_byte_offsets_and_preserve_half_open_edges() {
    for (at, expected) in [
        (0, Some(26..36)),
        (10, Some(26..36)),
        (20, Some(26..36)),
        (25, None),
        (30, Some(20..30)),
        (40, Some(20..30)),
    ] {
        let mut ledger = DiagnosticOwnership::default();
        ledger.record(owner(), 1..5, 22..27, false);
        ledger.record(owner(), 20..30, 22..27, true);
        ledger.note_generated_insertion(at, "🧭\r\n".len());
        assert_eq!(
            ranges(ledger),
            expected
                .map(|range| vec![(range, 22..27)])
                .unwrap_or_default()
        );
    }
}

#[test]
fn overlapping_primary_edits_remove_proof_and_same_offset_insertions_keep_order() {
    let mut invalid = DiagnosticOwnership::default();
    invalid.record(owner(), 1..5, 22..27, false);
    invalid.record(owner(), 20..30, 22..27, true);
    invalid.note_generated_insertion(3, 2);
    assert_eq!(ranges(invalid), vec![]);
    let mut valid = DiagnosticOwnership::default();
    valid.record(owner(), 1..5, 22..27, false);
    valid.record(owner(), 20..30, 22..27, true);
    valid.note_generated_insertion(20, 3);
    valid.note_generated_insertion(20, 2);
    assert_eq!(ranges(valid), vec![(25..35, 22..27)]);
}
