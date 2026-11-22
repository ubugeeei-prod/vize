//! Qualify the opt-in group traversal against the established runtime, including
//! the last observable event on failure and unchanged allocation behavior.
#![cfg(feature = "streamed-pass-groups")]

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure_returning};
use std::{alloc::System, hint::black_box, time::Instant};
use vize_l0::{
    Span,
    pass::{
        FailEvent, Fusability, FusionGroup, NoObserver, Pair, PassDesc, PassEvent, PassFailure,
        PassKind, PassObserver, Pipeline, Preserved, Remark, RemarkCollector, RemarkSink,
        run_grouped_pipeline, run_pipeline_remarked,
    },
};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

const FUSE: PassDesc = PassDesc::new(
    "read",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::from_bits(0x35),
);
const FOLD: PassDesc = PassDesc::new(
    "fold",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::from_bits(0x19),
);
const BARRIER: PassDesc = PassDesc::new(
    "verify",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::ALL,
);
const OPTIONAL: PassDesc = PassDesc::new(
    "write",
    PassKind::Optional,
    Fusability::Barrier,
    Preserved::NONE,
);
const fn schedule<const N: usize>() -> [PassDesc; N] {
    let mut passes = [FUSE; N];
    let mut i = 0;
    while i < N {
        passes[i] = [FUSE, FOLD, BARRIER, OPTIONAL, FOLD, FUSE, BARRIER][i % 7];
        i += 1;
    }
    passes
}

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Begin(&'static str),
    Before(usize, usize, FusionGroup),
    Remark(usize),
    After(usize, usize, FusionGroup),
    Fail(Option<usize>, &'static str),
    End(&'static str),
}
#[derive(Default, Debug, PartialEq, Eq)]
struct Trace(Vec<Entry>);
impl PassObserver for Trace {
    const REMARKS: bool = true;
    fn before_pipeline(&mut self, p: &Pipeline) {
        self.0.push(Entry::Begin(p.stage));
    }
    fn before_pass(&mut self, e: &PassEvent<'_>) {
        self.0
            .push(Entry::Before(e.pass_index, e.group_index, e.group));
    }
    fn on_remark(&mut self, e: &PassEvent<'_>, _: &Remark<'_>) {
        self.0.push(Entry::Remark(e.pass_index));
    }
    fn after_pass(&mut self, e: &PassEvent<'_>) {
        self.0
            .push(Entry::After(e.pass_index, e.group_index, e.group));
    }
    fn on_fail(&mut self, e: &FailEvent<'_>) {
        self.0.push(Entry::Fail(e.pass_index, e.reason));
    }
    fn after_pipeline(&mut self, p: &Pipeline) {
        self.0.push(Entry::End(p.stage));
    }
}

#[test]
fn groups_members_all_hooks_remarks_and_each_failure_match_the_original_plan() {
    const MIXED: [PassDesc; 14] = schedule();
    for passes in [
        &[][..],
        &[BARRIER][..],
        &[FUSE, FOLD, FUSE][..],
        &[BARRIER, OPTIONAL, BARRIER][..],
        &MIXED[..],
    ] {
        let pipeline = Pipeline::new("L2IR", passes);
        let mut grouped = pipeline.grouped_passes();
        for reference in pipeline.groups() {
            let (group, members) = grouped.next().unwrap();
            assert_eq!(group, reference);
            assert!(std::ptr::eq(members, pipeline.passes_of(reference)));
        }
        assert_eq!(grouped.next(), None);
        assert_eq!(grouped.next(), None);

        for stop in 0..=passes.len() {
            let run = |streamed| {
                let mut observer = Pair(Trace::default(), RemarkCollector::default());
                let step = |event: &PassEvent<'_>,
                            sink: &mut vize_l0::pass::PassRemarks<
                    '_,
                    '_,
                    Pair<Trace, RemarkCollector>,
                >| {
                    let i = event.pass_index as u32;
                    sink.emit(&Remark::analysis(
                        event.desc().name,
                        Span::new(i, i + 1),
                        &[],
                    ));
                    if event.pass_index == stop {
                        Err(PassFailure::new("stop"))
                    } else {
                        Ok(())
                    }
                };
                let result = if streamed {
                    run_grouped_pipeline(&pipeline, &mut observer, step)
                } else {
                    run_pipeline_remarked(&pipeline, &mut observer, step)
                };
                (result, observer.0, observer.1.finish())
            };
            let (expected, trace, remarks) = run(false);
            assert_eq!(run(true), (expected, trace, remarks));
        }
    }
}

fn digest(pipeline: &Pipeline, stream: bool) -> usize {
    let mut digest = 0usize;
    let mut step = |event: &PassEvent<'_>,
                    _: &mut vize_l0::pass::PassRemarks<'_, '_, NoObserver>| {
        digest = digest.rotate_left(5)
            ^ event.group_index
            ^ event.pass_index
            ^ event.group.preserved.to_bits() as usize;
        Ok(())
    };
    if stream {
        run_grouped_pipeline(pipeline, &mut NoObserver, &mut step)
    } else {
        run_pipeline_remarked(pipeline, &mut NoObserver, &mut step)
    }
    .unwrap();
    digest
}

#[test]
#[ignore = "isolated paired measurements of complete observed pass execution"]
fn linear_group_dispatch_cost() {
    mark_installed();
    const SHORT: [PassDesc; 2] = [BARRIER, OPTIONAL];
    const MEDIUM: [PassDesc; 32] = schedule();
    const LONG: [PassDesc; 128] = schedule();
    const FUSED: [PassDesc; 32] = [FUSE; 32];
    const SERIAL: [PassDesc; 32] = [BARRIER; 32];
    for passes in [&SHORT[..], &FUSED[..], &SERIAL[..], &MEDIUM[..], &LONG[..]] {
        let pipeline = Pipeline::new("L3IR", passes);
        let run = |stream| digest(black_box(&pipeline), stream);
        let (before, old) = measure_returning(|| run(false));
        let (after, new) = measure_returning(|| run(true));
        assert_eq!(before, after);
        for sample in [old.unwrap(), new.unwrap()] {
            assert_eq!((sample.calls, sample.peak_bytes_over_start), (0, 0));
        }
        let time = |stream| {
            let start = Instant::now();
            for _ in 0..128 {
                black_box(run(stream));
            }
            start.elapsed().as_nanos() / 128
        };
        let (mut old, mut new) = (Vec::new(), Vec::new());
        for round in 0..35 {
            let stream = round % 2 != 0;
            let x = time(stream);
            let y = time(!stream);
            if round >= 4 {
                old.push(if stream { y } else { x });
                new.push(if stream { x } else { y });
            }
        }
        old.sort_unstable();
        new.sort_unstable();
        println!(
            "grouped passes={} walks={} median_ns={}/{} allocations=0/0 peak_bytes=0/0",
            passes.len(),
            pipeline.group_count(),
            old[15],
            new[15]
        );
        assert!(new[15] <= old[15]);
    }
}
