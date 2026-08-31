#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde_json = "1"
//! ```

use serde_json::Value;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, PartialEq)]
struct Row {
    label: String,
    covered: u64,
    total: u64,
    percent: f64,
    minimum: f64,
    passed: bool,
}

struct Threshold {
    metric: String,
    minimum: f64,
}

struct Args {
    json_path: String,
    markdown_path: String,
    thresholds: Vec<Threshold>,
}

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = match parse_args(&args) {
        Ok(args) => args,
        Err(message) => {
            println!("{message}");
            return 1;
        }
    };

    let resolved = resolve_path(&args.json_path);
    let report = match read_json(&resolved) {
        Ok(report) => report,
        Err(()) => {
            println!("Failed to read or parse {}", resolved.display());
            return 1;
        }
    };
    let totals = match report_totals(&report) {
        Some(totals) => totals,
        None => {
            println!(
                "{} is not a cargo llvm-cov summary JSON report",
                resolved.display()
            );
            return 1;
        }
    };

    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for threshold in args.thresholds {
        let Some(percent) = metric_percent(totals, &threshold.metric) else {
            println!(
                "cargo llvm-cov report is missing {}.percent",
                threshold.metric
            );
            return 1;
        };
        let total = metric_count(totals, &threshold.metric, "count");
        let covered = metric_count(totals, &threshold.metric, "covered");
        let passed = total > 0 && percent >= threshold.minimum;
        let label = metric_label(&threshold.metric).to_string();
        rows.push(Row {
            label: label.clone(),
            covered,
            total,
            percent,
            minimum: threshold.minimum,
            passed,
        });
        if !passed {
            failures.push(format!(
                "{label} coverage {} < {}",
                format_percent(percent),
                format_percent(threshold.minimum)
            ));
        }
    }

    let markdown = render_markdown(&rows);
    print!("{markdown}");

    if !args.markdown_path.is_empty() && append_summary(&args.markdown_path, &markdown).is_err() {
        println!("Failed to append summary to {}", args.markdown_path);
        return 1;
    }

    if !failures.is_empty() {
        println!();
        println!("Rust source coverage budget failed:");
        for failure in failures {
            println!("{failure}");
        }
        return 1;
    }

    0
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut json_path = String::new();
    let mut markdown_path = env::var("GITHUB_STEP_SUMMARY").unwrap_or_default();
    let mut thresholds = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--json requires a value".to_string());
                };
                json_path = value.clone();
                index += 2;
            }
            "--markdown" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--markdown requires a value".to_string());
                };
                markdown_path = value.clone();
                index += 2;
            }
            flag if flag.starts_with("--min-") => {
                let Some(value) = args.get(index + 1) else {
                    return Err(format!("{flag} requires a value"));
                };
                let metric = flag
                    .strip_prefix("--min-")
                    .expect("checked min prefix")
                    .to_string();
                let minimum = value
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid minimum for {metric}: {value}"))?;
                if !minimum.is_finite() {
                    return Err(format!("Invalid minimum for {metric}: {value}"));
                }
                thresholds.push(Threshold { metric, minimum });
                index += 2;
            }
            arg => return Err(format!("Unknown argument: {arg}")),
        }
    }

    if json_path.is_empty() {
        return Err(usage());
    }

    Ok(Args {
        json_path,
        markdown_path,
        thresholds,
    })
}

fn usage() -> String {
    "Usage: rust-script tools/commands/ci/source-coverage.rs --json <path> [--markdown <path>] [--min-<metric> N]".to_string()
}

fn resolve_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn read_json(path: &Path) -> Result<Value, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    serde_json::from_str(&content).map_err(|_| ())
}

fn report_totals(report: &Value) -> Option<&serde_json::Map<String, Value>> {
    report
        .get("data")?
        .as_array()?
        .first()?
        .get("totals")?
        .as_object()
}

fn metric_percent(totals: &serde_json::Map<String, Value>, metric: &str) -> Option<f64> {
    totals.get(metric)?.get("percent")?.as_f64()
}

fn metric_count(totals: &serde_json::Map<String, Value>, metric: &str, field: &str) -> u64 {
    let Some(value) = totals.get(metric).and_then(|entry| entry.get(field)) else {
        return 0;
    };
    value
        .as_u64()
        .or_else(|| value.as_f64().map(|value| value as u64))
        .unwrap_or(0)
}

fn metric_label(metric: &str) -> &str {
    match metric {
        "branches" => "Branches",
        "functions" => "Functions",
        "lines" => "Lines",
        "regions" => "Regions",
        _ => metric,
    }
}

fn format_percent(value: f64) -> String {
    let scaled = (value * 100.0 + 0.5).floor() as u64;
    let whole = scaled / 100;
    let fraction = scaled % 100;
    format!("{whole}.{fraction:02}%")
}

fn render_markdown(rows: &[Row]) -> String {
    let mut lines = vec![
        "## Rust Source Coverage".to_string(),
        String::new(),
        "| Metric | Covered | Total | Percent | Minimum | Status |".to_string(),
        "| --- | ---: | ---: | ---: | ---: | --- |".to_string(),
    ];

    for row in rows {
        let status = if row.passed { "pass" } else { "fail" };
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {status} |",
            row.label,
            row.covered,
            row.total,
            format_percent(row.percent),
            format_percent(row.minimum),
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn append_summary(path: &str, markdown: &str) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(markdown.as_bytes())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_threshold_rows() {
        let markdown = render_markdown(&[Row {
            label: "Regions".to_string(),
            covered: 333,
            total: 500,
            percent: 66.66666666666667,
            minimum: 60.0,
            passed: true,
        }]);

        assert_eq!(
            markdown,
            "## Rust Source Coverage\n\n| Metric | Covered | Total | Percent | Minimum | Status |\n| --- | ---: | ---: | ---: | ---: | --- |\n| Regions | 333 | 500 | 66.67% | 60.00% | pass |\n"
        );
    }

    #[test]
    fn preserves_threshold_order() {
        let args = parse_args(&[
            "--json".to_string(),
            "summary.json".to_string(),
            "--min-lines".to_string(),
            "70".to_string(),
            "--min-branches".to_string(),
            "40".to_string(),
        ])
        .unwrap();

        assert_eq!(args.thresholds[0].metric, "lines");
        assert_eq!(args.thresholds[1].metric, "branches");
    }
}
