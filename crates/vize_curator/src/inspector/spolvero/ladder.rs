//! The Spolvero stage ladder for one template (C-2 / C-5): every rung the
//! Davinci pipeline has today, produced by the real compiler stages.
//!
//! One S1 parse feeds everything, so every page describes the same run:
//!
//! | page (`stage` / `pass`)        | producer                                              |
//! | ------------------------------ | ----------------------------------------------------- |
//! | `s1` / `parse`                 | S1 surface tree, rendered back (TS-19 fidelity)       |
//! | `s2` / `lower`                 | `vize_s1_to_s2::lower`, the S2 (Disegno) folio        |
//! | `s2` / *each executed pass*    | the artifact-selected S2 transform plan, via the      |
//! |                                | pass manager and a P2-13 `FolioDump` (ungated: one   |
//! |                                | page per pass, so "did it change?" is a byte compare) |
//! | `s3` / `lower`                 | `vize_s2_to_s3::lower`, the S3 (Impeto) graph folio  |
//! | `s3-partition` / `lower`       | the exported static/dynamic partition facts          |
//! | `s3-values` / `lower`          | the S3 operand values page                           |
//!
//! S3 lowers from the post-transform S2 root. On the Vue 3 path the
//! transform passes preserve the tree (their products are side tables), so
//! this is the root the Vapor/SSR bridges lower too; the byte-identical S2
//! pass pages are the observable proof.
//!
//! Stage names ending in a page family (`s3-partition`, `s3-values`) keep
//! `(stage, pass)` unique within one template's pages while all three S3
//! pages come from the one lowering step.

use core::cell::Cell;

use vize_davinci::folio::dump::FolioDump;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::{PassEvent, PassObserver};
use vize_s0::{Allocator, String};
use vize_s1_to_s2::pass::{TransformProfile, run_transform_with_pass_hook};
use vize_s2::folio::S2Folio;
use vize_s2_to_s3::S3PartitionFolio;
use vize_s3::folio::S3Folio;
use vize_s3::values_folio::S3ValuesFolio;

use super::SpolveroPage;

/// A monotonic clock in nanoseconds, supplied by the host. The library takes
/// no clock of its own: native hosts can pass `Instant`, the browser build
/// passes `performance.now()`, and tests pass a deterministic counter.
pub type LadderClock<'c> = &'c dyn Fn() -> u64;

/// One timed ladder step: a parse, a lowering, or one executed pass.
///
/// The window covers the compiler work only - page printing happens after
/// the clock is read - so a step's cost is never the cost of observing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LadderStep {
    /// Stage that ran the step (`s1`, `s2`, `s3`).
    pub stage: &'static str,
    /// The step: `parse`, `lower`, or the pass name.
    pub pass: &'static str,
    /// Wall time by the host clock.
    pub nanos: u64,
}

/// The pages and the step timings of one ladder run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderRun {
    /// Every stage page, in pipeline order (see the module table).
    pub pages: Vec<SpolveroPage>,
    /// Every step, in run order.
    pub steps: Vec<LadderStep>,
}

/// Every stage page for `template`, in pipeline order (see the module
/// table). `path` names the source file on every page.
#[must_use]
pub fn ladder_pages(path: &str, template: &str) -> Vec<SpolveroPage> {
    ladder_run(path, template, &|| 0).pages
}

/// Opens a pass's window at `before_pass`; the pass hook closes it.
struct PassStart<'a> {
    clock: LadderClock<'a>,
    started: &'a Cell<u64>,
}

impl PassObserver for PassStart<'_> {
    fn before_pass(&mut self, _event: &PassEvent<'_>) {
        self.started.set((self.clock)());
    }
}

/// [`ladder_pages`] plus the wall time of every step by `clock`.
#[must_use]
pub fn ladder_run(path: &str, template: &str, clock: LadderClock<'_>) -> LadderRun {
    let page = |stage: &str, pass: &str, text: String| SpolveroPage {
        path: Some(String::from(path)),
        stage: String::from(stage),
        pass: String::from(pass),
        text,
    };
    let mut steps = Vec::new();
    let mut timed = |stage: &'static str, pass: &'static str, started: u64| {
        let nanos = clock().saturating_sub(started);
        steps.push(LadderStep { stage, pass, nanos });
    };

    let allocator = Allocator::default();
    let started = clock();
    let (tree, errors) = vize_s1::parse(&allocator, template);
    timed("s1", "parse", started);
    let mut s1 = String::default();
    vize_s1::render::render(&tree, &mut |slice| s1.push_str(slice));
    let mut pages = vec![page("s1", "parse", s1)];

    let started = clock();
    let mut lowered = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    timed("s2", "lower", started);
    pages.push(page("s2", "lower", s2_text(&lowered.root.ops)));

    let mut dump = FolioDump::new(false);
    let pass_started = Cell::new(0);
    let mut observer = PassStart {
        clock,
        started: &pass_started,
    };
    run_transform_with_pass_hook(
        &mut lowered,
        &mut observer,
        TransformProfile::DEFAULT,
        |event, lowered| {
            timed(event.pipeline.stage, event.desc().name, pass_started.get());
            dump.after_pass(event, s2_text(&lowered.root.ops).as_str());
        },
    );
    pages.extend(dump.pages.into_iter().map(|dumped| SpolveroPage {
        path: Some(String::from(path)),
        stage: dumped.stage,
        pass: dumped.pass,
        text: dumped.text,
    }));

    let started = clock();
    let s3 = vize_s2_to_s3::lower(&allocator, &lowered.root);
    timed("s3", "lower", started);
    let program = S3Folio::of(&s3.program);
    pages.push(page(
        "s3",
        "lower",
        program.print_to_string(FolioMode::Full),
    ));
    let partition = S3PartitionFolio::of(&s3.partition);
    pages.push(page(
        "s3-partition",
        "lower",
        partition.print_to_string(FolioMode::Full),
    ));
    let values = S3ValuesFolio::of(&s3.program);
    pages.push(page(
        "s3-values",
        "lower",
        values.print_to_string(FolioMode::Full),
    ));
    LadderRun { pages, steps }
}

fn s2_text(ops: &[vize_s2::op::Op<'_>]) -> String {
    S2Folio::of(ops).print_to_string(FolioMode::Full)
}
