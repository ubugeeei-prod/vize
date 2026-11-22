use super::*;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired cold and retained products for the compiler and project effect reader"]
fn paired_script_products() {
    alloc::mark_installed();
    for size in [0, 1, 8, 32, 128] {
        let mut source = String::from("<script lang='ts'>import { ref,computed } from 'vue';");
        for n in 0..size {
            source.push_str(&format!("export const x{n}=ref({n});"));
        }
        source.push_str("</script><script setup lang='ts'>");
        for n in 0..size {
            source.push_str(&format!(
                "const {{a:[v{n}]}}=obj; const y{n}=computed(()=>x{n}.value);"
            ));
        }
        source.push_str("</script>");
        let descriptor = crate::parse_sfc(&source, Default::default()).unwrap();
        let blocks = blocks(&source, &descriptor);
        let arena = Allocator::new();
        let parsed = parse(&arena, blocks);
        compare(
            &read(&descriptor, &parsed).unwrap(),
            &original(blocks, &parsed),
            "paired",
        );
        for cold in [false, true] {
            assert!(measure(
                &descriptor,
                blocks,
                &parsed,
                cold,
                &size.to_string(),
                8,
                true
            ));
        }
    }
}

pub(super) fn measure(
    descriptor: &SfcDescriptor<'_>,
    blocks: [Option<Block<'_>>; 2],
    parsed: &[Option<AuthoredProgram<'_>>; 2],
    cold: bool,
    label: &str,
    iterations: u128,
    show_pairs: bool,
) -> bool {
    let project = |joined, prepared: &[Option<AuthoredProgram<'_>>; 2]| {
        if joined {
            read(black_box(descriptor), black_box(prepared)).unwrap()
        } else {
            original(black_box(blocks), black_box(prepared))
        }
    };
    let run = |joined| {
        if cold {
            let fresh = Allocator::new();
            project(joined, &parse(&fresh, black_box(blocks)))
        } else {
            project(joined, parsed)
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
    let pairs = sample(iterations, false);
    let slower = |pairs: &[(u128, u128)]| pairs.iter().filter(|(old, new)| new > old).count() >= 23;
    let mut passes =
        new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start;
    if slower(&pairs) {
        let confirmation = sample(iterations * 8, true);
        eprintln!("script_effect_confirmation input={label} cold={cold} ns={confirmation:?}");
        passes &= !slower(&confirmation);
    }
    if show_pairs || !passes {
        eprintln!("script_effect_samples input={label} cold={cold} ns={pairs:?}");
    }
    let (mut a, mut b): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    a.sort_unstable();
    b.sort_unstable();
    eprintln!(
        "script_effect_time input={label} cold={cold} ns={}/{} calls={}/{} peak={}/{} pass={passes}",
        a[15], b[15], old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    passes
}
