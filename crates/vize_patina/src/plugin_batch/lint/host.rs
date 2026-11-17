//! One host per config generation for per-document plugin lint (#13074):
//! the language server and the Oxlint integration.
//!
//! A config generation that enables plugin rules owns one lazily started
//! sidecar; edits never spawn a process. Each lint sends one document, with
//! every job, as one request. A request is bounded: the sidecar must resolve
//! its config within the startup budget and answer within the answer budget,
//! or it is stopped and restarted on the next lint. A newer document version
//! cancels the wait and the late answer is skipped. After repeated failures
//! the host reports the last one instead of restarting until the config
//! changes. The last answer for each document text is kept, so its
//! diagnostics, hover and code actions cross into JS once. A plan whose only
//! lint work is native (declared directive contracts) never starts a sidecar.

#![expect(
    clippy::disallowed_types,
    reason = "plugin batch specs and sidecar errors carry std strings"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use std::sync::{Mutex, MutexGuard, PoisonError};

use vize_carton::config::plugin_plan::{VerifiedPluginPlan, document_plugin_plan};
use vize_carton::config::{PluginLintSidecar, SidecarBudget, SidecarReply};
use vize_l0::config::{ConfigDocument, LintRuleSeverity};
use vize_l0::{FxHashMap, FxHashSet, cstr};

use super::PluginLintJobs;
use crate::LintDiagnostic;

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
pub enum PluginLintOutcome {
    /// Every job's diagnostics, in job order.
    Diagnostics(Vec<LintDiagnostic>),
    /// A newer document version superseded the request.
    Cancelled,
    /// A visible failure: planning, the sidecar, or a rule.
    Failed(String),
}

/// The plugin lint jobs of one config generation and their sidecar.
pub struct PluginLintHost {
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
    /// `None` when the config adds no plugin lint work. Planning failures
    /// stay visible on every linted document.
    pub fn plan(
        document: &ConfigDocument,
        source_path: Option<&Path>,
        severities: &FxHashMap<vize_l0::String, LintRuleSeverity>,
        product: &str,
    ) -> Option<Self> {
        match document_plugin_plan(document) {
            Ok(None) => None,
            Ok(Some(plan)) => Self::for_plan(&plan, source_path, severities, product),
            Err(error) => Some(Self::new(Err(cstr!("{error}").into_string()), source_path)),
        }
    }

    /// The host of one verified plan; `None` when it adds no lint work.
    pub fn for_plan(
        plan: &VerifiedPluginPlan,
        source_path: Option<&Path>,
        severities: &FxHashMap<vize_l0::String, LintRuleSeverity>,
        product: &str,
    ) -> Option<Self> {
        let jobs = match PluginLintJobs::plan(plan, severities, product) {
            Ok(None) => return None,
            Ok(Some(jobs)) => Ok(jobs),
            Err(error) => Err(error.into_string()),
        };
        // Only JS jobs need the authored config to start a sidecar.
        let jobs = jobs.and_then(|jobs| {
            if source_path.is_none() && jobs.crosses() {
                return Err("[plugin/plan-invalid] plan: plugin lint rules run from vite.config.*, which was not found".to_owned());
            }
            Ok(jobs)
        });
        Some(Self::new(jobs, source_path))
    }

    fn new(jobs: Result<PluginLintJobs, String>, source_path: Option<&Path>) -> Self {
        Self {
            config: source_path.map(Path::to_path_buf).unwrap_or_default(),
            safe_fixes: jobs
                .as_ref()
                .map(PluginLintJobs::safe_fix_ids)
                .unwrap_or_default(),
            jobs,
            slot: Mutex::default(),
            answered: Mutex::default(),
        }
    }

    /// Lint one SFC with every job in one bounded request.
    pub fn lint(
        &self,
        source: &str,
        filename: &str,
        cancelled: &dyn Fn() -> bool,
    ) -> PluginLintOutcome {
        let jobs = match &self.jobs {
            Ok(jobs) => jobs,
            Err(error) => return PluginLintOutcome::Failed(error.clone()),
        };
        if let Some((text, diagnostics)) = locked(&self.answered).get(filename)
            && text == source
        {
            return PluginLintOutcome::Diagnostics(diagnostics.clone());
        }
        let mut lowered = match jobs.lower(source, filename) {
            Ok(Some(lowered)) => lowered,
            // The plugin records cannot represent this template; `vize lint` skips it too.
            Ok(None) => return PluginLintOutcome::Diagnostics(Vec::new()),
            Err(error) => return PluginLintOutcome::Failed(error.into_string()),
        };
        let native = std::mem::take(&mut lowered.native);
        if !jobs.crosses() {
            return PluginLintOutcome::Diagnostics(native);
        }
        let request = match jobs.request(&[&lowered]) {
            Ok(request) => request,
            Err(error) => return PluginLintOutcome::Failed(error.into_string()),
        };
        // Requests share one sidecar in order; waiting for it is bounded and
        // cancellable like the request itself.
        let started = std::time::Instant::now();
        let mut slot = loop {
            match self.slot.try_lock() {
                Ok(slot) => break slot,
                Err(std::sync::TryLockError::Poisoned(poisoned)) => break poisoned.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => {
                    std::thread::sleep(Duration::from_millis(1));
                }
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
                        let diagnostics: Vec<_> = native
                            .into_iter()
                            .chain(
                                answers
                                    .into_iter()
                                    .flat_map(|answer| answer.documents.into_iter().flatten()),
                            )
                            .collect();
                        drop(slot);
                        let mut answered = locked(&self.answered);
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

impl PluginLintHost {
    /// Whether `rule` is declared `fixable: "safe"`.
    pub fn is_safe_fix(&self, rule: &str) -> bool {
        self.safe_fixes.contains(rule)
    }
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
