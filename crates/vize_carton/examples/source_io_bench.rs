//! Paired UTF-8 validation and file-read measurements, without parser work.

use std::{hint::black_box, io, time::Instant};
use vize_carton::source_io::{decode_utf8, read_to_string};

fn median(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples.get(samples.len() / 2).copied().unwrap_or_default()
}

fn measure(
    mut operation: impl FnMut() -> io::Result<usize>,
    iterations: usize,
) -> io::Result<u128> {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(operation()?);
    }
    Ok(start.elapsed().as_nanos() / iterations.max(1) as u128)
}

fn paired(
    mut standard: impl FnMut() -> io::Result<usize>,
    mut candidate: impl FnMut() -> io::Result<usize>,
    iterations: usize,
) -> io::Result<serde_json::Value> {
    for _ in 0..2 {
        black_box(standard()?);
        black_box(candidate()?);
    }
    let mut baseline = Vec::new();
    let mut head = Vec::new();
    for pair in 0..9 {
        if pair % 2 == 0 {
            baseline.push(measure(&mut standard, iterations)?);
            head.push(measure(&mut candidate, iterations)?);
        } else {
            head.push(measure(&mut candidate, iterations)?);
            baseline.push(measure(&mut standard, iterations)?);
        }
    }
    let baseline_samples = baseline.clone();
    let head_samples = head.clone();
    let standard_ns = median(&mut baseline);
    let candidate_ns = median(&mut head);
    Ok(serde_json::json!({
        "iterations": iterations,
        "standard_ns": standard_ns,
        "candidate_ns": candidate_ns,
        "ratio": candidate_ns as f64 / standard_ns.max(1) as f64,
        "standard_samples_ns": baseline_samples,
        "candidate_samples_ns": head_samples,
    }))
}

fn main() -> io::Result<()> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "source-io-bench.json".into());
    let directory = tempfile::tempdir()?;
    let mut rows = Vec::new();
    for size in [32, 1024, 8192, 65536, 1048576] {
        for (kind, pattern) in [
            ("ascii", "<template><div>Hello world</div></template>\n"),
            ("unicode", "<template><div>日本語🦀é</div></template>\n"),
        ] {
            let mut text = pattern.repeat(size / pattern.len() + 1);
            let mut end = size;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            let path = directory
                .path()
                .join(vize_carton::cstr!("{kind}-{size}.vue").as_str());
            std::fs::write(&path, &text)?;
            if decode_utf8(text.as_bytes()).map_err(io::Error::other)? != text
                || read_to_string(&path)? != std::fs::read_to_string(&path)?
            {
                return Err(io::Error::other("candidate changed source bytes"));
            }
            let iterations = (8 * 1024 * 1024 / size).clamp(16, 4096);
            let validation = paired(
                || {
                    core::str::from_utf8(black_box(text.as_bytes()))
                        .map(str::len)
                        .map_err(io::Error::other)
                },
                || {
                    decode_utf8(black_box(text.as_bytes()))
                        .map(str::len)
                        .map_err(io::Error::other)
                },
                iterations,
            )?;
            let file_read = paired(
                || std::fs::read_to_string(black_box(&path)).map(|text| text.len()),
                || read_to_string(black_box(&path)).map(|text| text.len()),
                iterations.min(512),
            )?;
            println!(
                "{kind}/{size}: validation {} file-read {}",
                validation.get("ratio").unwrap_or(&serde_json::Value::Null),
                file_read.get("ratio").unwrap_or(&serde_json::Value::Null)
            );
            rows.push(serde_json::json!({ "kind": kind, "requested_bytes": size,
                "actual_bytes": text.len(), "validation": validation, "file_read": file_read }));
        }
    }
    let report = serde_json::json!({
        "schema": 1,
        "head_sha": std::env::var("HEAD_SHA").unwrap_or_default(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "pairs": 9,
        "warmups": 2,
        "rows": rows,
    });
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)
}
