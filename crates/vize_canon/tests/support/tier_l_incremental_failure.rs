//! Preserve completed work when an unchanged Tier-L assertion unwinds.

use std::{fs, io::Write, path::PathBuf};

use serde_json::{Value, json};
use vize_canon::IncrementalCheckMetrics;
use vize_l0::String;

use super::artifact::BatchIncrementalBudget;

const LANES: [&str; 3] = ["cold", "brokenWarm", "repairedWarm"];

pub(super) struct FailureReceipt {
    output_dir: PathBuf,
    fixture_id: &'static str,
    revision: String,
    injected_file: &'static str,
    budget: BatchIncrementalBudget,
    budget_scale: f64,
    file_count: Option<usize>,
    completed: [Option<(u128, IncrementalCheckMetrics)>; 3],
    running: Option<usize>,
    armed: bool,
}

impl FailureReceipt {
    pub(super) fn new(
        output_dir: PathBuf,
        fixture_id: &'static str,
        revision: String,
        injected_file: &'static str,
        budget: BatchIncrementalBudget,
        budget_scale: f64,
    ) -> Self {
        Self {
            output_dir,
            fixture_id,
            revision,
            injected_file,
            budget,
            budget_scale,
            file_count: None,
            completed: [None; 3],
            running: None,
            armed: true,
        }
    }

    pub(super) fn begin(&mut self, lane: usize) {
        self.running = Some(lane);
    }

    pub(super) fn complete(
        &mut self,
        lane: usize,
        duration_ms: u128,
        metrics: IncrementalCheckMetrics,
        file_count: usize,
    ) {
        self.completed[lane] = Some((duration_ms, metrics));
        self.file_count = Some(file_count);
        self.running = None;
    }

    pub(super) fn disarm(&mut self) {
        self.armed = false;
    }

    fn write_failure(&self) -> Result<(), Box<dyn std::error::Error>> {
        let lanes: Vec<_> = LANES.iter().enumerate().map(|(index, name)| {
            match self.completed[index] {
                Some((duration, metrics)) => json!({
                    "name": name, "state": "COMPLETED", "durationMs": duration,
                    "metrics": complete_metrics(metrics),
                }),
                None => json!({
                    "name": name,
                    "state": if self.running == Some(index) { "IN_PROGRESS" } else { "NOT_EXECUTED" },
                    "durationMs": null, "metrics": null,
                }),
            }
        }).collect();
        let receipt = json!({
            "schemaVersion": 1, "outcome": "ASSERTION_UNWIND",
            "fixture": { "id": self.fixture_id, "revision": self.revision, "injectedFile": self.injected_file },
            "budget": self.budget, "budgetScale": self.budget_scale,
            "fileCount": self.file_count, "lanes": lanes,
            "nativeProgramCount": null, "nativeStartupCpu": null,
        });
        fs::create_dir_all(&self.output_dir)?;
        fs::write(
            self.output_dir.join("failure.json"),
            serde_json::to_vec_pretty(&receipt)?,
        )?;
        Ok(())
    }
}

impl Drop for FailureReceipt {
    fn drop(&mut self) {
        if self.armed && std::thread::panicking() {
            if let Err(error) = self.write_failure() {
                // Preserve the original assertion even if its receipt cannot be written.
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "Tier-L failure receipt could not be written: {error}"
                );
            }
        }
    }
}

fn complete_metrics(m: IncrementalCheckMetrics) -> Value {
    json!({
        "checks": m.checks, "sessionStarts": m.session_starts,
        "sessionReuses": m.session_reuses, "sessionRefreshes": m.session_refreshes,
        "sessionToCliFallbacks": m.session_to_cli_fallbacks,
        "lastSessionStarted": m.last_session_started,
        "lastSessionReused": m.last_session_reused,
        "lastSessionRefreshed": m.last_session_refreshed,
        "lastSessionToCliFallback": m.last_session_to_cli_fallback,
        "lastRequestedFiles": m.last_requested_files, "lastChangedFiles": m.last_changed_files,
        "lastCreatedFiles": m.last_created_files, "lastDeletedFiles": m.last_deleted_files,
        "lastMaterializedEntriesConsidered": m.last_materialized_entries_considered,
        "lastTreeEntriesScanned": m.last_tree_entries_scanned,
        "lastFullRebuild": m.last_full_rebuild,
        "lastSourceNodesRebuilt": m.last_source_nodes_rebuilt,
        "lastDependencyNodesReconciled": m.last_dependency_nodes_reconciled,
        "lastShadowBindingsRebuilt": m.last_shadow_bindings_rebuilt,
    })
}

#[cfg(test)]
#[path = "tier_l_incremental_failure_tests.rs"]
mod tests;
