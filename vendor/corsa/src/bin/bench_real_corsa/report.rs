use std::{fs, path::Path};

use corsa::Result;
use serde_json::json;

use crate::{args::Cli, dataset::DatasetCase, scenario::ScenarioRow};

pub fn write(path: &Path, cli: &Cli, datasets: &[DatasetCase], rows: &[ScenarioRow]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let datasets = datasets
        .iter()
        .map(|dataset| {
            json!({
                "label": dataset.label.as_str(),
                "configPath": dataset.config_path,
                "fileCount": dataset.file_count,
                "totalBytes": dataset.total_bytes,
                "totalLines": dataset.total_lines,
                "primaryFile": dataset.primary_file.as_str(),
            })
        })
        .collect::<Vec<_>>();
    let rows_json = rows
        .iter()
        .map(|row| {
            json!({
                "mode": row.mode.as_str(),
                "dataset": row.dataset.as_str(),
                "scenario": row.scenario.as_str(),
                "sampleCount": row.stats.sample_count(),
                "medianMs": row.stats.median_ms(),
                "p95Ms": row.stats.p95_ms(),
                "p99Ms": row.stats.p99_ms(),
                "meanMs": row.stats.mean_ms(),
                "stddevMs": row.stats.stddev_ms(),
                "cvPercent": row.stats.cv_percent(),
                "minMs": row.stats.min_ms(),
                "maxMs": row.stats.max_ms(),
                "profile": row.profile.iter().map(|profile| {
                    json!({
                        "method": profile.method.as_str(),
                        "phase": profile.phase.as_str(),
                        "sampleCount": profile.stats.sample_count(),
                        "medianMs": profile.stats.median_ms(),
                        "p95Ms": profile.stats.p95_ms(),
                        "meanMs": profile.stats.mean_ms(),
                        "minMs": profile.stats.min_ms(),
                        "maxMs": profile.stats.max_ms(),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "corsaPath": cli.corsa_path,
            "runMode": match cli.run_mode {
                crate::args::RunMode::Benchmark => "benchmark",
                crate::args::RunMode::Profiling => "profiling",
            },
            "coldIterations": cli.cold_iterations,
            "warmIterations": cli.warm_iterations,
            "datasets": datasets,
            "rows": rows_json,
            "comparisons": comparison_json(rows),
        }))?,
    )?;
    Ok(())
}

pub fn comparison_lines(rows: &[ScenarioRow]) -> Vec<String> {
    let mut lines = Vec::new();
    for comparison in comparisons(rows) {
        lines.push(format!(
            "{}\t{}\t{:.3}\t{:.3}\t{:.2}\t{:.2}",
            comparison.dataset,
            comparison.scenario,
            comparison.msgpack_median_ms,
            comparison.jsonrpc_median_ms,
            comparison.speedup_x,
            comparison.p95_ratio
        ));
    }
    lines
}

fn comparison_json(rows: &[ScenarioRow]) -> Vec<serde_json::Value> {
    comparisons(rows)
        .into_iter()
        .map(|comparison| {
            json!({
                "dataset": comparison.dataset,
                "scenario": comparison.scenario,
                "msgpackMedianMs": comparison.msgpack_median_ms,
                "jsonrpcMedianMs": comparison.jsonrpc_median_ms,
                "speedupX": comparison.speedup_x,
                "p95Ratio": comparison.p95_ratio,
            })
        })
        .collect()
}

struct Comparison<'a> {
    dataset: &'a str,
    scenario: &'a str,
    msgpack_median_ms: f64,
    jsonrpc_median_ms: f64,
    speedup_x: f64,
    p95_ratio: f64,
}

fn comparisons(rows: &[ScenarioRow]) -> Vec<Comparison<'_>> {
    let mut items = Vec::new();
    for msgpack in rows.iter().filter(|row| row.mode.as_str() == "msgpack") {
        let Some(jsonrpc) = rows.iter().find(|row| {
            row.mode.as_str() == "jsonrpc"
                && row.dataset == msgpack.dataset
                && row.scenario == msgpack.scenario
        }) else {
            continue;
        };
        let msgpack_median_ms = msgpack.stats.median_ms();
        let jsonrpc_median_ms = jsonrpc.stats.median_ms();
        items.push(Comparison {
            dataset: msgpack.dataset.as_str(),
            scenario: msgpack.scenario.as_str(),
            msgpack_median_ms,
            jsonrpc_median_ms,
            speedup_x: jsonrpc_median_ms / msgpack_median_ms,
            p95_ratio: jsonrpc.stats.p95_ms() / msgpack.stats.p95_ms(),
        });
    }
    items.sort_by(|left, right| {
        left.dataset
            .cmp(right.dataset)
            .then_with(|| left.scenario.cmp(right.scenario))
    });
    items
}
