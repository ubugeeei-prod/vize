use super::*;
use davinci_harness::alloc::{self, AllocMetrics, CountingAllocator};
use std::{hint::black_box, time::Instant};

#[global_allocator]
static COUNTER: CountingAllocator = CountingAllocator::mimalloc();

#[derive(Debug, Clone, Copy)]
enum Read {
    FirstSetup,
    BothPrograms,
    RepeatedSetup,
    Semantic,
    SemanticAndSetup,
}

fn source(rows: usize) -> String {
    use core::fmt::Write;
    let mut text = String::from(
        "<script lang='ts'>export default {props:['name']}</script><script setup lang='ts'>/* setup */\n",
    );
    for n in 0..rows {
        writeln!(text, "const price{n}: number = get({n});").unwrap();
    }
    text.push_str("</script><template>{{ price0 }}</template>");
    text
}

fn syntax_units(program: &EmbedSyntax<'_>) -> usize {
    black_box(program).program().unwrap().body.len()
}

fn semantic_units(producer: VueFileProducer<'_>) -> usize {
    let file = producer.finish().unwrap();
    black_box(file.file().units().len() + file.file().bindings().count())
}

fn run(view: AdmittedDescriptor<'_, '_>, read: Read, shared: bool) -> usize {
    let arena = Allocator::default();
    if matches!(read, Read::FirstSetup | Read::RepeatedSetup) {
        return view.setup().map_or(0, |selected| {
            let syntax = if shared {
                selected.observe_program(&arena)
            } else {
                direct(&arena, selected)
            };
            syntax_units(&syntax)
        });
    }
    let (needs_file, second_setup) = (
        matches!(read, Read::Semantic | Read::SemanticAndSetup),
        matches!(read, Read::SemanticAndSetup),
    );
    if shared {
        view.with_script_programs(&arena, |programs| {
            if needs_file {
                let summary = semantic_units(
                    VueFileProducer::from_script_programs(&arena, programs, ALL).unwrap(),
                );
                if second_setup {
                    summary + programs.get(ScriptRole::Setup).map_or(0, syntax_units)
                } else {
                    summary
                }
            } else {
                programs.get(ScriptRole::Ordinary).map_or(0, syntax_units)
                    + programs.get(ScriptRole::Setup).map_or(0, syntax_units)
            }
        })
    } else {
        // The established Vue producer accepts absent roles and retains each selected Program.
        let ordinary = view.ordinary().map(|selected| direct(&arena, selected));
        let setup = view.setup().map(|selected| direct(&arena, selected));
        if needs_file {
            let summary = semantic_units(
                VueFileProducer::scripts(
                    &arena,
                    view,
                    ordinary.as_ref().and_then(EmbedSyntax::admitted_program),
                    setup.as_ref().and_then(EmbedSyntax::admitted_program),
                    ALL,
                )
                .unwrap(),
            );
            if second_setup {
                summary
                    + view
                        .setup()
                        .map_or(0, |selected| syntax_units(&direct(&arena, selected)))
            } else {
                summary
            }
        } else {
            ordinary.as_ref().map_or(0, syntax_units) + setup.as_ref().map_or(0, syntax_units)
        }
    }
}

fn window(
    view: AdmittedDescriptor<'_, '_>,
    read: Read,
    shared: bool,
    time: bool,
) -> (usize, u128, Option<AllocMetrics>) {
    let arena = Allocator::default();
    let warmed = if matches!(read, Read::RepeatedSetup) && shared {
        Some(view.setup().unwrap().observe_program(&arena))
    } else {
        if matches!(read, Read::RepeatedSetup) {
            black_box(direct(&arena, view.setup().unwrap()));
        }
        None
    };
    let request = || {
        if let Some(program) = &warmed {
            syntax_units(black_box(program))
        } else {
            run(view, read, shared)
        }
    };
    if time {
        let repeats = if warmed.is_some() { 512 } else { 16 };
        let start = Instant::now();
        let mut answer = 0;
        for _ in 0..repeats {
            answer = black_box(request());
        }
        (answer, start.elapsed().as_nanos() / repeats, None)
    } else {
        let (answer, alloc) = alloc::measure_returning(request);
        (answer, 0, alloc)
    }
}

#[test]
#[ignore = "counterbalanced first, repeated and composed requests with allocation windows"]
fn shared_program_requests() {
    alloc::mark_installed();
    for n in [1, 32, 128] {
        let arena = Allocator::default();
        let text = source(n);
        let owner = Vue.observe_descriptor(&arena, &text, options());
        let view = owner.admitted().unwrap();
        for read in [
            Read::FirstSetup,
            Read::BothPrograms,
            Read::RepeatedSetup,
            Read::Semantic,
            Read::SemanticAndSetup,
        ] {
            let mut pairs = Vec::new();
            for i in 0..35 {
                let reverse = i % 2 == 1;
                let a = window(view, read, reverse, true);
                let b = window(view, read, !reverse, true);
                assert_eq!(a.0, b.0);
                if i >= 4 {
                    pairs.push(if reverse { (b.1, a.1) } else { (a.1, b.1) });
                }
            }
            let (a, _, left) = window(view, read, false, false);
            let (b, _, right) = window(view, read, true, false);
            assert_eq!(a, b);
            let (left, right) = (left.unwrap(), right.unwrap());
            assert!(
                right.calls <= left.calls
                    && right.peak_bytes_over_start <= left.peak_bytes_over_start,
                "{n} {read:?} {left:?}/{right:?}"
            );
            if matches!(read, Read::RepeatedSetup) {
                assert_eq!((right.calls, right.peak_bytes_over_start), (0, 0));
            }
            eprintln!(
                "rows={n} read={read:?} calls={}/{} peak={}/{} pairs={pairs:?}",
                left.calls, right.calls, left.peak_bytes_over_start, right.peak_bytes_over_start
            );
        }
    }
}

mod corpus;
