//! The between-pass wiring of the S2 verifier (P2-6): rigor per
//! `PassKind` through the P2-3 observer, exact panic reports naming the
//! offending pass, the arena-stamp liveness reuse, and the cfg shape.
//!
//! The panic-payload assertions compare the **whole** report string
//! (TS-13, no partial matching), which requires `catch_unwind` and a
//! downcast to the panic payload's concrete type — `std::string::String`
//! is what `panic!` formats into, so the house string-type lint is
//! file-allowed here for exactly that downcast, the
//! `davinci_profile_export.rs` precedent.
#![allow(clippy::disallowed_types)]

#[cfg(debug_assertions)]
use std::panic::{AssertUnwindSafe, catch_unwind};

use vize_davinci::folio::Folio;
use vize_davinci::id::NodeId;
use vize_davinci::pass::{
    Fusability, Pair, PassDesc, PassEvent, PassKind, Pipeline, Preserved, run_pipeline,
};
use vize_davinci::side_table::SideTable;
use vize_s0::Allocator;
use vize_s2::folio::DisegnoFolio;
#[cfg(debug_assertions)]
use vize_s2::verify::Rigor;
use vize_s2::verify::VerifyObserver;

const NORMALIZE: PassDesc = PassDesc::new(
    "normalize",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const DIAGNOSE: PassDesc = PassDesc::new(
    "diagnose",
    PassKind::MandatoryDiagnostic,
    Fusability::Barrier,
    Preserved::ALL,
);
const LOWER: PassDesc = PassDesc::new(
    "lower",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::NONE,
);

const WITHOUT_LOWERING: &[PassDesc] = &[NORMALIZE, DIAGNOSE];
const WITH_LOWERING: &[PassDesc] = &[NORMALIZE, LOWER, NORMALIZE];
const LOWER_ONLY: &[PassDesc] = &[LOWER];

/// The event for `passes[pass_index]` in a single-stage pipeline.
fn event_for(pipeline: &Pipeline, pass_index: usize) -> PassEvent<'_> {
    let group_index = pipeline
        .group_of_pass(pass_index)
        .expect("the pass exists in the pipeline");
    PassEvent {
        pipeline,
        group_index,
        group: pipeline.group(group_index).expect("its group exists"),
        pass_index,
    }
}

/// A grammar-valid page whose `ui.if` owns no branch: structurally sound,
/// canonically invalid.
#[cfg(debug_assertions)]
fn empty_if_page() -> DisegnoFolio {
    DisegnoFolio::parse("[disegno]\nops=1\n\n[disegno.ops]\nui.if @0:10\n\n")
        .expect("the page is grammar-valid")
}

#[cfg(debug_assertions)]
#[test]
fn rigor_escalates_at_the_lowering_pass_and_only_there() {
    let mut verifier = VerifyObserver::new();
    assert_eq!(verifier.rigor(), Rigor::Raw);

    let pipeline = Pipeline::new("s2", WITHOUT_LOWERING);
    run_pipeline(&pipeline, &mut verifier, |_| Ok(())).expect("no step fails");
    assert_eq!(verifier.rigor(), Rigor::Raw);

    let pipeline = Pipeline::new("s2", WITH_LOWERING);
    run_pipeline(&pipeline, &mut verifier, |_| Ok(())).expect("no step fails");
    assert_eq!(verifier.rigor(), Rigor::Canonical);
}

#[cfg(debug_assertions)]
#[test]
fn note_is_the_after_pass_hook_callable_from_a_step_body() {
    let pipeline = Pipeline::new("s2", WITH_LOWERING);
    let mut verifier = VerifyObserver::new();
    verifier.note(&event_for(&pipeline, 0));
    assert_eq!(verifier.rigor(), Rigor::Raw);
    verifier.note(&event_for(&pipeline, 1));
    assert_eq!(verifier.rigor(), Rigor::Canonical);
    verifier.note(&event_for(&pipeline, 2));
    assert_eq!(verifier.rigor(), Rigor::Canonical);
}

#[cfg(debug_assertions)]
#[test]
fn the_observer_composes_through_pair_and_still_tracks_rigor() {
    let pipeline = Pipeline::new("s2", WITH_LOWERING);
    let mut pair = Pair(VerifyObserver::new(), VerifyObserver::new());
    run_pipeline(&pipeline, &mut pair, |_| Ok(())).expect("no step fails");
    assert_eq!(pair.0.rigor(), Rigor::Canonical);
    assert_eq!(pair.1.rigor(), Rigor::Canonical);
}

#[test]
fn a_holding_artifact_passes_every_check_without_panicking() {
    let pipeline = Pipeline::new("s2", LOWER_ONLY);
    let event = event_for(&pipeline, 0);
    let text = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("reference.folio"),
    )
    .expect("committed reference page reads");
    let folio = DisegnoFolio::parse(&text).expect("the reference page parses");

    let mut verifier = VerifyObserver::new();
    verifier.note(&event);
    verifier.check(&event, &folio);

    let mut table = SideTable::new();
    for index in 0..9u32 {
        table.insert(
            NodeId::from_index(index).expect("small indices have ids"),
            index,
        );
    }
    verifier.check_table(&event, &folio, &table);

    let allocator = Allocator::default();
    verifier.check_live(&event, &allocator, allocator.stamp());
}

#[cfg(debug_assertions)]
#[test]
fn check_panics_with_the_exact_report_naming_the_pass() {
    let pipeline = Pipeline::new("s2", LOWER_ONLY);
    let event = event_for(&pipeline, 0);
    let folio = empty_if_page();
    let mut verifier = VerifyObserver::new();
    verifier.note(&event);

    let payload = catch_unwind(AssertUnwindSafe(|| verifier.check(&event, &folio)))
        .expect_err("a canonically invalid page must fail the check");
    let message = payload
        .downcast::<String>()
        .expect("the report panics with a formatted string");
    assert_eq!(
        *message,
        "S2 verifier: 1 violation(s) after `s2.lower`\n\
         S2V004 @0:10 `ui.if` owns no branch"
    );
}

#[cfg(debug_assertions)]
#[test]
fn check_at_raw_rigor_accepts_what_canonical_rigor_rejects() {
    let pipeline = Pipeline::new("s2", WITHOUT_LOWERING);
    let event = event_for(&pipeline, 0);
    let folio = empty_if_page();
    let verifier = VerifyObserver::new();
    // Raw rigor: the structural set holds, so the check returns.
    verifier.check(&event, &folio);
}

#[cfg(debug_assertions)]
#[test]
fn check_table_panics_with_the_exact_report_on_a_dangling_id() {
    let pipeline = Pipeline::new("s2", &[DIAGNOSE]);
    let event = event_for(&pipeline, 0);
    let folio = DisegnoFolio::parse(
        "[disegno]\nops=2\n\n[disegno.ops]\nui.element div @0:10\n  ui.text \"x\" @2:8\n\n",
    )
    .expect("the page is grammar-valid");
    let mut table = SideTable::new();
    table.insert(
        NodeId::from_index(1).expect("index 1 has an id"),
        "resolves",
    );
    table.insert(
        NodeId::from_index(99).expect("index 99 has an id"),
        "dangles",
    );

    let verifier = VerifyObserver::new();
    let payload = catch_unwind(AssertUnwindSafe(|| {
        verifier.check_table(&event, &folio, &table)
    }))
    .expect_err("a dangling reference must fail the check");
    let message = payload
        .downcast::<String>()
        .expect("the report panics with a formatted string");
    assert_eq!(
        *message,
        "S2 verifier: 1 violation(s) after `s2.diagnose`\n\
         S2V003 @0:0 side table references %99, but the artifact numbers 2 ops"
    );
}

/// Liveness reuses the P1-11 arena-generation stamp, so the failure is
/// that mechanism's own panic — the one committed message it carries,
/// asserted here the way `vize_s0`'s own stamp tests assert it.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "arena-backed value outlived its compile")]
fn check_live_panics_on_a_stamp_from_before_the_reset() {
    let pipeline = Pipeline::new("s2", LOWER_ONLY);
    let mut allocator = Allocator::default();
    let stamp = allocator.stamp();
    allocator.reset();
    VerifyObserver::new().check_live(&event_for(&pipeline, 0), &allocator, stamp);
}

/// Guardrail 5's cfg shape, behaviourally: the debug observer carries one
/// byte of rigor, the release observer is a ZST with empty bodies (also
/// const-asserted at the type).
#[test]
fn the_observer_has_the_documented_cfg_shape() {
    #[cfg(debug_assertions)]
    assert_eq!(size_of::<VerifyObserver>(), 1);
    #[cfg(not(debug_assertions))]
    assert_eq!(size_of::<VerifyObserver>(), 0);
}
