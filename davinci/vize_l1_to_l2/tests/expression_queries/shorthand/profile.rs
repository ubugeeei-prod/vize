use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired complete component props and original import destinations, cold and retained"]
fn two_consumers() {
    super::super::shorthand_products::request_modes(|label, cold, run| {
        let sample = |joined, iterations| {
            let start = Instant::now();
            for _ in 0..iterations {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / iterations
        };
        let measure = |iterations, reverse| {
            let (mut old, mut new) = (Vec::new(), Vec::new());
            let mut slower = 0;
            for i in 0..35 {
                let first = (i % 2 == 0) != reverse;
                let a = sample(first, iterations);
                let b = sample(!first, iterations);
                if i >= 4 {
                    let (before, after) = if first { (b, a) } else { (a, b) };
                    old.push(before);
                    new.push(after);
                    slower += usize::from(after > before);
                }
            }
            old.sort_unstable();
            new.sort_unstable();
            eprintln!(
                "shorthand_two_consumers case={label} cold={cold} separate_ns={} shared_ns={} slower={slower}/31",
                old[15], new[15]
            );
            slower >= 23
        };
        // Confirm a one-sided paired sign test with longer, reversed batches.
        assert!(
            !(measure(128, false) && measure(512, true)),
            "{label} cold={cold}"
        );
    });
}
