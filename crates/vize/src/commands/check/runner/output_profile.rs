use std::time::Duration;

use vize_carton::{String, cstr, profiler::global_profiler};
use vize_curator::profile::{ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report};

use super::{DeclarationSummary, ProgramExecution};
use crate::profile_support;

#[allow(clippy::too_many_arguments)]
pub(super) fn print_profile(
    executions: &[ProgramExecution],
    virtual_files: &[&vize_canon::VirtualFile],
    total_errors: usize,
    total_time: Duration,
    collect_time: Duration,
    import_time: Duration,
    gen_time: Duration,
    check_time: Duration,
    profile_artifact_time: Duration,
    diagnostics_render_time: Duration,
    emitted: Option<&DeclarationSummary>,
) {
    let profiler = global_profiler();
    let allocation_summary = profile_support::allocation_snapshot();
    let counter_summary = profiler.counter_summary();
    let operation_summary = profiler.summary();
    profiler.disable();
    let mut phases = vec![
        ProfilePhase {
            name: "collect inputs",
            duration: collect_time,
            kind: ProfilePhaseKind::Wall,
            note: "tsconfig or explicit patterns",
        },
        ProfilePhase {
            name: "resolve imports",
            duration: import_time,
            kind: ProfilePhaseKind::Wall,
            note: "transitive local and package graph",
        },
        ProfilePhase {
            name: "virtual project",
            duration: gen_time,
            kind: ProfilePhaseKind::Wall,
            note: "scan paths and generate Virtual TS",
        },
        ProfilePhase {
            name: "profile artifacts",
            duration: profile_artifact_time,
            kind: ProfilePhaseKind::Wall,
            note: "write node_modules/.vize/check-profile",
        },
        ProfilePhase {
            name: "corsa diagnostics",
            duration: check_time,
            kind: ProfilePhaseKind::Wall,
            note: "project-session diagnostics",
        },
        ProfilePhase {
            name: "render diagnostics",
            duration: diagnostics_render_time,
            kind: ProfilePhaseKind::Wall,
            note: "group diagnostics by file",
        },
    ];
    if let Some(summary) = emitted {
        phases.push(ProfilePhase {
            name: "declaration emit",
            duration: summary.elapsed,
            kind: ProfilePhaseKind::Wall,
            note: "materialized Corsa project",
        });
    }
    let virtual_bytes = virtual_files.iter().map(|file| file.content.len()).sum();
    let mut recommendations: Vec<String> = Vec::new();
    if check_time > gen_time * 2 {
        recommendations.push(
            "Corsa diagnostics dominate; inspect the largest generated virtual files.".into(),
        );
    } else if gen_time > check_time {
        recommendations.push(
            "Virtual TS generation dominates; inspect SFCs with large templates or cross-file imports."
                .into(),
        );
    }
    if let Some(largest) = virtual_files.iter().max_by_key(|file| file.content.len()) {
        recommendations.push(cstr!(
            "Largest Virtual TS: {} ({} bytes).",
            largest.original_path.display(),
            largest.content.len()
        ));
    }
    let summary = cstr!(
        "{} virtual file(s), {} error(s), {} tsconfig program(s)",
        virtual_files.len(),
        total_errors,
        executions.len()
    );
    print_profile_report(&ProfileReport {
        title: "check",
        summary: summary.as_str(),
        total: total_time,
        phases: &phases,
        files: &[],
        slow_threshold: Duration::from_millis(0),
        throughput_bytes: Some(virtual_bytes),
        operations: Some(&operation_summary),
        counters: Some(&counter_summary),
        allocations: allocation_summary,
        recommendations: &recommendations,
    });
}
