use super::*;
use davinci_harness::alloc::{self, AllocMetrics, CountingAllocator};
use std::{hint::black_box, time::Instant};

#[global_allocator]
static COUNTER: CountingAllocator = CountingAllocator::mimalloc();

#[derive(Debug, Clone, Copy)]
enum Demand {
    First,
    FirstBorrow,
    Repeated,
    Both,
    Different,
}

fn source(rows: usize) -> String {
    use core::fmt::Write;
    if rows == 1 {
        return String::from("export const x = 1;\n");
    }
    let mut text = String::from("import Z from './z';\nimport A from './a';\n");
    for i in 0..rows {
        writeln!(text, "export const item{i}:string=make(A,Z,'雪{i}');").unwrap();
    }
    text
}

struct Input<'q, 'a> {
    arena: &'q Allocator,
    program: &'q EmbedSyntax<'a>,
    defaults: FormatOptions,
    varied: FormatOptions,
    sort: ImportSortOptions,
}

impl<'q, 'a> Input<'q, 'a> {
    fn new(arena: &'q Allocator, program: &'q EmbedSyntax<'a>) -> Self {
        Self {
            arena,
            program,
            defaults: FormatOptions::default(),
            varied: variation(),
            sort: ImportSortOptions::default(),
        }
    }

    fn original(&self, demand: Demand) -> usize {
        let source = self.program.source().text();
        let ty = self.program.source_type();
        let read =
            |opts, sort| format_script_content_stable(source, opts, self.arena, ty, sort).unwrap();
        let len = black_box(read(&self.defaults, None)).len();
        match demand {
            Demand::Both => {
                len + black_box(
                    format_sfc_script_content_stable(source, &self.defaults, self.arena, ty, None)
                        .unwrap(),
                )
                .len()
            }
            Demand::Different => len + black_box(read(&self.varied, Some(&self.sort))).len(),
            _ => len,
        }
    }

    fn candidate(&self, demand: Demand) -> usize {
        let query = ScriptFormatQuery::new(self.arena, self.program, &self.defaults, None).unwrap();
        match demand {
            Demand::First => black_box(query.into_standalone().unwrap()).len(),
            Demand::Both => {
                black_box(query.standalone().unwrap()).len()
                    + black_box(query.sfc_body().unwrap()).len()
            }
            Demand::Different => {
                let len = black_box(query.into_standalone().unwrap()).len();
                let second = ScriptFormatQuery::new(
                    self.arena,
                    self.program,
                    &self.varied,
                    Some(&self.sort),
                )
                .unwrap();
                len + black_box(second.into_standalone().unwrap()).len()
            }
            Demand::FirstBorrow | Demand::Repeated => black_box(query.standalone().unwrap()).len(),
        }
    }
}

fn window(
    input: &Input<'_, '_>,
    demand: Demand,
    candidate: bool,
    time: bool,
) -> (usize, u128, Option<AllocMetrics>) {
    // Each default product call returns its own owned output. A shared query
    // additionally lets the next consumer borrow the already-completed result.
    let retained = if matches!(demand, Demand::Repeated) && candidate {
        let query =
            ScriptFormatQuery::new(input.arena, input.program, &input.defaults, None).unwrap();
        black_box(query.standalone().unwrap());
        Some(query)
    } else {
        if matches!(demand, Demand::Repeated) {
            black_box(input.original(demand));
        }
        None
    };
    let read = || {
        if let Some(query) = &retained {
            black_box(black_box(query).standalone().unwrap()).len()
        } else if candidate {
            input.candidate(demand)
        } else {
            input.original(demand)
        }
    };
    if time {
        let repeats = if matches!(demand, Demand::Repeated) && candidate {
            128
        } else {
            1
        };
        let start = Instant::now();
        let mut answer = 0;
        for _ in 0..repeats {
            answer = black_box(read());
        }
        (answer, start.elapsed().as_nanos() / repeats, None)
    } else {
        let (out, metrics) = alloc::measure_returning(read);
        (out, 0, metrics)
    }
}

#[test]
#[ignore = "paired results for complete formatting, shared consumption, changed settings and allocations"]
fn paired_formatter_requests() {
    alloc::mark_installed();
    for size in [1, 32, 128] {
        let text = source(size);
        let arena = Allocator::default();
        let block = SourceRoot::new(&text).unwrap().whole_block();
        let program = parse_program_once(
            &arena,
            EmbedSource::authored(&text, block.span()).unwrap(),
            ProgramOptions::module(Lang::Ts),
        );
        let input = Input::new(&arena, &program);
        let construction = alloc::measure(|| {
            black_box(ScriptFormatQuery::new(&arena, &program, &input.defaults, None).unwrap());
        })
        .unwrap();
        assert_eq!(
            (construction.calls, construction.peak_bytes_over_start),
            (0, 0)
        );
        for demand in [
            Demand::First,
            Demand::FirstBorrow,
            Demand::Repeated,
            Demand::Both,
            Demand::Different,
        ] {
            let mut pairs = Vec::new();
            for i in 0..35 {
                let reverse = i % 2 == 1;
                let a = window(&input, demand, reverse, true);
                let b = window(&input, demand, !reverse, true);
                assert_eq!(a.0, b.0);
                if i >= 4 {
                    pairs.push(if reverse { (b.1, a.1) } else { (a.1, b.1) });
                }
            }
            let (a, _, base) = window(&input, demand, false, false);
            let (b, _, head) = window(&input, demand, true, false);
            assert_eq!(a, b);
            let (base, head) = (base.unwrap(), head.unwrap());
            assert!(head.calls <= base.calls, "{demand:?}");
            assert!(
                head.peak_bytes_over_start <= base.peak_bytes_over_start,
                "{demand:?}: {head:?} vs {base:?}"
            );
            if matches!(demand, Demand::Repeated) {
                assert_eq!((head.calls, head.peak_bytes_over_start), (0, 0));
            }
            eprintln!(
                "size={size} demand={demand:?} calls={}/{} peak={}/{} pairs={pairs:?}",
                base.calls, head.calls, base.peak_bytes_over_start, head.peak_bytes_over_start
            );
        }
    }
}

mod corpus;
