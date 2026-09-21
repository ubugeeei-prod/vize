//! P4-6a's precision tiers and severities, pinned as exact tables: every
//! tier × severity admission, every clamp, every stable spelling and its
//! parse, and the verdict axis the witness verifier will read.
//!
//! The heuristic-error canary itself is a `compile_fail,E0080` doctest on
//! `vize_davinci::diagnostic::tier`; the runtime panic pinned here is the
//! same assertion reached outside a const context.

use std::panic::catch_unwind;

use vize_davinci::diagnostic::{Advisory, Domain, RuleContract, Severity, Tier, Verdict};

const DOMAIN: Domain = Domain::new("elements with a literal tag name in an SFC template");

#[test]
fn a_heuristic_tier_admits_every_severity_but_error() {
    let table: Vec<(Tier, Vec<bool>)> = Tier::ALL
        .into_iter()
        .map(|tier| {
            let row = Severity::ALL.map(|severity| tier.admits(severity));
            (tier, row.to_vec())
        })
        .collect();
    assert_eq!(
        table,
        [
            (Tier::Exact, vec![true, true, true, true]),
            (Tier::Sound, vec![true, true, true, true]),
            (Tier::Complete, vec![true, true, true, true]),
            (Tier::Heuristic, vec![false, true, true, true]),
        ]
    );
}

#[test]
fn each_tier_states_exactly_its_guarantees() {
    let table: Vec<(&str, bool, bool)> = Tier::ALL
        .into_iter()
        .map(|tier| {
            (
                tier.as_str(),
                tier.no_false_positives(),
                tier.no_false_negatives(),
            )
        })
        .collect();
    assert_eq!(
        table,
        [
            ("exact", true, true),
            ("sound", false, true),
            ("complete", true, false),
            ("heuristic", false, false),
        ]
    );
}

#[test]
fn spellings_round_trip_and_nothing_else_parses() {
    for tier in Tier::ALL {
        assert_eq!(Tier::from_str(tier.as_str()), Some(tier));
    }
    for severity in Severity::ALL {
        assert_eq!(Severity::from_str(severity.as_str()), Some(severity));
    }
    for verdict in [Verdict::Proven, Verdict::Refuted, Verdict::Unknown] {
        assert_eq!(Verdict::from_str(verdict.as_str()), Some(verdict));
    }
    for other in ["", "Exact", "warn", "maybe", "proven "] {
        assert_eq!(
            (
                Tier::from_str(other),
                Severity::from_str(other),
                Verdict::from_str(other)
            ),
            (None, None, None),
            "{other:?}"
        );
    }
}

#[test]
fn severities_order_from_most_to_least_claimed() {
    let mut shuffled = [
        Severity::Hint,
        Severity::Error,
        Severity::Info,
        Severity::Warning,
    ];
    shuffled.sort();
    assert_eq!(shuffled, Severity::ALL);
}

#[test]
fn an_advisory_is_every_severity_but_error() {
    let table = Severity::ALL.map(Advisory::from_severity);
    assert_eq!(
        table,
        [
            None,
            Some(Advisory::Warning),
            Some(Advisory::Info),
            Some(Advisory::Hint)
        ]
    );
    let back = Advisory::ALL.map(Severity::from);
    assert_eq!(back, [Severity::Warning, Severity::Info, Severity::Hint]);
}

#[test]
fn only_proven_is_proof() {
    let table = [Verdict::Proven, Verdict::Refuted, Verdict::Unknown].map(Verdict::is_proven);
    assert_eq!(table, [true, false, false]);
}

#[test]
fn a_contract_reads_back_what_it_declares() {
    static EXACT: RuleContract = RuleContract::new(Tier::Exact, DOMAIN, Severity::Error);
    assert_eq!(
        (EXACT.tier(), EXACT.domain(), EXACT.severity()),
        (Tier::Exact, DOMAIN, Severity::Error)
    );
    assert_eq!(
        EXACT.domain().as_str(),
        "elements with a literal tag name in an SFC template"
    );
}

#[test]
fn configuration_cannot_promote_a_heuristic_finding_into_a_proof() {
    static HEURISTIC: RuleContract = RuleContract::new(Tier::Heuristic, DOMAIN, Severity::Warning);
    static SOUND: RuleContract = RuleContract::new(Tier::Sound, DOMAIN, Severity::Warning);
    assert_eq!(
        Severity::ALL.map(|configured| HEURISTIC.clamp(configured)),
        [
            Severity::Warning,
            Severity::Warning,
            Severity::Info,
            Severity::Hint
        ]
    );
    assert_eq!(
        Severity::ALL.map(|configured| SOUND.clamp(configured)),
        Severity::ALL
    );
}

/// The const assertions panic with literal messages, whose payload is a
/// `&'static str`.
fn panic_message(run: impl FnOnce() + std::panic::UnwindSafe) -> &'static str {
    let payload = catch_unwind(run).expect_err("must panic");
    *payload
        .downcast::<&'static str>()
        .expect("a literal panic message")
}

#[test]
fn the_const_assertions_hold_outside_const_contexts_too() {
    let heuristic_error = panic_message(|| {
        let _ = RuleContract::new(Tier::Heuristic, DOMAIN, Severity::Error);
    });
    assert_eq!(
        heuristic_error,
        "a heuristic rule cannot declare error severity: an error must be a proof"
    );
    let empty_domain = panic_message(|| {
        let _ = Domain::new("");
    });
    assert_eq!(
        empty_domain,
        "a rule's declared domain must state what the tier's claim ranges over"
    );
}
