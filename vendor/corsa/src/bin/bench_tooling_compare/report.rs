use std::{fs, path::Path};

use corsa::Result;
use serde_json::json;

use crate::{args::Cli, dataset::DatasetCase, runner::ToolRow};

pub fn write(path: &Path, cli: &Cli, datasets: &[DatasetCase], rows: &[ToolRow]) -> Result<()> {
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
                "workload": row.workload.as_str(),
                "dataset": row.dataset.as_str(),
                "tool": row.tool.as_str(),
                "sampleCount": row.stats.sample_count(),
                "medianMs": row.stats.median_ms(),
                "p95Ms": row.stats.p95_ms(),
                "p99Ms": row.stats.p99_ms(),
                "meanMs": row.stats.mean_ms(),
                "stddevMs": row.stats.stddev_ms(),
                "cvPercent": row.stats.cv_percent(),
                "minMs": row.stats.min_ms(),
                "maxMs": row.stats.max_ms(),
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "corsaPath": cli.corsa_path,
            "nodeCommand": cli.node_command.as_str(),
            "iterations": cli.iterations,
            "warmupIterations": cli.warmup_iterations,
            "timeoutMs": cli.timeout_ms,
            "datasets": datasets,
            "rows": rows_json,
            "projectCheckVsCorsa": project_check_vs_corsa_json(rows),
            "projectCheckRequestedOpponentComparison": project_check_requested_opponent_json(rows),
            "workflowVsCorsaCli": workflow_vs_corsa_json(rows),
        }))?,
    )?;
    Ok(())
}

pub fn project_check_vs_corsa_lines(rows: &[ToolRow]) -> Vec<String> {
    project_check_vs_corsa(rows)
        .into_iter()
        .map(|comparison| {
            format!(
                "{}\t{}\t{:.3}\t{:.3}\t{:.2}",
                comparison.dataset,
                comparison.tool,
                comparison.median_ms,
                comparison.baseline_median_ms,
                comparison.vs_baseline_x
            )
        })
        .collect()
}

pub fn workflow_vs_corsa_lines(rows: &[ToolRow]) -> Vec<String> {
    workflow_vs_corsa(rows)
        .into_iter()
        .map(|comparison| {
            format!(
                "{}\t{}\t{:.3}\t{:.3}\t{:.2}",
                comparison.dataset,
                comparison.tool,
                comparison.median_ms,
                comparison.baseline_median_ms,
                comparison.vs_baseline_x
            )
        })
        .collect()
}

pub fn project_check_requested_opponent_lines(rows: &[ToolRow]) -> Vec<String> {
    project_check_requested_opponents(rows)
        .into_iter()
        .map(|comparison| {
            format!(
                "{}\t{}\t{}\t{:.3}\t{:.3}\t{:.2}\t{}",
                comparison.dataset,
                comparison.tool,
                comparison.opponent_tool,
                comparison.tool_median_ms,
                comparison.opponent_median_ms,
                comparison.tool_vs_opponent_x,
                comparison.winner
            )
        })
        .collect()
}

fn project_check_vs_corsa_json(rows: &[ToolRow]) -> Vec<serde_json::Value> {
    project_check_vs_corsa(rows)
        .into_iter()
        .map(|comparison| {
            json!({
                "dataset": comparison.dataset,
                "tool": comparison.tool,
                "medianMs": comparison.median_ms,
                "baselineTool": comparison.baseline_tool,
                "baselineMedianMs": comparison.baseline_median_ms,
                "vsBaselineX": comparison.vs_baseline_x,
            })
        })
        .collect()
}

fn project_check_requested_opponent_json(rows: &[ToolRow]) -> Vec<serde_json::Value> {
    project_check_requested_opponents(rows)
        .into_iter()
        .map(|comparison| {
            json!({
                "dataset": comparison.dataset,
                "tool": comparison.tool,
                "opponentTool": comparison.opponent_tool,
                "toolMedianMs": comparison.tool_median_ms,
                "opponentMedianMs": comparison.opponent_median_ms,
                "toolVsOpponentX": comparison.tool_vs_opponent_x,
                "winner": comparison.winner,
            })
        })
        .collect()
}

fn workflow_vs_corsa_json(rows: &[ToolRow]) -> Vec<serde_json::Value> {
    workflow_vs_corsa(rows)
        .into_iter()
        .map(|comparison| {
            json!({
                "dataset": comparison.dataset,
                "tool": comparison.tool,
                "medianMs": comparison.median_ms,
                "baselineTool": comparison.baseline_tool,
                "baselineMedianMs": comparison.baseline_median_ms,
                "vsBaselineX": comparison.vs_baseline_x,
            })
        })
        .collect()
}

struct Comparison<'a> {
    dataset: &'a str,
    tool: &'a str,
    baseline_tool: &'a str,
    median_ms: f64,
    baseline_median_ms: f64,
    vs_baseline_x: f64,
}

struct RequestedOpponentComparison<'a> {
    dataset: &'a str,
    tool: &'a str,
    opponent_tool: &'a str,
    tool_median_ms: f64,
    opponent_median_ms: f64,
    tool_vs_opponent_x: f64,
    winner: &'a str,
}

fn project_check_vs_corsa(rows: &[ToolRow]) -> Vec<Comparison<'_>> {
    let mut items = Vec::new();
    for row in rows
        .iter()
        .filter(|row| row.workload.as_str() == "project_check")
    {
        let Some(corsa) = rows.iter().find(|candidate| {
            candidate.workload.as_str() == "project_check"
                && candidate.dataset == row.dataset
                && candidate.tool.as_str() == "corsa"
        }) else {
            continue;
        };
        items.push(Comparison {
            dataset: row.dataset.as_str(),
            tool: row.tool.as_str(),
            baseline_tool: "corsa",
            median_ms: row.stats.median_ms(),
            baseline_median_ms: corsa.stats.median_ms(),
            vs_baseline_x: corsa.stats.median_ms() / row.stats.median_ms(),
        });
    }
    sort_comparisons(&mut items);
    items
}

fn project_check_requested_opponents(rows: &[ToolRow]) -> Vec<RequestedOpponentComparison<'_>> {
    let mut items = Vec::new();
    for row in rows.iter().filter(|row| {
        row.workload.as_str() == "project_check" && is_requested_opponent(row.tool.as_str())
    }) {
        let Some(corsa) = rows.iter().find(|candidate| {
            candidate.workload.as_str() == "project_check"
                && candidate.dataset == row.dataset
                && candidate.tool.as_str() == "corsa"
        }) else {
            continue;
        };
        let tool_median_ms = corsa.stats.median_ms();
        let opponent_median_ms = row.stats.median_ms();
        items.push(RequestedOpponentComparison {
            dataset: row.dataset.as_str(),
            tool: corsa.tool.as_str(),
            opponent_tool: row.tool.as_str(),
            tool_median_ms,
            opponent_median_ms,
            tool_vs_opponent_x: opponent_median_ms / tool_median_ms,
            winner: if tool_median_ms <= opponent_median_ms {
                corsa.tool.as_str()
            } else {
                row.tool.as_str()
            },
        });
    }
    sort_requested_opponent_comparisons(&mut items);
    items
}

fn workflow_vs_corsa(rows: &[ToolRow]) -> Vec<Comparison<'_>> {
    let mut items = Vec::new();
    for row in rows
        .iter()
        .filter(|row| row.workload.as_str() == "editor_workflow")
    {
        let Some(corsa) = rows.iter().find(|candidate| {
            candidate.workload.as_str() == "project_check"
                && candidate.dataset == row.dataset
                && candidate.tool.as_str() == "corsa"
        }) else {
            continue;
        };
        items.push(Comparison {
            dataset: row.dataset.as_str(),
            tool: row.tool.as_str(),
            baseline_tool: "corsa-cli-project-check",
            median_ms: row.stats.median_ms(),
            baseline_median_ms: corsa.stats.median_ms(),
            vs_baseline_x: corsa.stats.median_ms() / row.stats.median_ms(),
        });
    }
    sort_comparisons(&mut items);
    items
}

fn is_requested_opponent(tool: &str) -> bool {
    matches!(tool, "typescript-eslint" | "tsgolint" | "oxlint-bare")
}

fn sort_comparisons(items: &mut [Comparison<'_>]) {
    items.sort_by(|left, right| {
        left.dataset
            .cmp(right.dataset)
            .then_with(|| left.tool.cmp(right.tool))
    });
}

fn sort_requested_opponent_comparisons(items: &mut [RequestedOpponentComparison<'_>]) {
    items.sort_by(|left, right| {
        left.dataset
            .cmp(right.dataset)
            .then_with(|| left.tool.cmp(right.tool))
            .then_with(|| left.opponent_tool.cmp(right.opponent_tool))
    });
}
