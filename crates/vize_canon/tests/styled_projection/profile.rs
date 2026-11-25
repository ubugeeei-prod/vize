//! Measure both projections in an isolated process with the counting allocator.
#![expect(
    clippy::unwrap_used,
    clippy::disallowed_types,
    reason = "the profiling executable checks original source files and prints complete paired measurements"
)]
mod input;
use davinci_harness::alloc::{self, CountingAllocator};
use input::options;
use std::{alloc::System, hint::black_box, time::Instant};
use std::{fs, path::Path};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_native};
use vize_l4::targets::ts::vue::StyledVueInput;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn paired(owner: &NativeSfcObservation<'_>, source: usize) {
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
        owner.l1ir().source().len(),
        a.calls,
        b.calls,
        a.peak_bytes_over_start,
        b.peak_bytes_over_start
    );
}

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

fn main() {
    if let Some(manifest) = std::env::var_os("VIZE_STYLED_CHECKER_INPUTS") {
        let paths = fs::read(manifest).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut count = 0;
        for (index, raw) in paths
            .split(|&b| b == 0)
            .filter(|p| !p.is_empty())
            .enumerate()
        {
            let file = fs::read(root.join(std::str::from_utf8(raw).unwrap())).unwrap();
            let Ok(source) = std::str::from_utf8(&file) else {
                continue;
            };
            let arena = Allocator::default();
            let observation = lower_sfc_native(&arena, source, options());
            let Some(owner) = observation.admitted() else {
                continue;
            };
            if observation.l1ir().admitted().unwrap().styles().len() > 0
                && StyledVueInput::prepare(owner)
                    .and_then(|v| v.project())
                    .is_ok()
            {
                paired(&observation, index + 1);
                count += 1;
            }
        }
        assert!(count > 0);
        eprintln!("profiled original styled files={count}");
    }
    style_sizes();
}
