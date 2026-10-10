//! Assemble the lint command's existing profile report.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use super::LintArgs;
use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};
use vize_l0::profiler::{AllocationSnapshot, CounterSummary, ProfileSummary};
use vize_l0::{String, cstr};

pub(super) struct ProfileContext<'a> {
    pub(super) args: &'a LintArgs,
    pub(super) profile_rows: Option<Mutex<Vec<ProfileFileRow>>>,
    pub(super) collect_time: Duration,
    pub(super) lint_time: Duration,
    pub(super) cross_file_enabled: bool,
    pub(super) cross_file_time: Duration,
    pub(super) output_time: Duration,
    pub(super) elapsed: Duration,
    pub(super) operation_summary: Option<ProfileSummary>,
    pub(super) counter_summary: Option<CounterSummary>,
    pub(super) allocation_summary: Option<AllocationSnapshot>,
    pub(super) files: &'a [PathBuf],
    pub(super) total_errors: usize,
    pub(super) total_warnings: usize,
    pub(super) preset_name: &'a str,
}

impl ProfileContext<'_> {
    pub(super) fn print(self) {
        let Self {
            args,
            profile_rows,
            collect_time,
            lint_time,
            cross_file_enabled,
            cross_file_time,
            output_time,
            elapsed,
            operation_summary,
            counter_summary,
            allocation_summary,
            files,
            total_errors,
            total_warnings,
            preset_name,
        } = self;
        let mut file_rows = profile_rows
            .and_then(|profile_rows| profile_rows.into_inner().ok())
            .unwrap_or_default();
        file_rows.sort_by_key(|row| std::cmp::Reverse(row.total));

        let total_read = file_rows
            .iter()
            .fold(Duration::ZERO, |acc, row| acc + row.primary);
        let total_lint = file_rows
            .iter()
            .fold(Duration::ZERO, |acc, row| acc + row.secondary);
        let total_bytes = file_rows.iter().fold(0usize, |acc, row| acc + row.bytes);
        let mut phases = vec![
            ProfilePhase {
                name: "collect files",
                duration: collect_time,
                kind: ProfilePhaseKind::Wall,
                note: "glob and ignore-aware walk",
            },
            ProfilePhase {
                name: "lint wall",
                duration: lint_time,
                kind: ProfilePhaseKind::Wall,
                note: "parallel worker elapsed time",
            },
            ProfilePhase {
                name: "read total",
                duration: total_read,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
            ProfilePhase {
                name: "lint total",
                duration: total_lint,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
        ];
        if cross_file_enabled {
            phases.push(ProfilePhase {
                name: "cross-file lint",
                duration: cross_file_time,
                kind: ProfilePhaseKind::Wall,
                note: "project graph diagnostics",
            });
        }
        phases.push(ProfilePhase {
            name: "render output",
            duration: output_time,
            kind: ProfilePhaseKind::Wall,
            note: "diagnostic formatting",
        });
        let slow_threshold = Duration::from_millis(args.slow_threshold);
        let mut recommendations: Vec<String> = Vec::new();
        if let Some(summary) = operation_summary.as_ref()
            && let Some(entry) = summary.entries.first()
        {
            recommendations.push(cstr!(
                "Deepest hot operation: {} took {:.2}ms total across {} call(s).",
                entry.name,
                entry.total.as_secs_f64() * 1000.0,
                entry.count
            ));
        }
        for row in file_rows
            .iter()
            .filter(|row| row.total > slow_threshold)
            .take(4)
        {
            recommendations.push(cstr!(
                "{} exceeded the slow threshold; start with the lint rule preset and script/template size.",
                row.path.display()
            ));
        }
        if output_time > lint_time {
            recommendations.push(
                "Output rendering is heavier than linting; use --quiet during profiling runs that only need totals."
                    .into(),
            );
        }

        let summary = cstr!(
            "{} file(s), {} error(s), {} warning(s), preset '{}'",
            files.len(),
            total_errors,
            total_warnings,
            preset_name
        );
        let report = ProfileReport {
            title: "lint",
            summary: summary.as_str(),
            total: elapsed,
            phases: phases.as_slice(),
            files: &file_rows,
            slow_threshold,
            throughput_bytes: Some(total_bytes),
            operations: operation_summary.as_ref(),
            counters: counter_summary.as_ref(),
            allocations: allocation_summary,
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }
}
