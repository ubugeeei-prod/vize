//! Section renderers for counters, phases, files, operations, and recommendations.

use std::time::Duration;

use vize_carton::{String, append, appendln, appends};

use super::format::{
    append_padded, duration_ratio, percent_of, write_bar, write_bytes, write_calls_per_ms_padded,
    write_count_padded, write_counter_average_padded, write_counter_total_padded, write_duration,
    write_duration_padded, write_percent, write_percent_padded, write_rate_padded,
    write_ratio_padded,
};
use super::{
    BOLD, CYAN, DIM, GREEN, ProfileFileRow, ProfilePhaseKind, ProfileReport, RED, RESET, YELLOW,
};

pub(super) fn render_counter_table(
    out: &mut String,
    report: &ProfileReport<'_>,
    title: &str,
    prefix: &str,
) {
    let Some(summary) = report.counters else {
        return;
    };

    let entries: Vec<_> = summary
        .entries
        .iter()
        .filter(|entry| entry.name.starts_with(prefix))
        .collect();
    if entries.is_empty() {
        return;
    }

    appendln!(out);
    appendln!(out, BOLD, title, RESET);
    if prefix == "syscall." {
        appendln!(
            out,
            DIM,
            "  std::fs call-site hits tracked by Vize; this is not a kernel trace.",
            RESET
        );
    }
    appendln!(
        out,
        DIM,
        "  counter                                samples    total        avg         min         max",
        RESET
    );

    for entry in entries {
        appends!(out, "  ");
        append_padded(out, entry.name, 38);
        appends!(out, " ");
        write_count_padded(out, entry.samples, 7);
        appends!(out, "  ");
        write_counter_total_padded(out, entry.name, entry.total, 10);
        appends!(out, "  ");
        write_counter_average_padded(out, entry.name, entry.average, 10);
        appends!(out, "  ");
        write_counter_total_padded(out, entry.name, entry.min, 10);
        appends!(out, "  ");
        write_counter_total_padded(out, entry.name, entry.max, 10);
        out.push('\n');
    }
}

pub(super) fn render_phase_table(out: &mut String, report: &ProfileReport<'_>) {
    if report.phases.is_empty() {
        return;
    }

    appendln!(out);
    appendln!(out, BOLD, "Timing breakdown", RESET);
    appendln!(
        out,
        DIM,
        "  phase                         time        share  kind        note",
        RESET
    );

    for phase in report.phases {
        appends!(out, "  ");
        append_padded(out, phase.name, 28);
        appends!(out, " ");
        write_duration_padded(out, phase.duration, 10);
        appends!(out, "  ");

        match phase.kind {
            ProfilePhaseKind::Wall => {
                let percent = percent_of(phase.duration, report.total);
                write_percent(out, percent);
                appends!(out, "  wall        ");
                write_bar(out, percent);
            }
            ProfilePhaseKind::Cumulative => {
                appends!(out, "    n/a  cumulative  ");
                write_bar(out, 0.0);
            }
        }

        if !phase.note.is_empty() {
            appends!(out, "  ", DIM, phase.note, RESET);
        }
        out.push('\n');
    }
}

pub(super) fn render_file_table(mut out: &mut String, report: &ProfileReport<'_>) {
    if report.files.is_empty() {
        return;
    }

    appendln!(out);
    appendln!(out, BOLD, "Hot files", RESET);
    appends!(out, DIM, "  slow threshold: ");
    write_duration(out, report.slow_threshold);
    appendln!(out, RESET);
    appendln!(
        out,
        DIM,
        "  #  total       share   breakdown                                      gap        size      rate       status       file",
        RESET
    );

    let mut displayed = 0usize;
    for (index, file) in report.files.iter().take(20).enumerate() {
        displayed += 1;
        let is_slow = file.total > report.slow_threshold;
        let color = if is_slow { YELLOW } else { GREEN };
        let status = file_status(file, report.slow_threshold);
        let accounted = file.primary + file.secondary;
        let gap = file.total.saturating_sub(accounted);

        appends!(out, "  ");
        write_count_padded(out, (index + 1) as u64, 2);
        appends!(out, "  ", color);
        write_duration_padded(out, file.total, 10);
        appends!(out, RESET, "  ");
        write_percent_padded(out, percent_of(file.total, report.total), 6);
        appends!(out, "  ");
        append_padded(out, file.primary_label, 7);
        appends!(out, " ");
        write_duration_padded(out, file.primary, 9);
        appends!(out, " ");
        write_percent_padded(out, percent_of(file.primary, file.total), 6);
        appends!(out, "  ");
        append_padded(out, file.secondary_label, 7);
        appends!(out, " ");
        write_duration_padded(out, file.secondary, 9);
        appends!(out, " ");
        write_percent_padded(out, percent_of(file.secondary, file.total), 6);
        appends!(out, "  ");
        write_duration_padded(out, gap, 9);
        appends!(out, " ");
        write_percent_padded(out, percent_of(gap, file.total), 6);
        appends!(out, "  ");
        write_bytes(out, file.bytes);
        appends!(out, "  ");
        write_rate_padded(out, file.bytes, file.total, 9);
        appends!(out, "  ");
        appends!(out, color);
        append_padded(out, status.as_str(), 12);
        appends!(out, RESET, " ");
        append!(out, "{}", file.path.display());

        if let Some(note) = file.note.as_ref()
            && !note.is_empty()
        {
            appends!(out, DIM, "  ", note.as_str(), RESET);
        }
        out.push('\n');
    }

    if report.files.len() > displayed {
        appendln!(
            out,
            DIM,
            "  ... ",
            @(report.files.len() - displayed),
            " more file(s)",
            RESET
        );
    }
}

pub(super) fn render_operation_table(out: &mut String, report: &ProfileReport<'_>) {
    let Some(summary) = report.operations else {
        return;
    };
    if summary.entries.is_empty() {
        return;
    }

    appendln!(out);
    appendln!(out, BOLD, "Internal operations", RESET);
    appendln!(
        out,
        DIM,
        "  operation                         count   total       self        child       wall%   self%   avg         self/call   max/avg  status",
        RESET
    );

    let displayed = summary.entries.len().min(64);
    for entry in summary.entries.iter().take(displayed) {
        let max_avg_ratio = duration_ratio(entry.max, entry.average);
        let (status, color) = operation_status(entry, report.total, max_avg_ratio);

        appends!(out, "  ");
        append_padded(out, entry.name, 33);
        appends!(out, " ");
        write_count_padded(out, entry.count, 5);
        appends!(out, "  ");
        write_duration_padded(out, entry.total, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.self_total, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.child_total, 10);
        appends!(out, "  ");
        write_percent_padded(out, percent_of(entry.total, report.total), 6);
        appends!(out, "  ");
        write_percent_padded(out, percent_of(entry.self_total, report.total), 6);
        appends!(out, "  ");
        write_duration_padded(out, entry.average, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.self_average, 10);
        appends!(out, "  ");
        write_ratio_padded(out, max_avg_ratio, 7);
        appends!(out, "  ", color);
        append_padded(out, status, 8);
        appends!(out, RESET);
        out.push('\n');
    }

    if summary.entries.len() > displayed {
        appendln!(
            out,
            DIM,
            "  ... ",
            @(summary.entries.len() - displayed),
            " more operation(s)",
            RESET
        );
    }
}

pub(super) fn render_latency_table(out: &mut String, report: &ProfileReport<'_>) {
    let Some(summary) = report.operations else {
        return;
    };
    if summary.entries.is_empty() {
        return;
    }

    let mut entries: Vec<_> = summary.entries.iter().collect();
    entries.sort_by_key(|entry| std::cmp::Reverse((entry.p99, entry.max, entry.total)));

    appendln!(out);
    appendln!(out, BOLD, "Tail latency", RESET);
    appendln!(
        out,
        DIM,
        "  operation                         count   p50         p95         p99         min         max         >=1ms  >=10ms >=100ms",
        RESET
    );

    for entry in entries.into_iter().take(32) {
        appends!(out, "  ");
        append_padded(out, entry.name, 33);
        appends!(out, " ");
        write_count_padded(out, entry.count, 5);
        appends!(out, "  ");
        write_duration_padded(out, entry.p50, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.p95, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.p99, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.min, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.max, 10);
        appends!(out, "  ");
        write_count_padded(out, entry.samples_over_1ms, 5);
        appends!(out, "  ");
        write_count_padded(out, entry.samples_over_10ms, 5);
        appends!(out, "  ");
        write_count_padded(out, entry.samples_over_100ms, 6);
        out.push('\n');
    }
}

pub(super) fn render_call_volume_table(out: &mut String, report: &ProfileReport<'_>) {
    let Some(summary) = report.operations else {
        return;
    };
    if summary.entries.is_empty() {
        return;
    }

    let mut entries: Vec<_> = summary.entries.iter().collect();
    entries.sort_by_key(|entry| std::cmp::Reverse((entry.count, entry.total)));

    appendln!(out);
    appendln!(out, BOLD, "Call volume", RESET);
    appendln!(
        out,
        DIM,
        "  operation                         count     calls/ms  total       self        avg         self/call",
        RESET
    );

    for entry in entries.into_iter().take(32) {
        appends!(out, "  ");
        append_padded(out, entry.name, 33);
        appends!(out, " ");
        write_count_padded(out, entry.count, 7);
        appends!(out, "  ");
        write_calls_per_ms_padded(out, entry.count, report.total, 9);
        appends!(out, "  ");
        write_duration_padded(out, entry.total, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.self_total, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.average, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.self_average, 10);
        out.push('\n');
    }
}

fn file_status(file: &ProfileFileRow, slow_threshold: Duration) -> String {
    if slow_threshold.is_zero() || file.total <= slow_threshold {
        return String::from("ok");
    }

    let ratio = duration_ratio(file.total, slow_threshold);
    let mut status = String::default();
    append!(status, "SLOW x{ratio:.1}");
    status
}

fn operation_status(
    entry: &vize_carton::profiler::ProfileEntry,
    total: Duration,
    max_avg_ratio: f64,
) -> (&'static str, &'static str) {
    if percent_of(entry.total, total) >= 25.0 {
        ("HOT", YELLOW)
    } else if entry.count >= 3 && entry.max >= Duration::from_millis(1) && max_avg_ratio >= 8.0 {
        ("SPIKE", RED)
    } else {
        ("ok", GREEN)
    }
}

pub(super) fn render_recommendations(out: &mut String, report: &ProfileReport<'_>) {
    appendln!(out);
    appendln!(out, BOLD, "Notes", RESET);

    if report.recommendations.is_empty() {
        appendln!(
            out,
            "  ",
            GREEN,
            "No obvious hot spot crossed the configured threshold.",
            RESET
        );
        appendln!(
            out,
            DIM,
            "  Keep this report around as a baseline before the next performance change.",
            RESET
        );
        return;
    }

    for recommendation in report.recommendations.iter().take(8) {
        appendln!(out, "  ", CYAN, "- ", RESET, recommendation.as_str());
    }
}
