//! Plugin API v2 lint (#13074) in `vize lint`: the shared plugin lint jobs
//! over every Vue SFC of the run.
//!
//! Every job crosses into the trusted JS sidecar in one request per pass. The
//! sidecar starts when the pass is planned, so Node startup and config
//! evaluation overlap the built-in lint pass, and `--fix` passes reuse it.
//! A plan whose only lint work is native (declared directive contracts) never
//! starts one.

mod fix;
mod receipt;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::config::plugin_plan::VerifiedPluginPlan;
use vize_l0::{FxHashMap, String};
use vize_patina::Severity;
use vize_patina::plugin_batch::lint::PluginLintJobs;

pub(super) use receipt::PluginLintReceipt;

/// Every public plugin rule ID in `plan`, enabled or not, for config validation.
pub(super) fn rule_ids(plan: &VerifiedPluginPlan) -> Vec<String> {
    vize_patina::plugin_batch::lint::rule_ids(plan)
}

pub(super) struct PluginLint {
    sidecar: Option<std::sync::Mutex<crate::config::PluginLintSidecar>>,
    jobs: PluginLintJobs,
}

impl PluginLint {
    /// Plan the lint jobs of `plan`; `None` when it adds no lint work.
    pub(super) fn plan(
        plan: &VerifiedPluginPlan,
        severities: &FxHashMap<String, crate::config::LintRuleSeverity>,
        config_path: Option<&Path>,
    ) -> Result<Option<Self>, String> {
        let Some(jobs) = PluginLintJobs::plan(plan, severities, "lint")? else {
            return Ok(None);
        };
        let sidecar = if jobs.crosses() {
            Some(std::sync::Mutex::new(
                crate::config::PluginLintSidecar::start(&sidecar_config(config_path)?)?,
            ))
        } else {
            None
        };
        Ok(Some(Self { sidecar, jobs }))
    }

    /// `(plugin, key)` per job, in receipt order.
    pub(super) fn labels(&self) -> Vec<(&str, &str)> {
        self.jobs.labels()
    }

    /// Lint every Vue SFC once with every job; one sidecar crossing in total.
    pub(super) fn run(
        &self,
        results: &mut [super::cross_file::CliLintFileResult],
    ) -> Result<PluginLintReceipt, String> {
        let started = Instant::now();
        let mut receipt = PluginLintReceipt::new(self.jobs.labels().len());
        let receipt_result = self.pass(results, None, &mut receipt);
        receipt.finish(started);
        receipt_result.map(|()| receipt)
    }

    /// One plugin pass over `only` (or every result): one sidecar crossing
    /// when any job runs in JS.
    fn pass(
        &self,
        results: &mut [super::cross_file::CliLintFileResult],
        only: Option<&[usize]>,
        receipt: &mut PluginLintReceipt,
    ) -> Result<(), String> {
        // Lowering and fact computation are per document, so they run in parallel.
        let lowered = results
            .par_iter()
            .enumerate()
            .filter(|(index, (path, ..))| {
                // `only` is ascending: fix passes collect it in file order.
                only.is_none_or(|only| only.binary_search(index).is_ok())
                    && path.extension().is_some_and(|extension| extension == "vue")
            })
            .map(|(index, (_, filename, source, _))| {
                Ok(match self.jobs.lower(source, filename)? {
                    Some(lowered) => Ok((index, lowered)),
                    None => Err(filename.clone()),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut documents = Vec::with_capacity(lowered.len());
        for outcome in lowered {
            let (index, mut lowered) = match outcome {
                Ok(lowered) => lowered,
                Err(filename) => {
                    receipt.skipped.push(filename);
                    continue;
                }
            };
            if let Some((_, _, _, result)) = results.get_mut(index) {
                push_diagnostics(result, std::mem::take(&mut lowered.native));
            }
            for (position, built) in lowered.batches.iter().enumerate() {
                receipt.record_batch(position, built.nodes, built.json.len());
            }
            documents.push((index, lowered));
        }
        receipt.documents += documents.len();
        let Some(sidecar_slot) = self.sidecar.as_ref() else {
            return Ok(());
        };
        if documents.is_empty() {
            return Ok(());
        }
        let lowered: Vec<_> = documents.iter().map(|(_, lowered)| lowered).collect();
        let request = self.jobs.request(&lowered)?;
        let sidecar = Instant::now();
        let response = sidecar_slot
            .lock()
            .map_err(|_| {
                String::from("[plugin/plan-invalid] plan: the plugin lint sidecar is unavailable")
            })?
            .request(&request)
            .map_err(String::from)?;
        receipt.record_crossing(request.len(), response.len(), sidecar.elapsed());
        let answers = self.jobs.answers(&response, &lowered)?;
        for (position, answer) in answers.into_iter().enumerate() {
            receipt.record_js(position, answer.js_ns);
            for ((index, _), diagnostics) in documents.iter().zip(answer.documents) {
                receipt.record_reports(position, diagnostics.len());
                if let Some((_, _, _, result)) = results.get_mut(*index) {
                    push_diagnostics(result, diagnostics);
                }
            }
        }
        Ok(())
    }
}

fn push_diagnostics(
    result: &mut vize_patina::LintResult,
    diagnostics: Vec<vize_patina::LintDiagnostic>,
) {
    for diagnostic in diagnostics {
        match diagnostic.severity {
            Severity::Error => result.error_count += 1,
            Severity::Warning => result.warning_count += 1,
        }
        result.diagnostics.push(diagnostic);
    }
}

/// The sidecar re-evaluates the authored Vite config and checks its digest.
fn sidecar_config(config_path: Option<&Path>) -> Result<PathBuf, String> {
    let is_vite = |path: &Path| {
        path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("vite.config."))
    };
    if let Some(path) = config_path.filter(|path| is_vite(path)) {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    [
        "vite.config.ts",
        "vite.config.js",
        "vite.config.mjs",
        "vite.config.cjs",
        "vite.config.mts",
        "vite.config.cts",
    ]
    .into_iter()
    .map(|name| cwd.join(name))
    .find(|path| is_vite(path))
    .ok_or_else(|| {
        "[plugin/plan-invalid] plan: plugin lint rules run from vite.config.*, which was not found"
            .into()
    })
}
