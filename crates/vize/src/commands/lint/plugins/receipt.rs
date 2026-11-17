//! Per-plugin cost attribution for one lint run.

use std::time::{Duration, Instant};

use vize_l0::String;

#[derive(Default)]
struct JobCost {
    batches: usize,
    nodes: u64,
    batch_bytes: u64,
    reports: usize,
    js_ns: f64,
}

/// What the plugin pass cost: work per job, the one sidecar crossing, totals.
pub(crate) struct PluginLintReceipt {
    jobs: Vec<JobCost>,
    /// Vue documents lowered to plugin records.
    pub(crate) documents: usize,
    /// Documents whose template the plugin records cannot represent.
    pub(crate) skipped: Vec<String>,
    crossings: usize,
    request_bytes: usize,
    response_bytes: usize,
    sidecar: Duration,
    total: Duration,
}

impl PluginLintReceipt {
    pub(super) fn new(jobs: usize) -> Self {
        Self {
            jobs: (0..jobs).map(|_| JobCost::default()).collect(),
            documents: 0,
            skipped: Vec::new(),
            crossings: 0,
            request_bytes: 0,
            response_bytes: 0,
            sidecar: Duration::ZERO,
            total: Duration::ZERO,
        }
    }

    pub(super) fn record_batch(&mut self, job: usize, nodes: u32, bytes: usize) {
        if let Some(cost) = self.jobs.get_mut(job) {
            cost.batches += 1;
            cost.nodes += u64::from(nodes);
            cost.batch_bytes += bytes as u64;
        }
    }

    pub(super) fn record_crossing(&mut self, request: usize, response: usize, elapsed: Duration) {
        self.crossings += 1;
        self.request_bytes += request;
        self.response_bytes += response;
        self.sidecar += elapsed;
    }

    pub(super) fn record_js(&mut self, job: usize, js_ns: f64) {
        if let Some(cost) = self.jobs.get_mut(job) {
            cost.js_ns += js_ns;
        }
    }

    pub(super) fn record_reports(&mut self, job: usize, reports: usize) {
        if let Some(cost) = self.jobs.get_mut(job) {
            cost.reports += reports;
        }
    }

    pub(super) fn finish(&mut self, started: Instant) {
        self.total = started.elapsed();
    }

    /// The machine-readable receipt `--profile` prints.
    pub(crate) fn to_json(&self, labels: &[(&str, &str)]) -> serde_json::Value {
        serde_json::json!({
            "crossings": self.crossings,
            // One sidecar process serves every crossing of the run.
            "processes": 1,
            "documents": self.documents,
            "jobs": self.jobs.iter().zip(labels).map(|(cost, (plugin, key))| serde_json::json!({
                "batchBytes": cost.batch_bytes,
                "batches": cost.batches,
                "jsMs": cost.js_ns / 1e6,
                "key": key,
                "nodes": cost.nodes,
                "plugin": plugin,
                "reports": cost.reports,
            })).collect::<Vec<_>>(),
            "requestBytes": self.request_bytes,
            "responseBytes": self.response_bytes,
            "sidecarMs": self.sidecar.as_secs_f64() * 1e3,
            "skipped": self.skipped.iter().map(String::as_str).collect::<Vec<_>>(),
            "totalMs": self.total.as_secs_f64() * 1e3,
        })
    }
}
