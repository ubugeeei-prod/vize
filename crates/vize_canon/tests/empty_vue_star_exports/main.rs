//! Additive configured-native laws; the historical 32 fixtures are unchanged.
#![expect(
    clippy::unwrap_used,
    reason = "complete fixture laws fail by panicking"
)]
#![expect(
    clippy::expect_used,
    reason = "complete fixture laws fail by panicking"
)]
#![expect(
    clippy::disallowed_types,
    reason = "whole fixture packets use std strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "whole fixture packets use JSON and debug output"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "whole fixture packets use owned strings"
)]

mod contracts;
mod execution;
mod membership;
mod projection;
mod qualification;
mod support;

#[test]
fn authentic_six_counterfactual_packets_and_provider_bytes_remain_complete() {
    assert_eq!(
        support::hash(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    membership::controls();
    contracts::authentic_custody();
}

#[test]
fn configured_native_preserves_public_types_and_real_consumer_errors() {
    let Some(runtime) = support::runtime() else {
        return;
    };
    contracts::production(&runtime, "exact");
}

#[test]
fn configured_native_refuses_erasure_at_unproved_source_boundaries() {
    let Some(runtime) = support::runtime() else {
        return;
    };
    contracts::production(&runtime, "projection-refusal-observation");
}
