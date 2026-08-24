//! P2-3's timing law: **one profile span per walk, attributed, never one per
//! pass.**
//!
//! The failure being pinned is quiet rather than loud. Three passes fused into
//! one traversal made *one* walk; a timing observer that opened a span per
//! pass would report three, the numbers would look plausible, and every
//! decision downstream — which pass to optimize, whether fusion paid — would
//! be made against a cost that was never spent.
//!
//! # Why this test does not re-validate the export schema
//!
//! `TimingObserver` records through `vize_s0::profiler::record_attributed`,
//! which is the same serializer path P0-11's
//! `crates/vize_carton/tests/davinci_profile_export.rs` already validates
//! against `davinci-road/plan/profile-export.schema.json` — with a strict
//! validator that errors on any schema keyword it does not implement. Copying
//! that validator here would be a second implementation of the thing whose
//! duplication P0-11 exists to prevent, so this test proves the observer's
//! samples reach the export **with the right shape and attribution**, and the
//! schema conformance of that export stays pinned where the validator lives.
//!
//! The profiler is process-global, so this file holds a single `#[test]` in
//! its own binary — the `davinci_expr_reparse_floor.rs` shape.

use vize_davinci::pass::observer::TimingObserver;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved, run_pipeline};
use vize_s0::profiler::{ProfileExportBudget, ProfileExportOptions, global_profiler};

const NORMALIZE: PassDesc = PassDesc::new(
    "normalize",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const FOLD: PassDesc = PassDesc::new(
    "fold",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const VERIFY: PassDesc = PassDesc::new(
    "verify",
    PassKind::MandatoryDiagnostic,
    Fusability::Barrier,
    Preserved::ALL,
);

const KEY: &str = "davinci.test.walk";

#[test]
fn the_timing_observer_records_one_attributed_span_per_walk() {
    const PASSES: &[PassDesc] = &[NORMALIZE, FOLD, VERIFY];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);
    // Three passes, two walks: normalize+fold fuse, verify is a barrier.
    const _: () = assert!(PIPELINE.group_count() == 2);

    let profiler = global_profiler();
    profiler.enable();

    let mut timing = TimingObserver::with_key(KEY);
    run_pipeline(&PIPELINE, &mut timing, |_event| Ok(())).expect("no step fails");

    assert_eq!(
        timing.recorded_walks, 2,
        "two walks were made, so two spans were recorded - not three, one per pass"
    );

    let export = profiler.export_report(&ProfileExportOptions {
        command: "test",
        allocation: None,
        budget: ProfileExportBudget {
            max_spans: 64,
            max_counters: 64,
        },
    });

    let ours: Vec<_> = export.spans.iter().filter(|span| span.key == KEY).collect();
    assert_eq!(
        ours.len(),
        2,
        "the export must carry one entry per attributed walk, got {ours:?}"
    );

    // A walk is identified by the pass that leads its group; the passes fused
    // behind it are named by the pipeline, not by the span key.
    let mut leads: Vec<&str> = ours
        .iter()
        .map(|span| {
            span.attribution
                .as_ref()
                .and_then(|attribution| attribution.pass)
                .expect("every davinci walk span is attributed to its lead pass")
        })
        .collect();
    leads.sort_unstable();
    assert_eq!(leads, vec!["normalize", "verify"]);

    for span in &ours {
        let attribution = span
            .attribution
            .as_ref()
            .expect("every davinci walk span carries attribution");
        assert_eq!(
            attribution.stage,
            Some("s2"),
            "the stage is the pipeline's, so a span can be grouped by stage without parsing its key"
        );
        assert_eq!(span.count, 1, "each walk contributed exactly one sample");
    }

    profiler.disable();
}
