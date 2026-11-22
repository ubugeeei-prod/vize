use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "compare all retained checker and editor outputs at each expression density"]
fn paired_template_tool_products() {
    alloc::mark_installed();
    for repeats in [1, 8, 32, 128] {
        let content = "<div :title='a'>{{ a + b }}</div>".repeat(repeats);
        let source =
            format!("<script setup>const a=1;const b=2;</script><template>{content}</template>");
        let arena = Allocator::default();
        let owner = observe(&arena, &source);
        let native = owner.admitted().unwrap();
        let descriptor = fixture_descriptor(&source);
        compare(&native, &descriptor, "repeated elements");
        let run = |joined| {
            if joined {
                VirtualCodeGenerator::new()
                    .template_operand_products(black_box(&native), black_box(&descriptor), URI)
                    .unwrap()
            } else {
                separate(black_box(&native), black_box(&descriptor))
            }
        };
        let old = alloc::measure(|| run(false)).unwrap();
        let new = alloc::measure(|| run(true)).unwrap();
        let time = |joined| {
            let start = Instant::now();
            for _ in 0..8 {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / 8
        };
        let mut pairs = Vec::new();
        for round in 0..35 {
            let first = round % 2 == 0;
            let a = time(first);
            let b = time(!first);
            if round >= 4 {
                pairs.push(if first { (b, a) } else { (a, b) });
            }
        }
        eprintln!(
            "template_product_time repeats={repeats} calls={}/{} peak={}/{} ns={pairs:?}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        let (mut old_ns, mut new_ns): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        old_ns.sort_unstable();
        new_ns.sort_unstable();
        assert!(new_ns[15] <= old_ns[15], "repeats={repeats}");
        assert!(
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
            "repeats={repeats}"
        );
    }
}
