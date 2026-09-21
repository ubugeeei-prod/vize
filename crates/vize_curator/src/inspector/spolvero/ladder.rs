//! The Spolvero stage ladder for one template (C-2 / C-5): every rung the
//! Davinci pipeline has today, produced by the real compiler stages.
//!
//! One S1 parse feeds everything, so every page describes the same run:
//!
//! | page (`stage` / `pass`)        | producer                                              |
//! | ------------------------------ | ----------------------------------------------------- |
//! | `s1` / `parse`                 | S1 surface tree, rendered back (TS-19 fidelity)       |
//! | `s2` / `lower`                 | `vize_s1_to_s2::lower`, the S2 (Disegno) folio        |
//! | `s2-plan` / `transform`        | the executed transform plan's walks                  |
//! |                                | (`[fusion-plan-folio]`: which passes share a walk)   |
//! | `s2` / *each executed pass*    | the artifact-selected S2 transform plan, via the      |
//! |                                | pass manager and a P2-13 `FolioDump` (ungated: one   |
//! |                                | page per pass, so "did it change?" is a byte compare) |
//! | `s2-provenance` / `transform`  | every lowering and pass decision record after the    |
//! |                                | transform (`[s2-provenance-folio]`)                  |
//! | `s3` / `lower`                 | `vize_s2_to_s3::lower`, the S3 (Impeto) graph folio  |
//! | `s3-partition` / `lower`       | the exported static/dynamic partition facts          |
//! | `s3-values` / `lower`          | the S3 operand values page                           |
//!
//! Besides one step per parse, lowering and pass, the run times every
//! **walk** of the transform plan the way the timing observer does: the
//! window opens at the group's entry and closes at its exit, and the walk is
//! attributed to its lead pass. The plan page says which passes each walk
//! carried, so a fused walk is never read as one pass's cost.
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
use vize_davinci::folio::plan::FusionPlanFolio;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::{PassEvent, PassObserver, Pipeline};
use vize_s0::{Allocator, String};
use vize_s1_to_s2::pass::{TransformProfile, run_transform_with_pass_hook};
use vize_s2::folio::{S2Folio, S2ProvenanceFolio};
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
    /// Every walk of the transform plan, in run order; `pass` names the
    /// walk's lead pass (the timing observer's attribution).
    pub walks: Vec<LadderStep>,
}

/// Every stage page for `template`, in pipeline order (see the module
/// table). `path` names the source file on every page.
#[must_use]
pub fn ladder_pages(path: &str, template: &str) -> Vec<SpolveroPage> {
    ladder_run(path, template, &|| 0).pages
}

fn step(stage: &'static str, pass: &'static str, started: u64, now: u64) -> LadderStep {
    LadderStep {
        stage,
        pass,
        nanos: now.saturating_sub(started),
    }
}

/// The open pass window and the open walk window. `before_pass` opens them
/// and the pass hook closes them, each with one clock reading - the timing
/// observer's walk rule (open at the group entry, close at its exit,
/// attribute the lead pass) on a host clock.
#[derive(Debug, Default)]
struct PassWindows {
    pass: Cell<u64>,
    walk: Cell<u64>,
}

impl PassWindows {
    fn open(&self, event: &PassEvent<'_>, now: u64) {
        self.pass.set(now);
        if event.is_group_entry() {
            self.walk.set(now);
        }
    }

    /// The pass's step, and its walk's when the pass ends one.
    fn close(&self, event: &PassEvent<'_>, now: u64) -> (LadderStep, Option<LadderStep>) {
        let stage = event.pipeline.stage;
        let lead = event.pipeline.passes[event.group.start].name;
        (
            step(stage, event.desc().name, self.pass.get(), now),
            event
                .is_group_exit()
                .then(|| step(stage, lead, self.walk.get(), now)),
        )
    }
}

/// Opens the windows at `before_pass`, and keeps the plan the run executed.
struct PassStart<'a> {
    clock: LadderClock<'a>,
    windows: &'a PassWindows,
    pipeline: Option<Pipeline>,
}

impl PassObserver for PassStart<'_> {
    fn before_pipeline(&mut self, pipeline: &Pipeline) {
        self.pipeline = Some(*pipeline);
    }

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        self.windows.open(event, (self.clock)());
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
    let allocator = Allocator::default();
    let started = clock();
    let (tree, errors) = vize_s1::parse(&allocator, template);
    steps.push(step("s1", "parse", started, clock()));
    let mut s1 = String::default();
    vize_s1::render::render(&tree, &mut |slice| s1.push_str(slice));
    let mut pages = vec![page("s1", "parse", s1)];

    let started = clock();
    let mut lowered = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    steps.push(step("s2", "lower", started, clock()));
    pages.push(page("s2", "lower", s2_text(&lowered.root.ops)));

    let mut dump = FolioDump::new(false);
    let mut walks = Vec::new();
    let windows = PassWindows::default();
    let mut observer = PassStart {
        clock,
        windows: &windows,
        pipeline: None,
    };
    run_transform_with_pass_hook(
        &mut lowered,
        &mut observer,
        TransformProfile::DEFAULT,
        |event, lowered| {
            let (pass, walk) = windows.close(event, clock());
            steps.push(pass);
            walks.extend(walk);
            dump.after_pass(event, s2_text(&lowered.root.ops).as_str());
        },
    );
    if let Some(pipeline) = observer.pipeline {
        let plan = FusionPlanFolio::of(&pipeline);
        pages.push(page(
            "s2-plan",
            "transform",
            plan.print_to_string(FolioMode::Full),
        ));
    }
    pages.extend(dump.pages.into_iter().map(|dumped| SpolveroPage {
        path: Some(String::from(path)),
        stage: dumped.stage,
        pass: dumped.pass,
        text: dumped.text,
    }));
    // Every lowering and pass decision so far, in decision order: the
    // records answer "why is this op here" (and what was dropped).
    let provenance = S2ProvenanceFolio::of(&lowered.provenance);
    pages.push(page(
        "s2-provenance",
        "transform",
        provenance.print_to_string(FolioMode::Full),
    ));

    let started = clock();
    let s3 = vize_s2_to_s3::lower(&allocator, &lowered.root);
    steps.push(step("s3", "lower", started, clock()));
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
    LadderRun {
        pages,
        steps,
        walks,
    }
}

fn s2_text(ops: &[vize_s2::op::Op<'_>]) -> String {
    S2Folio::of(ops).print_to_string(FolioMode::Full)
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use vize_davinci::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved, run_pipeline};

    use super::{PassStart, PassWindows, step};

    const fn optional(name: &'static str, fusability: Fusability) -> PassDesc {
        PassDesc::new(name, PassKind::Optional, fusability, Preserved::ALL)
    }

    /// `a` and `b` share one walk; `c` owns the next.
    const PLAN: Pipeline = Pipeline::new(
        "s2",
        &[
            optional("a", Fusability::Fusable),
            optional("b", Fusability::Fusable),
            optional("c", Fusability::Barrier),
        ],
    );

    #[test]
    fn a_fused_walk_spans_its_passes_and_is_attributed_to_its_lead() {
        // Reads 0, 10, 20, ... - every window's width counts its reads.
        let reads = Cell::new(0_u64);
        let clock = || {
            let k = reads.get();
            reads.set(k + 1);
            k * 10
        };
        let windows = PassWindows::default();
        let mut observer = PassStart {
            clock: &clock,
            windows: &windows,
            pipeline: None,
        };
        let (mut steps, mut walks) = (Vec::new(), Vec::new());
        run_pipeline(&PLAN, &mut observer, |event| {
            let (pass, walk) = windows.close(event, clock());
            steps.push(pass);
            walks.extend(walk);
            Ok(())
        })
        .expect("no-op bodies cannot fail");

        assert_eq!(
            steps,
            vec![
                step("s2", "a", 0, 10),
                step("s2", "b", 20, 30),
                step("s2", "c", 40, 50),
            ]
        );
        // The fused walk opens with `a` and closes with `b`, the gap between
        // them included: one walk, not two passes' worth.
        assert_eq!(walks, vec![step("s2", "a", 0, 30), step("s2", "c", 40, 50)]);
        assert_eq!(observer.pipeline, Some(PLAN));
    }
}
