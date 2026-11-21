use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn compare(lint: &ScriptLinter, block: SourceBlock<'_>, label: &str) {
    let grammar = lint.authored_source_type().unwrap();
    let request = |shared| {
        let arena = Allocator::default();
        let syntax = AuthoredProgram::parse(&arena, black_box(block), grammar);
        // Both requests keep the actual original parser result for the other reader.
        black_box(syntax.parsed());
        if shared {
            black_box(lint.lint_authored(syntax).unwrap());
        } else {
            black_box(lint.lint(block.source(), block.start() as usize));
            drop(black_box(syntax));
        }
    };
    let before = alloc::measure(|| request(false)).unwrap();
    let after = alloc::measure(|| request(true)).unwrap();
    let time = |shared| {
        let start = Instant::now();
        for _ in 0..8 {
            request(shared);
        }
        start.elapsed().as_nanos() / 8
    };
    let mut pairs = Vec::new();
    for i in 0..35 {
        let first = i % 2 != 0;
        let a = time(first);
        let b = time(!first);
        if i >= 4 {
            pairs.push(if first { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "authored_lint_time {label} calls={}/{} peak={}/{} ns={pairs:?}",
        before.calls, after.calls, before.peak_bytes_over_start, after.peak_bytes_over_start
    );
    let (mut old, mut new): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    old.sort_unstable();
    new.sort_unstable();
    assert!(new[15] <= old[15], "{label}: median");
    assert!(after.calls <= before.calls, "{label}: allocations");
    assert!(
        after.peak_bytes_over_start <= before.peak_bytes_over_start,
        "{label}: peak"
    );
}

#[test]
#[ignore = "paired requests on original and controlled scripts"]
fn original_and_shared_lint_requests() {
    alloc::mark_installed();
    let lint = ScriptLinter::with_vapor_rules();
    for (name, source) in [
        ("clean", "export const ok = 1;"),
        (
            "diagnostics",
            "import { nextTick as tick } from 'vue'; tick();",
        ),
        ("recovered", "const broken: = 1; nextTick();"),
    ] {
        compare(&lint, SourceRoot::new(source).unwrap().whole_block(), name);
    }
    for_originals(|index, name, source| {
        if index % 89 == 0 {
            scripts(source, |block| compare(&lint, block, name));
        }
    });
}
