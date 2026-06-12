//! Setup context violation detection.
//!
//! Detects Vue APIs called outside of setup context, which can cause:
//! - CSRP (Cross-request State Pollution) in SSR
//! - Memory leaks from watchers/effects not being cleaned up
//! - Runtime errors from invalid API usage

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::CompactString;
use vize_carton::cstr;
use vize_croquis::setup_context::{
    SetupContextViolation, SetupContextViolationKind, ViolationSeverity,
};

/// A detected setup context issue with file context.
#[derive(Debug, Clone)]
pub struct SetupContextIssue {
    /// File where the issue occurs.
    pub file_id: FileId,
    /// Kind of violation.
    pub kind: SetupContextViolationKind,
    /// The API name that was called.
    pub api_name: CompactString,
    /// Offset in source.
    pub offset: u32,
    /// End offset in source.
    pub end: u32,
}

/// Analyze setup context violations across all files.
pub fn analyze_setup_context(
    registry: &ModuleRegistry,
    _graph: &DependencyGraph,
) -> (Vec<SetupContextIssue>, Vec<CrossFileDiagnostic>) {
    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();

    // Check all registered files for setup context violations
    // This includes both Vue SFCs (non-setup <script> blocks) and external scripts
    for entry in registry.iter() {
        let analysis = &entry.analysis;
        let file_id = entry.id;

        for violation in analysis.setup_context.violations() {
            let diag = create_diagnostic(file_id, violation);
            diagnostics.push(diag);

            issues.push(SetupContextIssue {
                file_id,
                kind: violation.kind,
                api_name: violation.api_name.clone(),
                offset: violation.start,
                end: violation.end,
            });
        }

        diagnostics.extend(lifecycle_pair_diagnostics(file_id, analysis));
    }

    (issues, diagnostics)
}

fn lifecycle_pair_diagnostics(
    file_id: FileId,
    analysis: &vize_croquis::Croquis,
) -> Vec<CrossFileDiagnostic> {
    let mut mounted_offset = None;
    let mut before_mount_offset = None;
    let mut activated_offset = None;
    let mut has_unmounted = false;
    let mut has_before_unmount = false;
    let mut has_deactivated = false;

    for scope in analysis.scopes.iter() {
        let vize_croquis::ScopeData::ClientOnly(data) = scope.data() else {
            continue;
        };

        match data.hook_name.as_str() {
            "onMounted" => {
                mounted_offset.get_or_insert(scope.span.start);
            }
            "onUnmounted" => has_unmounted = true,
            "onBeforeMount" => {
                before_mount_offset.get_or_insert(scope.span.start);
            }
            "onBeforeUnmount" => has_before_unmount = true,
            "onActivated" => {
                activated_offset.get_or_insert(scope.span.start);
            }
            "onDeactivated" => has_deactivated = true,
            _ => {}
        }
    }

    let mut diagnostics = Vec::new();
    push_lifecycle_pair_diagnostic(
        &mut diagnostics,
        file_id,
        mounted_offset.filter(|_| !has_unmounted),
        "onMounted",
        "onUnmounted",
    );
    push_lifecycle_pair_diagnostic(
        &mut diagnostics,
        file_id,
        before_mount_offset.filter(|_| !has_before_unmount),
        "onBeforeMount",
        "onBeforeUnmount",
    );
    push_lifecycle_pair_diagnostic(
        &mut diagnostics,
        file_id,
        activated_offset.filter(|_| !has_deactivated),
        "onActivated",
        "onDeactivated",
    );
    diagnostics
}

fn push_lifecycle_pair_diagnostic(
    diagnostics: &mut Vec<CrossFileDiagnostic>,
    file_id: FileId,
    offset: Option<u32>,
    hook_name: &str,
    cleanup_hook: &str,
) {
    let Some(offset) = offset else {
        return;
    };

    diagnostics.push(
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::LifecycleHookWithoutCleanup {
                hook_name: CompactString::new(hook_name),
                cleanup_hook: CompactString::new(cleanup_hook),
            },
            DiagnosticSeverity::Warning,
            file_id,
            offset,
            cstr!("`{hook_name}()` has no matching `{cleanup_hook}()` cleanup hook"),
        )
        .with_suggestion(cstr!(
            "Register `{cleanup_hook}()` when `{hook_name}()` creates listeners, timers, subscriptions, or activated resources"
        )),
    );
}

/// Create a diagnostic from a setup context violation.
fn create_diagnostic(file_id: FileId, violation: &SetupContextViolation) -> CrossFileDiagnostic {
    let severity = match violation.kind.severity() {
        ViolationSeverity::Error => DiagnosticSeverity::Error,
        ViolationSeverity::Warning => DiagnosticSeverity::Warning,
        ViolationSeverity::Info => DiagnosticSeverity::Info,
    };

    let (message, hint) = match violation.kind {
        SetupContextViolationKind::ModuleLevelState => (
            cstr!(
                "Module-level reactive state (`{}`) causes CSRP in SSR",
                violation.api_name
            ),
            Some(CompactString::new(
                "Move reactive state inside setup() or <script setup> to avoid sharing state across requests",
            )),
        ),
        SetupContextViolationKind::ModuleLevelWatch => (
            cstr!(
                "Module-level `{}()` is never cleaned up, causing memory leaks",
                violation.api_name
            ),
            Some(CompactString::new(
                "Move watch/watchEffect inside setup() where it will be auto-disposed on unmount",
            )),
        ),
        SetupContextViolationKind::ModuleLevelComputed => (
            cstr!(
                "Module-level `{}()` is never cleaned up, causing memory leaks",
                violation.api_name
            ),
            Some(CompactString::new(
                "Move computed inside setup() where it will be auto-disposed on unmount",
            )),
        ),
        SetupContextViolationKind::ModuleLevelProvide => (
            CompactString::new("`provide()` must be called inside setup() or <script setup>"),
            Some(CompactString::new(
                "provide() requires the component instance context which is only available during setup",
            )),
        ),
        SetupContextViolationKind::ModuleLevelInject => (
            CompactString::new("`inject()` must be called inside setup() or <script setup>"),
            Some(CompactString::new(
                "inject() requires the component instance context which is only available during setup",
            )),
        ),
        SetupContextViolationKind::ModuleLevelLifecycle => (
            cstr!(
                "`{}()` must be called inside setup() or <script setup>",
                violation.api_name
            ),
            Some(CompactString::new(
                "Lifecycle hooks require the component instance context which is only available during setup",
            )),
        ),
    };

    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::SetupContextViolation {
            kind: violation.kind,
            api_name: violation.api_name.clone(),
        },
        severity,
        file_id,
        violation.start,
        message,
    )
    .with_end_offset(violation.end);

    if let Some(suggestion) = hint {
        diag.with_suggestion(suggestion)
    } else {
        diag
    }
}

#[cfg(test)]
mod tests {
    use vize_croquis::setup_context::{SetupContextViolationKind, ViolationSeverity};

    #[test]
    fn test_violation_severity_mapping() {
        assert_eq!(
            ViolationSeverity::Error,
            SetupContextViolationKind::ModuleLevelProvide.severity()
        );
        assert_eq!(
            ViolationSeverity::Warning,
            SetupContextViolationKind::ModuleLevelState.severity()
        );
        assert_eq!(
            ViolationSeverity::Warning,
            SetupContextViolationKind::ModuleLevelWatch.severity()
        );
    }
}
