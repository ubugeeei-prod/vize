//! The independent L3 reference over every committed page: TS-28 ladder
//! traces, stateful observations and their fail-closed witnesses, and the
//! C-23 graph checker. The matrix observations are checked beside their
//! producers in `matrix.rs`, `models.rs` and `slots.rs`.

use level_test_support::l3_reference::behavior::{check, run};
use level_test_support::l3_reference::checker_corpus::check_dumps;
use level_test_support::l3_reference::matrix::check_ladder;
use level_test_support::l3_reference::{fixtures, witness};

#[test]
fn ladder_pages_execute_to_their_committed_traces() {
    assert_eq!(check_ladder(&fixtures()).unwrap(), 8);
}

#[test]
fn expression_subset_matches_its_pinned_table() {
    let (supported, unsupported) = witness::expressions(&fixtures()).unwrap();
    assert!(supported >= 50 && unsupported >= 30);
}

#[test]
fn single_button_observations_and_witnesses() {
    witness::behavior(&fixtures()).unwrap();
    check(&fixtures().join("rust-lowered-static-dynamic"), run).unwrap();
}

#[test]
fn control_and_slot_observations_and_witnesses() {
    witness::control::check(&fixtures()).unwrap();
    check(&fixtures().join("rust-lowered-control-slots"), run).unwrap();
}

#[test]
fn loop_observations_reconciliation_and_witnesses() {
    witness::loops::check(&fixtures()).unwrap();
}

#[test]
fn model_and_supplied_slot_witnesses() {
    witness::models::check(&fixtures()).unwrap();
}

/// C-23: every committed L3 graph, Rust-lowered or hand-written, passes the
/// independent checker, and its damage witnesses report the expected codes.
#[test]
fn committed_graphs_pass_the_independent_checker() {
    assert!(check_dumps(&fixtures()).unwrap() >= 50);
}
