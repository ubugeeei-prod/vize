//! Instrumenter perturbation control, not an end-user performance benchmark.

use std::hint::black_box;
use std::time::Instant;

fn main() {
    let args: vize_l0::SmallVec<[_; 4]> = std::env::args().map(vize_l0::String::from).collect();
    let enabled = args.get(1).is_some_and(|arg| arg == "enabled");
    let workers = args
        .get(2)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(8);
    let iterations = args
        .get(3)
        .and_then(|arg| arg.parse::<u64>().ok())
        .unwrap_or(500_000);
    let profiler = vize_l0::profiler::global_profiler();
    profiler.clear();
    if enabled {
        profiler.enable();
    }
    let start = Instant::now();
    let checksum: u64 = std::thread::scope(|scope| {
        let handles: vize_l0::SmallVec<[_; 8]> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut total = 0_u64;
                    for index in 0..iterations {
                        let value = black_box(index.wrapping_mul(3));
                        vize_l0::micro_count!("control.synthetic.units", value);
                        total = total.wrapping_add(value);
                    }
                    total
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(value) => value,
                Err(_) => std::process::exit(1),
            })
            .fold(0_u64, u64::wrapping_add)
    });
    let wall_ns = start.elapsed().as_nanos();
    profiler.disable();
    #[cfg(feature = "micro-profiling")]
    let counters = {
        let counters = vize_l0::profiler::micro::snapshot();
        if enabled {
            assert_eq!(counters.len(), 1);
            assert_eq!(
                counters.first().map(|entry| entry.calls),
                Some(workers as u64 * iterations)
            );
            assert_eq!(
                counters.first().map(|entry| entry.units),
                Some(workers as u64 * 3 * iterations * (iterations - 1) / 2)
            );
        } else {
            assert!(counters.is_empty());
        }
        counters
    };
    #[cfg(not(feature = "micro-profiling"))]
    let counters: [u64; 0] = [];
    println!(
        "{}",
        serde_json::json!({
            "metric": "synthetic-instrumenter-overhead", "feature": cfg!(feature = "micro-profiling"),
            "enabled": enabled, "workers": workers, "iterationsPerWorker": iterations,
            "wallNs": wall_ns, "checksum": checksum, "counters": counters,
            "scope": "Rust worker setup and loop; not product speed or allocator overhead",
        })
    );
}
