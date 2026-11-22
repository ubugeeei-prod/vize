use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired complete component props and original import destinations, cold and retained"]
fn two_consumers() {
    super::super::shorthand_products::request_modes(|label, cold, run| {
        let sample = |joined| {
            let start = Instant::now();
            for _ in 0..128 {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / 128
        };
        let (mut old, mut new) = (Vec::new(), Vec::new());
        for i in 0..35 {
            let first = i % 2 == 0;
            let a = sample(first);
            let b = sample(!first);
            if i >= 4 {
                let (before, after) = if first { (b, a) } else { (a, b) };
                old.push(before);
                new.push(after);
            }
        }
        old.sort_unstable();
        new.sort_unstable();
        eprintln!(
            "shorthand_two_consumers case={label} cold={cold} separate_ns={} shared_ns={}",
            old[15], new[15]
        );
        assert!(new[15] <= old[15]);
    });
}
