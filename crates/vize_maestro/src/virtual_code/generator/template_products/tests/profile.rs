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
        let time = |joined, iterations| {
            let start = Instant::now();
            for _ in 0..iterations {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / iterations
        };
        let sample = |iterations, reverse| {
            let mut pairs = Vec::with_capacity(31);
            for round in 0..35 {
                let first = (round % 2 == 0) != reverse;
                let a = time(first, iterations);
                let b = time(!first, iterations);
                if round >= 4 {
                    pairs.push(if first { (b, a) } else { (a, b) });
                }
            }
            pairs
        };
        let pairs = sample(8, false);
        eprintln!(
            "template_product_time repeats={repeats} calls={}/{} peak={}/{} ns={pairs:?}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        if consistent_slowdown(&pairs) {
            let confirmation = sample(64, true);
            eprintln!("template_product_confirmation repeats={repeats} ns={confirmation:?}");
            assert!(
                !consistent_slowdown(&confirmation),
                "confirmed slowdown, repeats={repeats}"
            );
        }
        assert!(
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
            "repeats={repeats}"
        );
    }
}

// A one-sided paired sign test: under an equal-cost null, at least 23 slower
// observations out of 31 has probability below 0.006. Ties count toward neither
// a regression nor a repeat. Confirmation reverses order and uses longer windows.
fn consistent_slowdown(pairs: &[(u128, u128)]) -> bool {
    assert_eq!(pairs.len(), 31);
    pairs.iter().filter(|(old, new)| new > old).count() >= 23
}

#[test]
fn time_gate_distinguishes_scheduling_noise_from_repeated_small_slowdowns() {
    let mut pairs = [(100, 100); 31];
    pairs[..15].fill((100, 1_000));
    assert!(!consistent_slowdown(&pairs));
    pairs[..22].fill((100, 101));
    assert!(!consistent_slowdown(&pairs));
    pairs[22] = (100, 101);
    assert!(consistent_slowdown(&pairs));
}
