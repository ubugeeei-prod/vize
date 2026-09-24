//! Local, production-entry profiling probe. Run against the PR benchmark corpus.
//!
//! `cargo run --profile ci-opt -p vize_patina --example profile_sfc -- <directory> 100`
//! enables wall timing; set `VIZE_LINT_PROFILE=1` for per-pass attribution.
use std::{fs, hint::black_box, time::Instant};
use vize_patina::Linter;
use vize_s0::profiler::global_profiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args()
        .nth(1)
        .ok_or("usage: profile_sfc <fixture directory> [iterations]")?;
    let iterations: usize = std::env::args().nth(2).unwrap_or("20".into()).parse()?;
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "vue") {
            paths.push(path);
        }
    }
    paths.sort();
    let inputs = paths
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<_>, _>>()?;
    let linter = Linter::new();
    if std::env::var_os("VIZE_LINT_PROFILE").is_some() {
        global_profiler().enable();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        for (path, source) in paths.iter().zip(&inputs) {
            black_box(linter.lint_sfc(source, &path.to_string_lossy()));
        }
    }
    let elapsed = start.elapsed();
    global_profiler().disable();
    eprintln!(
        "{} files in {:.3} ms",
        iterations * inputs.len(),
        elapsed.as_secs_f64() * 1000.0
    );
    for entry in global_profiler().summary().entries {
        eprintln!(
            "{:<48} {:>9.3} ms self {:>9.3} ms ({})",
            entry.name,
            entry.total.as_secs_f64() * 1000.0,
            entry.self_total.as_secs_f64() * 1000.0,
            entry.count
        );
    }
    Ok(())
}
