use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{alloc::System, hint::black_box, time::Instant};

#[global_allocator]
static ALLOC: CountingAllocator<System> = CountingAllocator::system();

#[test]
#[ignore = "alternating full product requests on the checked original TSRX files"]
fn original_request_costs() {
    alloc::mark_installed();
    let linter = Linter::with_preset(LintPreset::Opinionated);
    each_file(|path, source| measure(&linter, path.to_str().unwrap(), source));
    for roots in [1, 8, 64] {
        let mut source = vize_l0::String::default();
        for i in 0..roots {
            use core::fmt::Write;
            writeln!(source, "export function List{i}({{ items }}: {{ items: string[] }}) @{{ <ul>{{items.map(x => <li><img src={{x}}/></li>)}}</ul> }}").unwrap();
        }
        measure(&linter, &format!("generated-{roots}.tsrx"), &source);
    }
}

fn measure(linter: &Linter, filename: &str, source: &str) {
    let run = |shared| {
        if shared {
            retained(black_box(linter), black_box(source), black_box(filename))
        } else {
            original(black_box(linter), black_box(source), black_box(filename))
        }
    };
    same(&run(false), &run(true), filename);
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    assert!(
        new.calls <= old.calls,
        "{filename}: allocation calls {old:?} → {new:?}"
    );
    assert!(
        new.peak_bytes_over_start <= old.peak_bytes_over_start,
        "{filename}: peak bytes {old:?} → {new:?}"
    );
    let time = |shared| {
        let start = Instant::now();
        for _ in 0..8 {
            drop(black_box(run(shared)));
        }
        start.elapsed().as_nanos() / 8
    };
    let mut pairs = Vec::new();
    for i in 0..35 {
        let reversed = i % 2 == 1;
        let a = time(reversed);
        let b = time(!reversed);
        if i >= 4 {
            pairs.push(if reversed { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "tsrx_request {filename} calls={}/{} peak={}/{} ns={pairs:?}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
}
