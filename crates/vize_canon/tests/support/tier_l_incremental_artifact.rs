use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vize_canon::IncrementalCheckMetrics;
use vize_s0::{String, append, cstr};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BatchIncrementalBudget {
    pub(super) cold_ms: u64,
    pub(super) warm_ms: u64,
    pub(super) max_requested_files: usize,
    pub(super) max_changed_files: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Artifact {
    pub(super) schema_version: u8,
    pub(super) fixture: FixtureEvidence,
    pub(super) budget: BatchIncrementalBudget,
    pub(super) budget_scale: f64,
    pub(super) file_count: usize,
    pub(super) lanes: Vec<LaneEvidence>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FixtureEvidence {
    pub(super) id: &'static str,
    pub(super) revision: String,
    pub(super) injected_file: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LaneEvidence {
    name: &'static str,
    duration_ms: u128,
    metrics: MetricsEvidence,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MetricsEvidence {
    checks: usize,
    session_starts: usize,
    session_reuses: usize,
    session_refreshes: usize,
    session_to_cli_fallbacks: usize,
    requested_files: usize,
    changed_files: usize,
    created_files: usize,
    deleted_files: usize,
}

impl From<IncrementalCheckMetrics> for MetricsEvidence {
    fn from(metrics: IncrementalCheckMetrics) -> Self {
        Self {
            checks: metrics.checks,
            session_starts: metrics.session_starts,
            session_reuses: metrics.session_reuses,
            session_refreshes: metrics.session_refreshes,
            session_to_cli_fallbacks: metrics.session_to_cli_fallbacks,
            requested_files: metrics.last_requested_files,
            changed_files: metrics.last_changed_files,
            created_files: metrics.last_created_files,
            deleted_files: metrics.last_deleted_files,
        }
    }
}

pub(super) fn lane(
    name: &'static str,
    duration_ms: u128,
    metrics: IncrementalCheckMetrics,
) -> LaneEvidence {
    LaneEvidence {
        name,
        duration_ms,
        metrics: metrics.into(),
    }
}

pub(super) fn write_artifact(repo_root: &Path, artifact: &Artifact) {
    let output_dir = output_dir(repo_root);
    fs::create_dir_all(&output_dir).expect("metrics directory should be created");
    fs::write(
        output_dir.join("metrics.json"),
        serde_json::to_vec_pretty(artifact).expect("metrics should serialize"),
    )
    .expect("metrics JSON should write");

    let mut summary = cstr!(
        "# Vue Vben Admin batch incremental oracle\n\n| lane | duration (ms) | requested | changed | session reuse | fallback |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for lane in &artifact.lanes {
        append!(
            summary,
            "| {} | {} | {} | {} | {} | {} |\n",
            lane.name,
            lane.duration_ms,
            lane.metrics.requested_files,
            lane.metrics.changed_files,
            lane.metrics.session_reuses,
            lane.metrics.session_to_cli_fallbacks,
        );
    }
    append!(
        summary,
        "\nDeterministic gates require one session start, two reuses, exact one-file warm deltas, and zero CLI fallbacks. Durations are independent hard ceilings at budget scale {}.\n",
        artifact.budget_scale,
    );
    fs::write(output_dir.join("summary.md"), summary).expect("metrics summary should write");
}

fn output_dir(repo_root: &Path) -> PathBuf {
    let path = std::env::var_os("VIZE_TIER_L_METRICS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/vize-tests/metrics/vben-batch-incremental"));
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}
