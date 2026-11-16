//! Plugin lint rules in the editor (#13074).
//!
//! A config generation that enables plugin rules owns one lazily started
//! sidecar; edits never spawn a process. Each lint sends one document, with
//! every job, as one request. A request is bounded: the sidecar must resolve
//! its config within the startup budget and answer within the answer budget,
//! or it is stopped and restarted on the next lint. A newer document version
//! cancels the wait and the late answer is skipped. After repeated failures
//! the host reports the last one instead of restarting until the config
//! changes. The last answer for each document text is kept, so its
//! diagnostics, hover and code actions cross into JS once.

#![expect(
    clippy::disallowed_types,
    reason = "plugin batch specs and sidecar errors carry std strings"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use parking_lot::Mutex;
use vize_carton::config::plugin_plan::document_plugin_plan;
use vize_carton::config::{PluginLintSidecar, SidecarBudget, SidecarReply};
use vize_l0::config::{ConfigDocument, LintRuleSeverity};
use vize_l0::{FxHashMap, FxHashSet, cstr};
use vize_patina::LintDiagnostic;
use vize_patina::plugin_batch::lint::PluginLintJobs;

/// Node startup and config evaluation, then one single-document answer.
const BUDGET: SidecarBudget = SidecarBudget {
    startup: Duration::from_secs(30),
    answer: Duration::from_secs(2),
};
/// Consecutive sidecar failures before the host stops restarting it.
const MAX_FAILURES: u32 = 3;
/// Documents whose last answer is kept.
const ANSWERED_DOCUMENTS: usize = 256;

/// The result of one document's plugin pass.
pub(crate) enum PluginLintOutcome {
    Diagnostics(Vec<LintDiagnostic>),
    /// A newer document version superseded the request.
    Cancelled,
    /// A visible failure: planning, the sidecar, or a rule.
    Failed(String),
}

/// The plugin lint jobs of one config generation and their sidecar.
pub(crate) struct PluginLintHost {
    config: PathBuf,
    jobs: Result<PluginLintJobs, String>,
    /// Rules declared `fixable: "safe"`; their fixes are preferred actions.
    safe_fixes: FxHashSet<&'static str>,
    slot: Mutex<Slot>,
    /// The last diagnostics per filename, with the exact text they answer.
    answered: Mutex<FxHashMap<String, (String, Vec<LintDiagnostic>)>>,
}

#[derive(Default)]
struct Slot {
    sidecar: Option<PluginLintSidecar>,
    failures: u32,
    last_failure: Option<String>,
}

impl PluginLintHost {
    /// `None` when the config enables no plugin rule. Planning failures stay
    /// visible on every linted document.
    pub(crate) fn plan(
        document: &ConfigDocument,
        source_path: Option<&Path>,
        severities: &FxHashMap<vize_l0::String, LintRuleSeverity>,
    ) -> Option<Self> {
        let jobs = match document_plugin_plan(document) {
            Ok(None) => return None,
            Ok(Some(plan)) => match PluginLintJobs::plan(&plan, severities, "lsp") {
                Ok(None) => return None,
                Ok(Some(jobs)) => Ok(jobs),
                Err(error) => Err(error.into_string()),
            },
            Err(error) => Err(cstr!("{error}").into_string()),
        };
        let config = source_path.map(Path::to_path_buf);
        let jobs = jobs.and_then(|jobs| {
            config.as_ref().map(|_| jobs).ok_or_else(|| {
                "[plugin/plan-invalid] plan: plugin lint rules run from vite.config.*, which was not found".to_owned()
            })
        });
        Some(Self {
            config: config.unwrap_or_default(),
            safe_fixes: jobs
                .as_ref()
                .map(PluginLintJobs::safe_fix_ids)
                .unwrap_or_default(),
            jobs,
            slot: Mutex::default(),
            answered: Mutex::default(),
        })
    }

    /// Lint one SFC with every job in one bounded request.
    pub(crate) fn lint(
        &self,
        source: &str,
        filename: &str,
        cancelled: &dyn Fn() -> bool,
    ) -> PluginLintOutcome {
        let jobs = match &self.jobs {
            Ok(jobs) => jobs,
            Err(error) => return PluginLintOutcome::Failed(error.clone()),
        };
        if let Some((text, diagnostics)) = self.answered.lock().get(filename)
            && text == source
        {
            return PluginLintOutcome::Diagnostics(diagnostics.clone());
        }
        let lowered = match jobs.lower(source, filename) {
            Ok(Some(lowered)) => lowered,
            // The plugin records cannot represent this template; `vize lint` skips it too.
            Ok(None) => return PluginLintOutcome::Diagnostics(Vec::new()),
            Err(error) => return PluginLintOutcome::Failed(error.into_string()),
        };
        let request = match jobs.request(&[&lowered]) {
            Ok(request) => request,
            Err(error) => return PluginLintOutcome::Failed(error.into_string()),
        };
        // Requests share one sidecar in order; waiting for it is bounded and
        // cancellable like the request itself.
        let started = std::time::Instant::now();
        let mut slot = loop {
            if let Some(slot) = self.slot.try_lock_for(Duration::from_millis(5)) {
                break slot;
            }
            if cancelled() {
                return PluginLintOutcome::Cancelled;
            }
            if started.elapsed() >= BUDGET.answer {
                return PluginLintOutcome::Failed(
                    "[plugin/timeout] plan: the plugin lint sidecar stayed busy".into(),
                );
            }
        };
        if slot.failures >= MAX_FAILURES {
            return PluginLintOutcome::Failed(slot.last_failure.clone().unwrap_or_default());
        }
        if !slot
            .sidecar
            .as_ref()
            .is_some_and(PluginLintSidecar::is_running)
        {
            match PluginLintSidecar::start(&self.config) {
                Ok(sidecar) => slot.sidecar = Some(sidecar),
                Err(error) => return slot.fail(error),
            }
        }
        let Some(sidecar) = slot.sidecar.as_mut() else {
            return slot.fail("the plugin lint sidecar is unavailable".into());
        };
        match sidecar.request_within(&request, Some(BUDGET), cancelled) {
            Ok(SidecarReply::Answer(answer)) => {
                slot.failures = 0;
                match jobs.answers(&answer, &[&lowered]) {
                    Ok(answers) => {
                        let diagnostics: Vec<_> = answers
                            .into_iter()
                            .flat_map(|answer| answer.documents.into_iter().flatten())
                            .collect();
                        drop(slot);
                        let mut answered = self.answered.lock();
                        if answered.len() >= ANSWERED_DOCUMENTS {
                            answered.clear();
                        }
                        answered.insert(filename.into(), (source.into(), diagnostics.clone()));
                        PluginLintOutcome::Diagnostics(diagnostics)
                    }
                    Err(error) => PluginLintOutcome::Failed(error.into_string()),
                }
            }
            Ok(SidecarReply::Cancelled) => PluginLintOutcome::Cancelled,
            Ok(SidecarReply::TimedOut) => {
                let resolved = slot
                    .sidecar
                    .as_ref()
                    .and_then(PluginLintSidecar::startup)
                    .is_some();
                let (phase, budget) = if resolved {
                    ("plugin lint rules did not answer", BUDGET.answer)
                } else {
                    ("vite.config.* did not resolve its plugins", BUDGET.startup)
                };
                slot.fail(
                    cstr!(
                        "[plugin/timeout] plan: {phase} within {} ms",
                        budget.as_millis()
                    )
                    .into_string(),
                )
            }
            Err(error) => slot.fail(error),
        }
    }
}

impl Slot {
    fn fail(&mut self, error: String) -> PluginLintOutcome {
        self.sidecar = None;
        self.failures += 1;
        self.last_failure = Some(error.clone());
        PluginLintOutcome::Failed(error)
    }
}

impl super::ServerState {
    /// Plugin diagnostics of one Vue document that carry a fix, each with
    /// whether its rule is declared safe.
    pub(crate) fn plugin_lint_fixes(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        content: &str,
    ) -> Vec<(LintDiagnostic, bool)> {
        let Some(host) = self.plugin_lint_for_uri(uri) else {
            return Vec::new();
        };
        let version = self.documents.version(uri);
        let cancelled = || self.documents.version(uri) != version;
        match host.lint(content, uri.path(), &cancelled) {
            PluginLintOutcome::Diagnostics(found) => found
                .into_iter()
                .filter(|diagnostic| diagnostic.fix.is_some())
                .map(|diagnostic| {
                    let safe = host.safe_fixes.contains(diagnostic.rule_name);
                    (diagnostic, safe)
                })
                .collect(),
            PluginLintOutcome::Cancelled | PluginLintOutcome::Failed(_) => Vec::new(),
        }
    }

    /// Add the plugin lint diagnostics of one Vue document. A failure is one
    /// error at the start of the document, never a silent skip.
    pub(crate) fn extend_plugin_lint_diagnostics(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        content: &str,
        diagnostics: &mut Vec<LintDiagnostic>,
    ) {
        let Some(host) = self.plugin_lint_for_uri(uri) else {
            return;
        };
        let version = self.documents.version(uri);
        let cancelled = || self.documents.version(uri) != version;
        match host.lint(content, uri.path(), &cancelled) {
            PluginLintOutcome::Diagnostics(found) => diagnostics.extend(found),
            // The superseding version publishes its own pass.
            PluginLintOutcome::Cancelled => {}
            PluginLintOutcome::Failed(error) => {
                diagnostics.push(LintDiagnostic::error("plugin", error.as_str(), 0, 0));
            }
        }
    }
}
