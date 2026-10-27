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
    // Both setup-only and load controls must materialize the identical initialized table.
    black_box(&next);
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
    let mut child_pid = None;
    let mut child_exit = None;
    let mut workers = Vec::new();
    let checksum = match mode.as_str() {
        "baseline" => black_box(42),
        "instructions" => instruction_work(iterations),
        "branches" => branch_work(iterations),
        "cache-small" | "cache-large" => cache_work(iterations, bytes),
        "cache-setup-small" | "cache-setup-large" => cache_work(0, bytes),
        "threads" => {
            let first = std::thread::spawn(move || instruction_work(iterations));
            let second = std::thread::spawn(move || instruction_work(iterations));
            workers.extend([first.join().unwrap(), second.join().unwrap()]);
            workers[0].wrapping_add(workers[1])
        }
        "child" => {
            let process = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["instructions", &iterations.to_string(), &bytes.to_string()])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            child_pid = Some(process.id());
            let output = process.wait_with_output().unwrap();
            child_exit = output.status.code();
            assert!(output.status.success());
            eprintln!(
                "child_packet={}",
                std::str::from_utf8(&output.stdout).unwrap().trim()
            );
            output
                .stdout
                .iter()
                .fold(0_u64, |sum, byte| sum.wrapping_add(u64::from(*byte)))
        }
        _ => panic!("unknown control"),
    };
    let dependent_loads = match mode.as_str() {
        "cache-small" | "cache-large" => iterations.to_string(),
        "cache-setup-small" | "cache-setup-large" => "0".to_owned(),
        _ => "null".to_owned(),
    };
    let child_pid = child_pid.map_or_else(|| "null".to_owned(), |pid| pid.to_string());
    let child_exit = child_exit.map_or_else(|| "null".to_owned(), |exit| exit.to_string());
    let worker_checksums = workers
        .iter()
        .map(|word| format!("\"{word:016x}\""))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"schema\":\"vize-pmu-calibration\",\"mode\":\"{mode}\",\"iterations\":{iterations},\"bytes\":{bytes},\"checksum\":\"{checksum:016x}\",\"completed\":true,\"dependentLoads\":{dependent_loads},\"childPid\":{child_pid},\"childExit\":{child_exit},\"workerChecksums\":[{worker_checksums}]}}"
    );
}
