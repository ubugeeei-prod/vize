use super::*;
use davinci_harness::alloc;
use std::{fmt::Write, hint::black_box, time::Instant};
use vize_l2::query::script::assignments::prefix::LiteralPrefixIndex;

fn time<T>(mut run: impl FnMut() -> T) -> u128 {
    let start = Instant::now();
    for _ in 0..24 {
        black_box(run());
    }
    start.elapsed().as_nanos() / 24
}

fn timed(name: &str, original: &str) {
    let old = SourceRoot::new(original).unwrap().whole_block();
    let arena = Allocator::new();
    let syntax = AuthoredProgram::parse(&arena, old, SourceType::ts());
    let model = LiteralAssignments::new(&syntax).unwrap();
    let proof = model.prove();
    let idx = LiteralPrefixIndex::collect(old, &model.check(&proof).unwrap()).unwrap();
    let changed = format!("{original}\nconst half: = ;");
    let root = SourceRoot::new(&changed).unwrap();
    let whole = root.whole_block();
    let proven_end = idx
        .revalidate(whole, SourceType::ts())
        .unwrap()
        .covered()
        .end as usize;
    let prefix = root.block(&changed[..proven_end], 0).unwrap();
    let checked = model.check(&proof).unwrap();
    let construction =
        alloc::measure(|| LiteralPrefixIndex::collect(old, &checked).unwrap()).unwrap();
    eprintln!(
        "retained literal index {name} build allocations={} peak={}",
        construction.calls, construction.peak_bytes_over_start
    );
    // The rebuild gets the correct fence for free. It reparses exactly the same
    // original bytes and produces the same cursor result in the new document.
    let offset = original.rfind("binding").unwrap() as u32;
    let run = |incremental| {
        if incremental {
            idx.revalidate(black_box(whole), SourceType::ts())
                .unwrap()
                .binding_at(black_box(offset))
        } else {
            let arena = Allocator::new();
            let program = AuthoredProgram::parse(&arena, black_box(prefix), SourceType::ts());
            let query = LiteralAssignments::new(&program).unwrap();
            let rows = query.prove();
            query
                .check(&rows)
                .unwrap()
                .assignments()
                .find(|r| r.binding.start <= offset && offset < r.binding.end)
                .copied()
        }
    };
    assert_eq!(run(true), run(false));
    assert!(run(true).is_some());
    let old_cost = alloc::measure(|| run(false)).unwrap();
    let new_cost = alloc::measure(|| run(true)).unwrap();
    let batch = |v| time(|| run(black_box(v)));
    let mut fresh = Vec::new();
    let mut saved = Vec::new();
    for round in 0..35 {
        let (a, b) = if round % 2 == 0 {
            (batch(false), batch(true))
        } else {
            let b = batch(true);
            (batch(false), b)
        };
        if round >= 4 {
            fresh.push(a);
            saved.push(b);
        }
    }
    fresh.sort_unstable();
    saved.sort_unstable();
    let a = fresh[15];
    let b = saved[15];
    eprintln!(
        "recovered literal cursor {name} ns={a}/{b} calls={}/{} peak={}/{}",
        old_cost.calls,
        new_cost.calls,
        old_cost.peak_bytes_over_start,
        new_cost.peak_bytes_over_start
    );
    assert!(
        b <= a
            && new_cost.calls <= old_cost.calls
            && new_cost.peak_bytes_over_start <= old_cost.peak_bytes_over_start,
        "{name}"
    );
    // A first open has no cached syntax or known boundary. Charge the whole
    // failed parse, all checked prefix probes, and the actual cursor query.
    let open = || {
        let idx = LiteralPrefixIndex::open(black_box(whole), SourceType::ts()).unwrap();
        idx.revalidate(whole, SourceType::ts())
            .unwrap()
            .binding_at(black_box(offset))
    };
    assert_eq!(open(), run(true));
    let cost = alloc::measure(open).unwrap();
    for _ in 0..4 {
        black_box(time(open));
    }
    let mut times = [0; 31];
    for t in &mut times {
        *t = time(open);
    }
    times.sort_unstable();
    eprintln!(
        "first open broken cursor {name} ns={} calls={} peak={}",
        times[15], cost.calls, cost.peak_bytes_over_start
    );
}

#[test]
#[ignore = "paired hot edit and fresh certified prefix rebuild with exact matching responses"]
fn incremental_cursor_cost() {
    alloc::mark_installed();
    for n in [1, 32, 256] {
        let mut source = String::new();
        for i in 0..n {
            writeln!(source, "const binding{i}: boolean = true;").unwrap();
        }
        timed(&format!("{n} bindings"), &source);
    }
}
