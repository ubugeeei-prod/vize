// Same-run product capture for the Vue DOM emitter's selected L2 plan.

use core::cell::RefCell;

use vize_l0::dump::capture::{CaptureArg, CaptureArgValue, CaptureRemark, CaptureSink};
use vize_l0::dump::plan::Page as PlanPage;
use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::level::Level;
use vize_l0::pass::observer::RemarkArgValue;
use vize_l0::pass::{
    FailEvent, PassEvent, PassObserver, Pipeline, Remark, RemarkCollector, WalkTiming,
};
use vize_l0::{Allocator, String, ensure_sufficient_stack};
use vize_l1::markup::{SurfaceParseOptions, parse_with_options_and_caps};
use vize_l2::dump::{Page as L2Page, ProvenancePage};

use crate::lower::{LegacyCaps, LowerPolicy, Lowered, lower_with_caps_and_comment_policy};
use crate::pass::{TransformProfile, run_transform_with_pass_hook};

use super::super::EmitError;
use super::super::run::{DomEmitObservation, emit_dom_observed};
use super::EmitInvocation;

struct TimingState {
    pass_started: Option<u64>,
    walk: WalkTiming<u64>,
    complete: bool,
    failed: bool,
}

impl TimingState {
    const fn new(complete: bool) -> Self {
        Self {
            pass_started: None,
            walk: WalkTiming::new(),
            complete,
            failed: false,
        }
    }
}

struct ProductObserver<'a, 'c, O, C> {
    downstream: &'a mut O,
    capture: &'a RefCell<&'c mut C>,
    timing: &'a RefCell<TimingState>,
    remarks: RemarkCollector,
}

impl<O: PassObserver, C: CaptureSink> PassObserver for ProductObserver<'_, '_, O, C> {
    const REMARKS: bool = true;

    fn before_pipeline(&mut self, pipeline: &Pipeline) {
        self.downstream.before_pipeline(pipeline);
        self.timing.borrow_mut().walk.discard();
        self.capture.borrow_mut().page(Level::L2, "plan", || {
            PlanPage::of(pipeline).print_to_string(DumpMode::Full)
        });
    }

    fn after_pipeline(&mut self, pipeline: &Pipeline) {
        self.downstream.after_pipeline(pipeline);
    }

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        self.downstream.before_pass(event);
        self.remarks.before_pass(event);
        let started = self.capture.borrow_mut().clock_now();
        let mut timing = self.timing.borrow_mut();
        timing.pass_started = started;
        timing.walk.begin(event, || started);
        timing.complete &= started.is_some();
    }

    fn after_pass(&mut self, event: &PassEvent<'_>) {
        self.downstream.after_pass(event);
    }

    fn on_fail(&mut self, event: &FailEvent<'_>) {
        self.downstream.on_fail(event);
        let mut timing = self.timing.borrow_mut();
        timing.failed = true;
        timing.complete = false;
        timing.walk.discard();
    }

    fn on_remark(&mut self, event: &PassEvent<'_>, remark: &Remark<'_>) {
        if O::REMARKS {
            self.downstream.on_remark(event, remark);
        }
        self.remarks.on_remark(event, remark);
    }
}

fn record_step<C: CaptureSink>(
    capture: &mut C,
    level: Level,
    step: &'static str,
    started: Option<u64>,
) -> bool {
    let Some(started) = started else {
        return false;
    };
    let Some(finished) = capture.clock_now() else {
        return false;
    };
    capture.timing(level, step, || finished.saturating_sub(started));
    true
}

fn after_pass<C: CaptureSink>(
    capture: &RefCell<&mut C>,
    timing: &RefCell<TimingState>,
    event: &PassEvent<'_>,
    lowered: &Lowered<'_>,
) {
    let finished = capture.borrow_mut().clock_now();
    let mut timing = timing.borrow_mut();
    let started = timing.pass_started.take();
    let walk = timing.walk.end(event);
    timing.complete &= started.is_some() && finished.is_some();
    if let (Some(started), Some(finished)) = (started, finished) {
        capture
            .borrow_mut()
            .timing(Level::L2, event.desc().name, || {
                finished.saturating_sub(started)
            });
    }
    if let (Some((started, attribution)), Some(finished)) = (walk, finished) {
        let lead = attribution.pass.unwrap_or(event.desc().name);
        capture
            .borrow_mut()
            .walk_timing(Level::L2, lead, || finished.saturating_sub(started));
    }
    drop(timing);
    capture.borrow_mut().page(Level::L2, event.desc().name, || {
        L2Page::of(&lowered.artifact.root.ops).print_to_string(DumpMode::Full)
    });
}

fn retain_channels<O: PassObserver, C: CaptureSink>(observer: &mut ProductObserver<'_, '_, O, C>) {
    let timing = observer.timing.borrow();
    if timing.failed {
        return;
    }
    let timing_complete = timing.complete;
    drop(timing);
    let remarks = core::mem::take(&mut observer.remarks).finish();
    let mut capture = observer.capture.borrow_mut();
    for remark in remarks {
        capture.remark(move || CaptureRemark {
            level: Level::L2,
            pass: remark.pass,
            kind: String::from(remark.kind.as_str()),
            name: remark.name,
            span: remark.span,
            args: remark
                .args
                .into_iter()
                .map(|arg| CaptureArg {
                    name: arg.key,
                    value: match arg.value {
                        RemarkArgValue::Str(value) => CaptureArgValue::Str(value),
                        RemarkArgValue::Int(value) => CaptureArgValue::Int(value),
                        RemarkArgValue::Bool(value) => CaptureArgValue::Bool(value),
                    },
                })
                .collect(),
        });
    }
    capture.remark_channel_observed();
    if timing_complete {
        capture.timing_channel_observed();
    }
}

/// Compile through the artifact-selected pass manager and observe only that run.
pub(super) fn emit_dom_source_captured<'a, O: PassObserver, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
    invocation: EmitInvocation<'_, '_, '_>,
    downstream: &mut O,
    capture: &mut C,
) -> Result<DomEmitObservation, EmitError> {
    let EmitInvocation {
        options,
        syntax,
        strict_slot_params,
        no_slotted,
        development,
        preflight,
    } = invocation;
    ensure_sufficient_stack(|| {
        let started = capture.clock_now();
        let (tree, errors) = parse_with_options_and_caps(
            allocator,
            source,
            SurfaceParseOptions {
                experimental_in_tag_comments: options.experimental_in_tag_comments,
                syntax,
            },
            caps,
        );
        let mut timing_complete = record_step(capture, Level::L1, "parse", started);
        if let Some(preflight) = preflight
            && !preflight.check(Some((&tree, &errors)))
        {
            return Err(EmitError::Diagnostics);
        }
        capture.page(Level::L1, "parse", || {
            let mut text = String::default();
            vize_l1::markup::render::render(&tree, &mut |slice| text.push_str(slice));
            text
        });

        let started = capture.clock_now();
        let mut lowered = lower_with_caps_and_comment_policy(
            allocator,
            &tree,
            &errors,
            caps,
            LowerPolicy::compile(options.comments),
            options.custom_element_patterns,
            options.custom_element_predicate,
        );
        timing_complete &= record_step(capture, Level::L2, "lower", started);
        capture.page(Level::L2, "lower", || {
            L2Page::of(&lowered.artifact.root.ops).print_to_string(DumpMode::Full)
        });

        let mut profile = TransformProfile::DEFAULT;
        if !options.hoist_static {
            profile = profile.without_static_analysis();
        }
        let capture_cell = RefCell::new(capture);
        let timing = RefCell::new(TimingState::new(timing_complete));
        let facts = {
            let mut observer = ProductObserver {
                downstream,
                capture: &capture_cell,
                timing: &timing,
                remarks: RemarkCollector::new(),
            };
            let facts = run_transform_with_pass_hook(
                &mut lowered,
                &mut observer,
                profile,
                |event, lowered| after_pass(&capture_cell, &timing, event, lowered),
            );
            retain_channels(&mut observer);
            facts
        };
        let capture = capture_cell.into_inner();
        capture.page(Level::L2, "provenance", || {
            ProvenancePage::of(&lowered.artifact.provenance).print_to_string(DumpMode::Full)
        });
        capture.page(Level::L2, "facts", || vize_l0::cstr!("{facts:#?}"));
        let emitted = emit_dom_observed(
            &lowered,
            &facts,
            options,
            strict_slot_params,
            no_slotted,
            development,
        )?;
        capture.page(Level::L4, "emit", || emitted.emit.assembled());
        Ok(emitted)
    })
}
