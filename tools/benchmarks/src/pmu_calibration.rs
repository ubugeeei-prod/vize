//! Original single-process controls for hardware-counter admission, not Vize speed claims.
use std::hint::black_box;

#[inline(never)]
fn instruction_work(iterations: u64) -> u64 {
    let mut word = 0x1234_5678_9abc_def0_u64;
    for index in 0..iterations {
        for _ in 0..32 {
            word = black_box(word)
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(index);
            word ^= word.rotate_left(17);
        }
    }
    black_box(word)
}

#[inline(never)]
fn even(word: u64) -> u64 {
    black_box(word.rotate_left(17).wrapping_add(0x57a9))
}

#[inline(never)]
fn odd(word: u64) -> u64 {
    black_box(word.rotate_right(11).wrapping_mul(0x796d))
}

#[inline(never)]
fn branch_work(iterations: u64) -> u64 {
    let mut word = 0x1234_5678_9abc_def0_u64;
    for _ in 0..iterations {
        word ^= word << 13;
        word ^= word >> 7;
        word ^= word << 17;
        word = if black_box(word) & 1 == 0 {
            even(word)
        } else {
            odd(word)
        };
    }
    black_box(word)
}

#[inline(never)]
fn cache_work(iterations: u64, bytes: usize) -> u64 {
    let words = bytes / std::mem::size_of::<usize>();
    assert!(words.is_power_of_two());
    // An odd increment and a 1-mod-4 multiplier form a full-period permutation
    // of a power-of-two working set. Dependent reads prevent vectorized loads.
    let next: Vec<usize> = (0..words)
        .map(|index| index.wrapping_mul(1_664_525).wrapping_add(1_013_904_223) & (words - 1))
        .collect();
    let mut index = 0;
    for _ in 0..iterations {
        index = black_box(next[black_box(index)]);
    }
    black_box(index as u64)
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let mode = arguments.next().expect("mode is required");
    let iterations: u64 = arguments
        .next()
        .expect("iterations are required")
        .parse()
        .expect("iterations must be an integer");
    let bytes: usize = arguments
        .next()
        .expect("working-set bytes are required")
        .parse()
        .expect("bytes must be an integer");
    assert!(arguments.next().is_none());
    assert!((1..=100_000_000).contains(&iterations));
    assert!(bytes >= 64 && bytes.is_power_of_two());
    let checksum = match mode.as_str() {
        "baseline" => black_box(42),
        "instructions" => instruction_work(iterations),
        "branches" => branch_work(iterations),
        "cache-small" | "cache-large" => cache_work(iterations, bytes),
        _ => panic!("unknown control"),
    };
    println!(
        "{{\"schema\":\"vize-pmu-calibration\",\"mode\":\"{mode}\",\"iterations\":{iterations},\"bytes\":{bytes},\"checksum\":\"{checksum:016x}\",\"completed\":true}}"
    );
}
