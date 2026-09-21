//! TS-36: every witness is re-checked against the fact base, every forged
//! witness is rejected with its exact [`WitnessError`], and every valid one
//! verifies.
//!
//! The forged fixtures are the committed table in
//! `forged_witnesses_are_rejected_with_their_exact_error`: one per way a
//! link can lie — a group that cannot back a witness, the right fact under
//! the wrong group, the wrong span, a missing key, a maybe cited as a proof,
//! a group the producer never declared, a group the run never computed —
//! each asserted equal to the whole error value.

mod facts;

use facts::{
    CHECKS, Hidden, Lengths, REGISTRY, RepeatedWord, Repeats, Sentence, Words, repeated_words,
};
use vize_davinci::diagnostic::{
    Advisory, Diagnostic, Exemption, Stage, Verdict, WitnessChain, WitnessKey, WitnessLink,
};
use vize_davinci::fact::{Demand, FactError, FactGroup, FactManager};
use vize_davinci::witness::{
    AuditReport, WitnessAudit, WitnessError, WitnessFailure, unverifiable_witnesses, verify,
    verify_as, verify_chain,
};
use vize_s0::{Span, String};

/// Words: the@0..3 cat@4..7 saw@8..11 the@12..15 dog@16..19
/// maybe?@20..26 maybe?@27..33 — the last two are maybes.
const TEXT: &str = "the cat saw the dog maybe? maybe?";

static EXEMPT: Exemption = Exemption::new("witness_verify_fixture", "legacy");

fn link(group: AnalysisGroup, start: u32, end: u32, key: WitnessKey) -> WitnessLink {
    WitnessLink::new(group.id(), Span::new(start, end), key)
}

/// The fixture groups by name, so a forged link reads like the fact it lies
/// about.
#[derive(Clone, Copy)]
enum AnalysisGroup {
    Words,
    Repeats,
    Sentence,
    Lengths,
    Hidden,
}

impl AnalysisGroup {
    fn id(self) -> vize_davinci::pass::AnalysisId {
        match self {
            Self::Words => Words::ID,
            Self::Repeats => Repeats::ID,
            Self::Sentence => Sentence::ID,
            Self::Lengths => Lengths::ID,
            Self::Hidden => Hidden::ID,
        }
    }
}

fn name(text: &str) -> WitnessKey {
    WitnessKey::Name(String::from(text))
}

fn the_valid_chain() -> WitnessChain {
    WitnessChain::new(link(AnalysisGroup::Words, 0, 3, WitnessKey::Index(0)))
        .then(link(AnalysisGroup::Repeats, 12, 15, name("the")))
        .then(link(AnalysisGroup::Sentence, 0, 33, WitnessKey::Artifact))
}

#[test]
fn the_rule_emits_one_proven_error_whose_witness_verifies() {
    let mut manager = FactManager::new(&REGISTRY, TEXT);
    let view = manager.prepare::<RepeatedWord>().expect("registered");
    let found = repeated_words(&view);
    let expected = Diagnostic::proven(
        Stage::Semantic,
        Span::new(12, 15),
        "`the` is repeated",
        the_valid_chain(),
    );
    assert_eq!(found, std::slice::from_ref(&expected));
    assert_eq!(verify(&expected, &view, &CHECKS), Ok(()));
    assert_eq!(
        verify_as::<RepeatedWord, str>(&expected, &manager, &CHECKS),
        Ok(())
    );
}

#[test]
fn forged_witnesses_are_rejected_with_their_exact_error() {
    use AnalysisGroup::{Hidden as HiddenGroup, Lengths as LengthsGroup, Repeats as R, Words as W};
    let undeclared = if cfg!(debug_assertions) {
        FactError::Undeclared {
            consumer: "repeated-word",
            group: Hidden::ID,
        }
    } else {
        FactError::NotComputed { group: Hidden::ID }
    };
    let fixtures: Vec<(&str, WitnessLink, WitnessError)> = vec![
        (
            "a group that cannot back a witness",
            link(LengthsGroup, 0, 3, WitnessKey::Index(0)),
            WitnessError::UnknownGroup {
                link: 0,
                group: Lengths::ID,
            },
        ),
        (
            "the repeat fact cited under the words group",
            link(W, 12, 15, name("the")),
            WitnessError::KeyShape {
                link: 0,
                group: Words::ID,
                key: name("the"),
            },
        ),
        (
            "the first `the` cited at the second one's span",
            link(W, 12, 15, WitnessKey::Index(0)),
            WitnessError::SpanMismatch {
                link: 0,
                group: Words::ID,
                fact: Span::new(0, 3),
                witness: Span::new(12, 15),
            },
        ),
        (
            "a word index past the sentence",
            link(W, 0, 3, WitnessKey::Index(99)),
            WitnessError::MissingKey {
                link: 0,
                group: Words::ID,
                key: WitnessKey::Index(99),
            },
        ),
        (
            "a word that occurs once cited as a repeat",
            link(R, 4, 7, name("cat")),
            WitnessError::MissingKey {
                link: 0,
                group: Repeats::ID,
                key: name("cat"),
            },
        ),
        (
            "a maybe cited as a proof",
            link(W, 20, 26, WitnessKey::Index(5)),
            WitnessError::NotProven {
                link: 0,
                group: Words::ID,
                verdict: Verdict::Unknown,
            },
        ),
        (
            "a group outside the producer's demand",
            link(HiddenGroup, 0, 3, WitnessKey::Index(0)),
            WitnessError::Fact {
                link: 0,
                error: undeclared,
            },
        ),
    ];
    let mut manager = FactManager::new(&REGISTRY, TEXT);
    let view = manager.prepare::<RepeatedWord>().expect("registered");
    for (case, forged, expected) in fixtures {
        let error = Diagnostic::proven(
            Stage::Semantic,
            forged.span,
            "forged",
            WitnessChain::new(forged),
        );
        assert_eq!(verify(&error, &view, &CHECKS), Err(expected), "{case}");
    }
}

#[test]
fn the_first_failing_link_is_reported_by_its_position_in_the_chain() {
    let mut manager = FactManager::new(&REGISTRY, TEXT);
    let view = manager.prepare::<RepeatedWord>().expect("registered");
    let forged = the_valid_chain().then(link(AnalysisGroup::Sentence, 0, 32, WitnessKey::Artifact));
    assert_eq!(
        verify_chain(&forged, &view, &CHECKS),
        Err(WitnessError::SpanMismatch {
            link: 3,
            group: Sentence::ID,
            fact: Span::new(0, 33),
            witness: Span::new(0, 32),
        })
    );
}

#[test]
fn a_view_taken_before_the_run_computed_the_group_refuses_the_witness() {
    let manager = FactManager::new(&REGISTRY, TEXT);
    let view = manager.view::<RepeatedWord>();
    assert_eq!(
        verify_chain(&the_valid_chain(), &view, &CHECKS),
        Err(WitnessError::Fact {
            link: 0,
            error: FactError::NotComputed { group: Words::ID },
        })
    );
}

#[test]
fn only_chains_are_rechecked_and_a_forged_why_is_refused_too() {
    let mut manager = FactManager::new(&REGISTRY, TEXT);
    let view = manager.prepare::<RepeatedWord>().expect("registered");
    let exempt = Diagnostic::legacy_error(&EXEMPT, Stage::Surface, Span::new(0, 3), "legacy");
    let plain = Diagnostic::new(Advisory::Warning, Stage::Semantic, Span::new(0, 3), "plain");
    let forged_why =
        Diagnostic::new(Advisory::Hint, Stage::Semantic, Span::new(0, 3), "why").with_witness(
            WitnessChain::new(link(AnalysisGroup::Words, 0, 3, WitnessKey::Index(1))),
        );
    assert_eq!(verify(&exempt, &view, &CHECKS), Ok(()));
    assert_eq!(verify(&plain, &view, &CHECKS), Ok(()));
    assert_eq!(
        verify(&forged_why, &view, &CHECKS),
        Err(WitnessError::SpanMismatch {
            link: 0,
            group: Words::ID,
            fact: Span::new(4, 7),
            witness: Span::new(0, 3),
        })
    );
}

#[test]
fn the_audit_rechecks_every_diagnostic_in_debug_and_is_free_in_release() {
    let mut manager = FactManager::new(&REGISTRY, TEXT);
    let view = manager.prepare::<RepeatedWord>().expect("registered");
    let mut batch = repeated_words(&view);
    batch.push(Diagnostic::legacy_error(
        &EXEMPT,
        Stage::Surface,
        Span::new(0, 3),
        "legacy",
    ));
    let forged = link(AnalysisGroup::Words, 20, 26, WitnessKey::Index(5));
    batch.push(Diagnostic::proven(
        Stage::Semantic,
        forged.span,
        "forged",
        WitnessChain::new(forged),
    ));
    batch.push(Diagnostic::new(
        Advisory::Info,
        Stage::Semantic,
        Span::new(0, 0),
        "note",
    ));
    let before = unverifiable_witnesses();
    let mut audit = WitnessAudit::new();
    audit.audit(&batch[..2], &view, &CHECKS);
    audit.audit(&batch[2..], &view, &CHECKS);
    let expected = if cfg!(debug_assertions) {
        AuditReport {
            observed: 4,
            verified: 1,
            exempt: 1,
            failures: vec![WitnessFailure {
                diagnostic: 2,
                error: WitnessError::NotProven {
                    link: 0,
                    group: Words::ID,
                    verdict: Verdict::Unknown,
                },
            }],
        }
    } else {
        AuditReport::default()
    };
    assert_eq!(audit.report(), expected);
    assert_eq!(
        unverifiable_witnesses() - before,
        u64::from(cfg!(debug_assertions))
    );
}

#[test]
fn the_registry_names_exactly_the_witness_capable_groups() {
    let expected = Demand::NONE
        .with(Words::ID)
        .with(Repeats::ID)
        .with(Sentence::ID)
        .with(Hidden::ID);
    assert_eq!(CHECKS.groups(), expected);
    let names: Vec<Option<&str>> = [
        Words::ID,
        Repeats::ID,
        Sentence::ID,
        Lengths::ID,
        Hidden::ID,
    ]
    .map(|group| CHECKS.check(group).map(|check| check.name))
    .to_vec();
    assert_eq!(
        names,
        [
            Some("words"),
            Some("repeats"),
            Some("sentence"),
            None,
            Some("hidden")
        ]
    );
}
