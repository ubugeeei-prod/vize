#![expect(
    clippy::disallowed_types,
    reason = "plugin batch specs and JSON values carry std strings"
)]
//! Plugin API v2 lint (#13074): admitted `lint.rule` capabilities and adapted
//! v1 `lint.plugin-v1` plugins from `vue.plugins`.
//!
//! The pass is planned once from the verified plugin plan. Disabled rules add
//! no work. Enabled jobs share one record lowering and fact computation per
//! document, and every job crosses into the trusted JS sidecar in one request
//! per lint run. Reports map back to host-owned authored spans.

mod fix;
mod receipt;
mod reports;

use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::config::plugin_plan::{PlannedPlugin, VerifiedPluginPlan};
use serde_json::Value;
use vize_l0::fact::FactManager;
use vize_l0::{FxHashMap, String, cstr};
use vize_patina::Severity;
use vize_patina::plugin_batch::{
    batch::{PluginSpec, build_batch},
    document::PluginDocument,
    facts::REGISTRY,
};

pub(super) use receipt::PluginLintReceipt;

const PLUGIN_RULE_DOCS: &str = "docs/release/integrated-plugin-incubation.md#plugin-lint-rules";

/// A JSON member, or `null` when absent.
fn field<'v>(value: &'v Value, key: &str) -> &'v Value {
    static NULL: Value = Value::Null;
    value.get(key).unwrap_or(&NULL)
}

/// Every public plugin rule ID in `plan`, enabled or not, for config validation.
pub(super) fn rule_ids(plan: &VerifiedPluginPlan) -> Vec<String> {
    let mut ids = Vec::new();
    for plugin in &plan.plan().plugins {
        for capability in &plugin.capabilities {
            match capability.id.as_str() {
                "lint.rule" => ids.push(capability.key.clone()),
                "lint.plugin-v1" => ids.extend(
                    field(&capability.data, "rules")
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|rule| cstr!("{}/{rule}", plugin.name)),
                ),
                _ => {}
            }
        }
    }
    ids
}

/// One admitted capability with at least one enabled rule.
struct Job {
    plugin: String,
    capability: &'static str,
    key: String,
    version: String,
    fingerprint: String,
    visit: Option<Vec<std::string::String>>,
    demands: Vec<std::string::String>,
    rules: reports::Rules,
}

pub(super) struct PluginLint {
    config_path: PathBuf,
    digest: String,
    jobs: Vec<Job>,
}

impl PluginLint {
    /// Plan the lint jobs of `plan`; `None` when no plugin rule is enabled.
    pub(super) fn plan(
        plan: &VerifiedPluginPlan,
        severities: &FxHashMap<String, crate::config::LintRuleSeverity>,
        config_path: Option<&Path>,
    ) -> Result<Option<Self>, String> {
        plan.admit("lint", None).map_err(|error| cstr!("{error}"))?;
        let mut jobs = Vec::new();
        for plugin in &plan.plan().plugins {
            for capability in &plugin.capabilities {
                let job = match capability.id.as_str() {
                    "lint.rule" => reports::Rules::v2(plugin, capability, severities)?,
                    "lint.plugin-v1" => reports::Rules::v1(plugin, capability, severities)?,
                    _ => continue,
                };
                let Some(rules) = job else { continue };
                jobs.push(job_for(plugin, capability, rules)?);
            }
        }
        if jobs.is_empty() {
            return Ok(None);
        }
        // Plugin rules document themselves through `meta.docs` help; their
        // generic page explains where plugin rules come from.
        vize_patina::register_plugin_rule_docs(
            jobs.iter()
                .flat_map(|job| job.rules.ids())
                .map(|id| (id, String::from(PLUGIN_RULE_DOCS)))
                .collect(),
        );
        Ok(Some(Self {
            config_path: sidecar_config(config_path)?,
            digest: plan.digest().into(),
            jobs,
        }))
    }

    /// `(plugin, key)` per job, in receipt order.
    pub(super) fn labels(&self) -> Vec<(&str, &str)> {
        self.jobs
            .iter()
            .map(|job| (job.plugin.as_str(), job.key.as_str()))
            .collect()
    }

    /// Lint every Vue SFC once with every job; one sidecar crossing in total.
    pub(super) fn run(
        &self,
        results: &mut [super::cross_file::CliLintFileResult],
    ) -> Result<PluginLintReceipt, String> {
        let started = Instant::now();
        let mut receipt = PluginLintReceipt::new(self.jobs.len());
        let receipt_result = self.pass(results, None, &mut receipt);
        receipt.finish(started);
        receipt_result.map(|()| receipt)
    }

    /// One plugin pass over `only` (or every result): one sidecar crossing.
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
                let Ok(document) = PluginDocument::build(source, filename) else {
                    return Ok(Err(filename.clone()));
                };
                let mut facts = FactManager::new(&REGISTRY);
                let batches = self
                    .jobs
                    .iter()
                    .map(|job| {
                        let spec = PluginSpec {
                            name: &job.plugin,
                            version: &job.version,
                            fingerprint: &job.fingerprint,
                            visit: job.visit.as_deref(),
                            demands: &job.demands,
                        };
                        build_batch(&document, &spec, &mut facts)
                            .map_err(|error| cstr!("[plugin/plan-invalid] {}: {error}", job.plugin))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok(Ok((index, document, batches)))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut documents = Vec::with_capacity(lowered.len());
        let mut jobs: Vec<Vec<Value>> = vec![Vec::new(); self.jobs.len()];
        for outcome in lowered {
            let (index, document, batches) = match outcome {
                Ok(lowered) => lowered,
                Err(filename) => {
                    receipt.skipped.push(filename);
                    continue;
                }
            };
            for (position, built) in batches.into_iter().enumerate() {
                receipt.record_batch(position, built.nodes, built.json.len());
                if let Some(batches) = jobs.get_mut(position) {
                    batches.push(serde_json::json!({ "file": document.filename.as_str(), "batch": built.json }));
                }
            }
            documents.push((index, document));
        }
        receipt.documents += documents.len();
        if documents.is_empty() {
            return Ok(());
        }
        let request = serde_json::json!({
            "digest": self.digest.as_str(),
            "jobs": self.jobs.iter().zip(jobs).map(|(job, documents)| serde_json::json!({
                "capability": job.capability,
                "documents": documents,
                "key": job.key.as_str(),
                "plugin": job.plugin.as_str(),
            })).collect::<Vec<_>>(),
        });
        let request = serde_json::to_vec(&request).map_err(|error| cstr!("{error}"))?;
        let sidecar = Instant::now();
        let response =
            crate::config::run_plugin_lint(&self.config_path, &request).map_err(String::from)?;
        receipt.record_crossing(request.len(), response.len(), sidecar.elapsed());
        let response: Value = serde_json::from_slice(&response)
            .map_err(|error| cstr!("[plugin/plan-invalid] unreadable sidecar response: {error}"))?;
        let answered = field(&response, "jobs")
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        if answered.len() != self.jobs.len()
            || field(&response, "digest").as_str() != Some(&self.digest)
        {
            return Err("[plugin/plan-stale] plan: the sidecar answered a different plan".into());
        }
        for ((position, job), answer) in self.jobs.iter().enumerate().zip(answered) {
            receipt.record_js(position, field(answer, "jsNs").as_f64().unwrap_or_default());
            let replies = field(answer, "documents")
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            if replies.len() != documents.len() {
                return Err(cstr!(
                    "[plugin/plan-stale] {}: missing document reports",
                    job.plugin
                ));
            }
            for ((index, document), reply) in documents.iter().zip(replies) {
                let diagnostics = job.rules.map(document, field(reply, "reports"))?;
                receipt.record_reports(position, diagnostics.len());
                if let Some((_, _, _, result)) = results.get_mut(*index) {
                    for diagnostic in diagnostics {
                        match diagnostic.severity {
                            Severity::Error => result.error_count += 1,
                            Severity::Warning => result.warning_count += 1,
                        }
                        result.diagnostics.push(diagnostic);
                    }
                }
            }
        }
        Ok(())
    }
}

fn job_for(
    plugin: &PlannedPlugin,
    capability: &crate::config::plugin_plan::PlannedCapability,
    rules: reports::Rules,
) -> Result<Job, String> {
    let strings = |value: &Value| -> Option<Vec<std::string::String>> {
        value
            .as_array()?
            .iter()
            .map(|item| item.as_str().map(std::string::String::from))
            .collect()
    };
    let data = &capability.data;
    let invalid = || {
        cstr!(
            "[plugin/plan-invalid] {}: malformed {}",
            plugin.name,
            capability.id
        )
    };
    Ok(Job {
        plugin: plugin.name.clone(),
        capability: if capability.id == "lint.rule" {
            "lint.rule"
        } else {
            "lint.plugin-v1"
        },
        key: capability.key.clone(),
        version: plugin.version.clone(),
        fingerprint: plugin.fingerprint.clone(),
        visit: if field(data, "visit").is_null() {
            None
        } else {
            Some(strings(field(data, "visit")).ok_or_else(invalid)?)
        },
        demands: strings(field(data, "demands")).ok_or_else(invalid)?,
        rules,
    })
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
