//! Alternate both arms against the same retained file and offset sequence.
use super::{ExpressionPositionIndex, Model, input::repeated, observe};
use std::{hint::black_box, time::Instant};
use vize_l0::Allocator;

fn time(rounds: usize, mut work: impl FnMut(usize) -> usize) -> u128 {
    let t = Instant::now();
    let mut checksum = 0;
    for i in 0..rounds {
        checksum += black_box(work(black_box(i)));
    }
    black_box(checksum);
    t.elapsed().as_nanos() / rounds as u128
}

fn pairs(
    label: &str,
    repeats: usize,
    rounds: usize,
    mut left: impl FnMut(usize) -> usize,
    mut right: impl FnMut(usize) -> usize,
) {
    let _warm = (time(rounds, &mut left), time(rounds, &mut right));
    let mut values = Vec::new();
    for order in 0..31 {
        values.push(if order % 2 == 0 {
            (time(rounds, &mut left), time(rounds, &mut right))
        } else {
            let new = time(rounds, &mut right);
            (time(rounds, &mut left), new)
        });
    }
    eprintln!("mode={label} repeats={repeats} pairs={values:?}");
}

#[test]
#[ignore = "run in an optimized build without concurrent work"]
fn paired_reference_queries() {
    for repeats in [1, 32, 128] {
        let arena = Allocator::default();
        let source = repeated(repeats);
        let observation = observe(&arena, &source);
        let native = observation.admitted().unwrap();
        let file = native.file().l2ir();
        let binding = file.bindings().next().unwrap();
        let model = Model::new(file);
        let index = ExpressionPositionIndex::new(file);
        let offsets: Vec<_> = source
            .match_indices("value")
            .skip(1)
            .map(|(i, _)| i as u32)
            .chain([source.len() as u32])
            .collect();
        let offset = |i: usize| offsets[i % offsets.len()];
        let old = |i| usize::from(native.reference_at_offset(offset(i)).unwrap().is_some());
        let scan = |i| {
            usize::from(
                model
                    .expression_reference_at_offset(offset(i))
                    .unwrap()
                    .is_some(),
            )
        };
        let lookup = |i| {
            usize::from(
                index
                    .expression_reference_at_offset(offset(i))
                    .unwrap()
                    .is_some(),
            )
        };
        let rounds = (65_536 / repeats).clamp(128, 4096);
        pairs("scan", repeats, rounds, old, scan);
        pairs("indexed", repeats, rounds, old, lookup);
        pairs("first", repeats, rounds, old, |i| {
            usize::from(
                ExpressionPositionIndex::new(file)
                    .expression_reference_at_offset(offset(i))
                    .unwrap()
                    .is_some(),
            )
        });
        pairs(
            "two",
            repeats,
            rounds,
            |i| old(i) + old(i + 1),
            |i| {
                let fresh = ExpressionPositionIndex::new(file);
                usize::from(
                    fresh
                        .expression_reference_at_offset(offset(i))
                        .unwrap()
                        .is_some(),
                ) + usize::from(
                    fresh
                        .expression_reference_at_offset(offset(i + 1))
                        .unwrap()
                        .is_some(),
                )
            },
        );
        pairs(
            "references",
            repeats,
            rounds,
            |_| {
                let mut total = 0;
                native
                    .for_each_reference_to(binding, |s| {
                        total += usize::from(s.template().is_some())
                    })
                    .unwrap();
                total
            },
            |_| {
                let mut total = 0;
                model
                    .for_each_expression_reference_to(binding, |_| total += 1)
                    .unwrap();
                total
            },
        );
    }
}
