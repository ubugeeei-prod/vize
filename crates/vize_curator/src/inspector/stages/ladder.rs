//! The Spolvero stage ladder for one template (C-2 / C-5): every rung the
//! Davinci pipeline has today, produced by the real compiler layers.
//! The stage fields below retain schema-1 wire identifiers (`s1`–`s3`).
//!
//! One L1 parse feeds everything, so every page describes the same run:
//!
//! | page (`stage` / `pass`)        | producer                                              |
//! | ------------------------------ | ----------------------------------------------------- |
//! | `s1` / `parse`                 | L1 surface tree, rendered back (TS-19 fidelity)       |
//! | `s2` / `lower`                 | `vize_l1_to_l2::lower`, the L2 (Disegno) folio        |
//! | `s2-plan` / `transform`        | the executed transform plan's walks                  |
//! |                                | (`[fusion-plan-folio]`: which passes share a walk)   |
//! | `s2` / *each executed pass*    | the artifact-selected L2 transform plan, via the      |
//! |                                | pass manager and a P2-13 `Collector` (ungated: one   |
//! |                                | page per pass, so "did it change?" is a byte compare) |
//! | `s2-provenance` / `transform`  | every lowering and pass decision record after the    |
//! |                                | transform (`[s2-provenance-folio]`)                  |
//! | `s3` / `lower`                 | `vize_l2_to_l3::lower`, the L3 (Impeto) graph folio  |
//! | `s3-partition` / `lower`       | the exported static/dynamic partition facts          |
//! | `s3-values` / `lower`          | the L3 operand values page                           |
//!
//! Besides one step per parse, lowering and pass, the run times every
//! **walk** of the transform plan the way the timing observer does: the
//! window opens at the group's entry and closes at its exit, and the walk is
//! attributed to its lead pass. The plan page says which passes each walk
//! carried, so a fused walk is never read as one pass's cost.
//!
//! L3 lowers from the post-transform L2 root. On the Vue 3 path the
//! transform passes preserve the tree (their products are side tables), so
//! this is the root the Vapor/SSR bridges lower too; the byte-identical L2
//! pass pages are the observable proof.
//!
//! Stage names ending in a page family (`s3-partition`, `s3-values`) keep
//! `(stage, pass)` unique within one template's pages while all three L3
//! pages come from the one lowering step.

use core::cell::Cell;

use vize_l0::dump::collector::Collector;
use vize_l0::dump::plan::Page as PlanPage;
use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::pass::{Pair, PassEvent, PassObserver, Pipeline, RemarkCollector};
use vize_l0::{Allocator, String};
use vize_l1_to_l2::pass::{TransformProfile, run_transform_with_pass_hook};
use vize_l2::dump::Page as L2Page;
use vize_l2::dump::provenance::Page as ProvenancePage;
use vize_l2_to_l3::partition::dump::Page as PartitionPage;
use vize_l3::dump::Page as L3Page;
use vize_l3::values_dump::Page as ValuesPage;

use super::{StagePage, StageRemark, StageUnavailable, inspection};

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
    pub pages: Vec<StagePage>,
    /// Failed inspections from this run, separate from valid dump pages.
    pub unavailable: Vec<StageUnavailable>,
    /// Optimization decisions collected during the same L2 pass execution.
    pub remarks: Vec<StageRemark>,
    /// Every step, in run order.
    pub steps: Vec<LadderStep>,
    /// Every walk of the transform plan, in run order; `pass` names the
    /// walk's lead pass (the timing observer's attribution).
    pub walks: Vec<LadderStep>,
}

/// Every stage page for `template`, in pipeline order (see the module
/// table). `path` names the source file on every page.
#[must_use]
pub fn ladder_pages(path: &str, template: &str) -> Vec<StagePage> {
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
        let lead = event.pipeline.passes.get(event.group.start);
        (
            step(stage, event.desc().name, self.pass.get(), now),
            lead.filter(|_| event.is_group_exit())
                .map(|lead| step(stage, lead.name, self.walk.get(), now)),
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
    let page = |stage: &str, pass: &str, text: String| StagePage {
        path: Some(String::from(path)),
        stage: String::from(stage),
        pass: String::from(pass),
        text,
    };
    let mut steps = Vec::new();
    let mut unavailable = Vec::new();
    let allocator = Allocator::default();
    let started = clock();
    let (tree, errors) = vize_l1::parse(&allocator, template);
    steps.push(step("s1", "parse", started, clock()));
    let mut s1 = String::default();
    vize_l1::render::render(&tree, &mut |slice| s1.push_str(slice));
    let mut pages = vec![page("s1", "parse", s1)];

    let started = clock();
    let mut lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    steps.push(step("s2", "lower", started, clock()));
    if let Some(text) = inspection::record(
        &mut unavailable,
        path,
        "s2",
        "lower",
        l2_text(&lowered.root.ops),
    ) {
        pages.push(page("s2", "lower", text));
    }

    let mut dump = Collector::new(false);
    let mut walks = Vec::new();
    let windows = PassWindows::default();
    let mut observer = Pair(
        PassStart {
            clock,
            windows: &windows,
            pipeline: None,
        },
        RemarkCollector::new(),
    );
    run_transform_with_pass_hook(
        &mut lowered,
        &mut observer,
        TransformProfile::DEFAULT,
        |event, lowered| {
            let (pass, walk) = windows.close(event, clock());
            steps.push(pass);
            walks.extend(walk);
            if let Some(text) = inspection::record(
                &mut unavailable,
                path,
                event.pipeline.stage,
                event.desc().name,
                l2_text(&lowered.root.ops),
            ) {
                dump.after_pass(event, text.as_str());
            }
        },
    );
    if let Some(pipeline) = observer.0.pipeline {
        let plan = PlanPage::of(&pipeline);
        pages.push(page(
            "s2-plan",
            "transform",
            plan.print_to_string(DumpMode::Full),
        ));
    }
    pages.extend(dump.pages.into_iter().map(|dumped| StagePage {
        path: Some(String::from(path)),
        stage: dumped.stage,
        pass: dumped.pass,
        text: dumped.text,
    }));
    // Every lowering and pass decision so far, in decision order: the
    // records answer "why is this op here" (and what was dropped).
    let provenance = ProvenancePage::of(&lowered.provenance);
    pages.push(page(
        "s2-provenance",
        "transform",
        provenance.print_to_string(DumpMode::Full),
    ));

    let started = clock();
    let s3 = vize_l2_to_l3::lower(&allocator, &lowered.root);
    steps.push(step("s3", "lower", started, clock()));
    let program = L3Page::of(&s3.program);
    pages.push(page("s3", "lower", program.print_to_string(DumpMode::Full)));
    let partition = PartitionPage::of(&s3.partition);
    pages.push(page(
        "s3-partition",
        "lower",
        partition.print_to_string(DumpMode::Full),
    ));
    let values = ValuesPage::of(&s3.program);
    pages.push(page(
        "s3-values",
        "lower",
        values.print_to_string(DumpMode::Full),
    ));
    let remarks = observer
        .1
        .finish()
        .into_iter()
        .map(|remark| StageRemark {
            path: Some(String::from(path)),
            remark,
        })
        .collect();
    LadderRun {
        pages,
        unavailable,
        remarks,
        steps,
        walks,
    }
}

fn l2_text(ops: &[vize_l2::op::Op<'_>]) -> Result<String, vize_l2::dump::NativeDumpError> {
    L2Page::of(ops).map(|page| page.print_to_string(DumpMode::Full))
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use vize_l0::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved, run_pipeline};

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
