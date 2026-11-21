use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{alloc::System, hint::black_box, time::Instant};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

pub(super) fn paired(owner: &NativeSfcObservation<'_>, source: usize) {
    alloc::mark_installed();
    let prepare = || StyledVueInput::prepare(black_box(owner).admitted().unwrap()).unwrap();
    let run = |shared| {
        let input = prepare();
        let on = input.project().unwrap();
        let off = if shared {
            input.project_no_links().unwrap()
        } else {
            prepare().project_no_links().unwrap()
        };
        black_box((on, off))
    };
    let (a, b) = (run(false), run(true));
    assert_eq!(a.0.document(), b.0.document());
    assert_eq!(a.1.document(), b.1.document());
    drop((a, b));
    let prepare_memory = alloc::measure(prepare).unwrap();
    assert_eq!(prepare_memory.calls, 0);
    let a = alloc::measure(|| run(false)).unwrap();
    let b = alloc::measure(|| run(true)).unwrap();
    assert!(b.calls <= a.calls && b.peak_bytes_over_start <= a.peak_bytes_over_start);
    let time = |shared| {
        let start = Instant::now();
        for _ in 0..32 {
            drop(run(shared));
        }
        start.elapsed().as_nanos() / 32
    };
    let mut pairs = Vec::new();
    for i in 0..35 {
        let rev = i % 2 != 0;
        let x = time(rev);
        let y = time(!rev);
        if i >= 4 {
            pairs.push(if rev { (y, x) } else { (x, y) });
        }
    }
    eprintln!(
        "styled_doc={source} input={} calls={}/{} peak={}/{} pairs={pairs:?}",
        owner.descriptor().source().len(),
        a.calls,
        b.calls,
        a.peak_bytes_over_start,
        b.peak_bytes_over_start
    );
}

#[test]
#[ignore = "paired preparation, emission and allocation on original styled sources"]
fn style_sizes() {
    for (blocks, rows) in [(1, 0), (1, 1), (4, 4), (1, 128), (4, 128), (1, 2048)] {
        let mut source = String::from(
            "<script setup>const title = '🎨';</script><template>{{title}}</template>",
        );
        for _ in 0..blocks {
            source.push_str("<style>");
            for _ in 0..rows {
                source.push_str(".section { color: rebeccapurple; }\n");
            }
            source.push_str("</style>");
        }
        let arena = Allocator::default();
        let obs = lower_sfc_native(&arena, &source, options());
        paired(&obs, rows);
    }
}
