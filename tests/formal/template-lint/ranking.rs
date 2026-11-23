//! Model-check the same safe Rust insertion that publishes Patina's labels.
#[path = "../../../crates/vize_patina/src/rules/facts/max_template_complexity/original/window.rs"]
mod window;

#[kani::proof]
#[kani::unwind(8)]
fn every_displaced_label_is_the_reference_prefix_entry() {
    const CAPACITY: usize = 5;
    let length = kani::any::<usize>();
    kani::assume(length <= CAPACITY);
    let scores: [u8; CAPACITY] = kani::any();
    let score = kani::any::<u8>();
    let old: [Option<(u8, usize)>; CAPACITY] =
        core::array::from_fn(|i| (i < length).then_some((scores[i], i)));
    let new = (score, CAPACITY);
    let mut actual = old;
    window::insert(&mut actual, new, |a, b| a.0 > b.0);

    // The unlimited stable insertion is the old prefix, the new row, then
    // the untouched suffix. A full tied window keeps every older identity.
    let mut at = 0;
    while at < length && scores[at] >= score {
        at += 1;
    }
    for (i, result) in actual.iter().enumerate() {
        let expected = if i < at {
            old[i]
        } else if i == at {
            Some(new)
        } else {
            old[i - 1]
        };
        assert_eq!(*result, expected);
    }
}
