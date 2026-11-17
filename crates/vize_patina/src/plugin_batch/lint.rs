//! Plugin lint jobs shared by `vize lint` and the language server (#13074).
//!
//! Admitted `lint.rule` capabilities and adapted v1 `lint.plugin-v1` plugins
//! are planned once from the verified plugin plan for one product. Disabled
//! rules add no work. A document is lowered once; every enabled job gets its
//! batch from the same records and facts, and all jobs cross into the trusted
//! JS sidecar in one request. Answers map back to host-owned authored spans.
//! Declared `vue.directive` contracts are checked natively on the same
//! records and never cross into JS.

#![expect(
    clippy::disallowed_types,
    reason = "plugin batch specs and JSON values carry std strings"
)]

#[cfg(not(target_arch = "wasm32"))]
pub mod host;
mod rules;

use serde_json::Value;
use vize_carton::config::plugin_plan::{PlannedCapability, PlannedPlugin, VerifiedPluginPlan};
use vize_l0::config::{DirectiveContract, LintRuleSeverity};
use vize_l0::fact::FactManager;
use vize_l0::{FxHashMap, FxHashSet, String, cstr};

use super::batch::{BuiltBatch, PluginSpec, build_batch};
use super::directives::{DIRECTIVE_CONTRACT_RULE, directive_contract_diagnostics};
use super::document::PluginDocument;
use super::facts::REGISTRY;
use crate::LintDiagnostic;
use rules::Rules;

const PLUGIN_RULE_DOCS: &str = "docs/release/integrated-plugin-incubation.md#plugin-lint-rules";
const DIRECTIVE_DOCS: &str = "docs/release/integrated-plugin-incubation.md#decisions";

/// A JSON member, or `null` when absent.
fn field<'v>(value: &'v Value, key: &str) -> &'v Value {
    static NULL: Value = Value::Null;
    value.get(key).unwrap_or(&NULL)
}

/// Every public plugin rule ID in `plan`, enabled or not, for config validation.
pub fn rule_ids(plan: &VerifiedPluginPlan) -> Vec<String> {
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
    rules: Rules,
}

/// The enabled plugin lint jobs of one verified plan.
pub struct PluginLintJobs {
    digest: String,
    jobs: Vec<Job>,
    directives: Vec<DirectiveContract>,
}

/// One Vue document lowered once, with its batch for every job.
pub struct LoweredDocument {
    /// The document the reports map back to.
    pub document: PluginDocument,
    /// Its batch for every job, in job order.
    pub batches: Vec<BuiltBatch>,
    /// Diagnostics found without JS: directive contract violations.
    pub native: Vec<LintDiagnostic>,
}

/// One job's answer: its JS time and the diagnostics of each document.
pub struct JobAnswer {
    /// Nanoseconds the job spent in JS.
    pub js_ns: f64,
    /// Diagnostics per requested document, in request order.
    pub documents: Vec<Vec<LintDiagnostic>>,
}

impl PluginLintJobs {
    /// Plan the jobs `product` runs; `None` when no plugin rule is enabled
    /// and no directive is declared.
    pub fn plan(
        plan: &VerifiedPluginPlan,
        severities: &FxHashMap<String, LintRuleSeverity>,
        product: &str,
    ) -> Result<Option<Self>, String> {
        plan.admit(product, None)
            .map_err(|error| cstr!("{error}"))?;
        let mut jobs = Vec::new();
        for plugin in &plan.plan().plugins {
            for capability in &plugin.capabilities {
                let job = match capability.id.as_str() {
                    "lint.rule" => Rules::v2(plugin, capability, severities)?,
                    "lint.plugin-v1" => Rules::v1(plugin, capability, severities)?,
                    _ => continue,
                };
                let Some(rules) = job else { continue };
                jobs.push(job_for(plugin, capability, rules)?);
            }
        }
        let directives = plan.directives().map_err(|error| cstr!("{error}"))?;
        if jobs.is_empty() && directives.is_empty() {
            return Ok(None);
        }
        // Plugin rules document themselves through `meta.docs` help; their
        // generic page explains where plugin rules come from.
        crate::register_plugin_rule_docs(
            jobs.iter()
                .flat_map(|job| job.rules.ids())
                .map(|id| (id, String::from(PLUGIN_RULE_DOCS)))
                .chain(
                    (!directives.is_empty())
                        .then(|| (DIRECTIVE_CONTRACT_RULE, String::from(DIRECTIVE_DOCS))),
                )
                .collect(),
        );
        Ok(Some(Self {
            digest: plan.digest().into(),
            jobs,
            directives,
        }))
    }

    /// Whether any job runs in JS. Without one, no sidecar is needed.
    pub fn crosses(&self) -> bool {
        !self.jobs.is_empty()
    }

    /// The plan digest the sidecar must answer for.
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// `(plugin, key)` per job, in request order.
    pub fn labels(&self) -> Vec<(&str, &str)> {
        self.jobs
            .iter()
            .map(|job| (job.plugin.as_str(), job.key.as_str()))
            .collect()
    }

    /// Rule IDs whose fixes `--fix` may apply (`meta.fixable === "safe"`).
    pub fn safe_fix_ids(&self) -> FxHashSet<&'static str> {
        self.jobs
            .iter()
            .filter_map(|job| job.rules.safe_fix_id())
            .collect()
    }

    /// Lower one SFC for every job. `Ok(None)` when the plugin records cannot
    /// represent its template.
    pub fn lower(&self, source: &str, filename: &str) -> Result<Option<LoweredDocument>, String> {
        let Ok(document) = PluginDocument::build(source, filename) else {
            return Ok(None);
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
        let native = directive_contract_diagnostics(&document, &self.directives);
        Ok(Some(LoweredDocument {
            document,
            batches,
            native,
        }))
    }

    /// One sidecar request carrying every job over `documents`.
    pub fn request(&self, documents: &[&LoweredDocument]) -> Result<Vec<u8>, String> {
        let mut jobs: Vec<Vec<Value>> = vec![Vec::new(); self.jobs.len()];
        for lowered in documents {
            for (position, built) in lowered.batches.iter().enumerate() {
                if let Some(batches) = jobs.get_mut(position) {
                    batches.push(serde_json::json!({ "file": lowered.document.filename.as_str(), "batch": built.json }));
                }
            }
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
        serde_json::to_vec(&request).map_err(|error| cstr!("{error}"))
    }

    /// Map the sidecar's answer to diagnostics, per job and then per document.
    pub fn answers(
        &self,
        response: &[u8],
        documents: &[&LoweredDocument],
    ) -> Result<Vec<JobAnswer>, String> {
        let response: Value = serde_json::from_slice(response)
            .map_err(|error| cstr!("[plugin/plan-invalid] unreadable sidecar response: {error}"))?;
        // A plan refusal or rule failure, answered instead of crashing the sidecar.
        if let Some(error) = field(&response, "error").as_str() {
            return Err(error.into());
        }
        let answered = field(&response, "jobs")
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        if answered.len() != self.jobs.len()
            || field(&response, "digest").as_str() != Some(&self.digest)
        {
            return Err("[plugin/plan-stale] plan: the sidecar answered a different plan".into());
        }
        self.jobs
            .iter()
            .zip(answered)
            .map(|(job, answer)| {
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
                Ok(JobAnswer {
                    js_ns: field(answer, "jsNs").as_f64().unwrap_or_default(),
                    documents: documents
                        .iter()
                        .zip(replies)
                        .map(|(lowered, reply)| {
                            job.rules.map(&lowered.document, field(reply, "reports"))
                        })
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect()
    }
}

fn job_for(
    plugin: &PlannedPlugin,
    capability: &PlannedCapability,
    rules: Rules,
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
