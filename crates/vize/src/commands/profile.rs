//! Shared CLI profile report rendering.

use std::path::PathBuf;
use std::time::Duration;

use vize_carton::profiler::{AllocationSnapshot, CounterSummary, ProfileSummary};
use vize_carton::{String, append, appendln, appends};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[90m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProfilePhaseKind {
    Wall,
    Cumulative,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProfilePhase {
    pub name: &'static str,
    pub duration: Duration,
    pub kind: ProfilePhaseKind,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct ProfileFileRow {
    pub path: PathBuf,
    pub bytes: usize,
    pub total: Duration,
    pub primary_label: &'static str,
    pub primary: Duration,
    pub secondary_label: &'static str,
    pub secondary: Duration,
    pub note: Option<String>,
}

pub(crate) struct ProfileReport<'a> {
    pub title: &'a str,
    pub summary: &'a str,
    pub total: Duration,
    pub phases: &'a [ProfilePhase],
    pub files: &'a [ProfileFileRow],
    pub slow_threshold: Duration,
    pub throughput_bytes: Option<usize>,
    pub operations: Option<&'a ProfileSummary>,
    pub counters: Option<&'a CounterSummary>,
    pub allocations: Option<AllocationSnapshot>,
    pub recommendations: &'a [String],
}

pub(crate) fn print_profile_report(report: &ProfileReport<'_>) {
    eprint!("{}", render_profile_report(report));
}

pub(crate) fn render_profile_report(report: &ProfileReport<'_>) -> String {
    let mut out = String::default();

    appendln!(out);
    appendln!(out, BOLD, CYAN, "Vize profile: ", report.title, RESET);
    if !report.summary.is_empty() {
        appendln!(out, DIM, "  ", report.summary, RESET);
    }
    appendln!(out);

    appends!(out, BOLD, "Total wall time", RESET, ": ");
    write_duration(&mut out, report.total);
    if let Some(bytes) = report.throughput_bytes {
        appends!(out, DIM, "  throughput ");
        write_throughput(&mut out, bytes, report.total);
        appends!(out, RESET);
    }
    out.push('\n');

    render_strict_audit(&mut out, report);
    render_allocation_table(&mut out, report);
    render_counter_table(&mut out, report, "I/O counters", "io.");
    render_counter_table(&mut out, report, "System calls", "syscall.");
    render_phase_table(&mut out, report);
    render_file_table(&mut out, report);
    render_operation_table(&mut out, report);
    render_latency_table(&mut out, report);
    render_call_volume_table(&mut out, report);
    render_recommendations(&mut out, report);

    out
}

fn render_strict_audit(out: &mut String, report: &ProfileReport<'_>) {
    let wall_tracked = report
        .phases
        .iter()
        .filter(|phase| matches!(phase.kind, ProfilePhaseKind::Wall))
        .fold(Duration::ZERO, |acc, phase| acc + phase.duration);
    let cumulative_tracked = report
        .phases
        .iter()
        .filter(|phase| matches!(phase.kind, ProfilePhaseKind::Cumulative))
        .fold(Duration::ZERO, |acc, phase| acc + phase.duration);
    let untracked_wall = report.total.saturating_sub(wall_tracked);
    let slow_file_count = report
        .files
        .iter()
        .filter(|file| file.total > report.slow_threshold)
        .count();
    let (operation_count, operation_total, operation_self, operation_child) = report
        .operations
        .map(|summary| {
            summary.entries.iter().fold(
                (0u64, Duration::ZERO, Duration::ZERO, Duration::ZERO),
                |(count, total, self_total, child_total), entry| {
                    (
                        count + entry.count,
                        total + entry.total,
                        self_total + entry.self_total,
                        child_total + entry.child_total,
                    )
                },
            )
        })
        .unwrap_or((0, Duration::ZERO, Duration::ZERO, Duration::ZERO));
    let io_read_bytes = counter_total(report, "io.read.bytes");
    let io_write_bytes = counter_total(report, "io.write.bytes");
    let syscall_calls = counter_total_matching(report, "syscall.", ".calls");
    let syscall_failures = counter_total_matching(report, "syscall.", ".failures");

    appendln!(out);
    appendln!(out, BOLD, "Strict audit", RESET);
    appendln!(
        out,
        DIM,
        "  metric                  value                  status",
        RESET
    );

    let wall_share = percent_of(wall_tracked, report.total);
    let mut value = String::default();
    write_duration(&mut value, wall_tracked);
    append!(value, " ({:.1}%)", wall_share);
    let (status, color) = if wall_share < 80.0 {
        ("profile gap", YELLOW)
    } else if wall_share > 120.0 {
        ("overlapping phases", CYAN)
    } else {
        ("covered", GREEN)
    };
    audit_row(out, "wall accounted", value.as_str(), status, color);

    value.clear();
    write_duration(&mut value, untracked_wall);
    append!(value, " ({:.1}%)", percent_of(untracked_wall, report.total));
    let (status, color) = if percent_of(untracked_wall, report.total) > 5.0 {
        ("unprofiled work", YELLOW)
    } else {
        ("tight", GREEN)
    };
    audit_row(out, "untracked wall", value.as_str(), status, color);

    value.clear();
    write_duration(&mut value, cumulative_tracked);
    append!(
        value,
        " ({:.1}x wall)",
        duration_ratio(cumulative_tracked, report.total)
    );
    let (status, color) = if cumulative_tracked.is_zero() {
        ("none", DIM)
    } else {
        ("parallel/nested", CYAN)
    };
    audit_row(out, "cumulative work", value.as_str(), status, color);

    value.clear();
    append!(value, "{} / {}", slow_file_count, report.files.len());
    let (status, color) = if slow_file_count == 0 {
        ("threshold clear", GREEN)
    } else {
        ("threshold hit", YELLOW)
    };
    audit_row(out, "slow files", value.as_str(), status, color);

    value.clear();
    append!(value, "{operation_count} call(s), ");
    write_duration(&mut value, operation_total);
    let (status, color) = if operation_count == 0 {
        ("not captured", DIM)
    } else {
        ("captured", GREEN)
    };
    audit_row(out, "internal spans", value.as_str(), status, color);

    value.clear();
    write_duration(&mut value, operation_self);
    append!(
        value,
        " ({:.1}%)",
        percent_of(operation_self, operation_total)
    );
    let (status, color) = if operation_count == 0 {
        ("not captured", DIM)
    } else {
        ("exclusive cost", GREEN)
    };
    audit_row(out, "internal self", value.as_str(), status, color);

    value.clear();
    write_duration(&mut value, operation_child);
    append!(
        value,
        " ({:.1}%)",
        percent_of(operation_child, operation_total)
    );
    let (status, color) = if operation_child > operation_self {
        ("nested heavy", YELLOW)
    } else if operation_child.is_zero() {
        ("flat", DIM)
    } else {
        ("nested visible", CYAN)
    };
    audit_row(out, "nested span tax", value.as_str(), status, color);

    if let Some(allocation) = report.allocations {
        value.clear();
        append!(value, "{} call(s), ", allocation.allocation_calls());
        write_bytes_u64(&mut value, allocation.requested_bytes());
        let (status, color) = if allocation.allocation_calls() == 0 {
            ("zero", DIM)
        } else if allocation.allocation_failures() > 0 {
            ("failures", RED)
        } else {
            ("tracked", GREEN)
        };
        audit_row(out, "alloc pressure", value.as_str(), status, color);

        value.clear();
        write_signed_bytes(&mut value, allocation.net_bytes());
        append!(
            value,
            " ({:.1} B/call)",
            allocation.requested_bytes_per_call()
        );
        let (status, color) = if allocation.net_bytes() > 0 {
            ("growth window", YELLOW)
        } else if allocation.net_bytes() < 0 {
            ("release window", CYAN)
        } else {
            ("balanced", GREEN)
        };
        audit_row(out, "heap delta", value.as_str(), status, color);
    }

    if report.counters.is_some() {
        value.clear();
        appends!(value, "read ");
        write_bytes_u64(&mut value, io_read_bytes);
        appends!(value, ", wrote ");
        write_bytes_u64(&mut value, io_write_bytes);
        let (status, color) = if io_read_bytes == 0 && io_write_bytes == 0 {
            ("no bytes", DIM)
        } else {
            ("tracked", GREEN)
        };
        audit_row(out, "I/O bytes", value.as_str(), status, color);

        value.clear();
        append!(
            value,
            "{syscall_calls} call-site hit(s), {syscall_failures} failed"
        );
        let (status, color) = if syscall_calls == 0 {
            ("not captured", DIM)
        } else if syscall_failures > 0 {
            ("failures", RED)
        } else {
            ("clean", GREEN)
        };
        audit_row(out, "syscall sites", value.as_str(), status, color);
    }
}

fn audit_row(out: &mut String, metric: &str, value: &str, status: &str, color: &str) {
    appends!(out, "  ");
    append_padded(out, metric, 23);
    appends!(out, " ");
    append_padded(out, value, 22);
    appends!(out, " ", color);
    append_padded(out, status, 16);
    appendln!(out, RESET);
}

fn render_allocation_table(out: &mut String, report: &ProfileReport<'_>) {
    let Some(allocation) = report.allocations else {
        return;
    };

    appendln!(out);
    appendln!(out, BOLD, "Allocation pressure", RESET);
    appendln!(
        out,
        DIM,
        "  kind             calls       bytes          failures  note",
        RESET
    );

    allocation_row(
        out,
        "alloc",
        allocation.alloc_calls + allocation.alloc_zeroed_calls,
        allocation.alloc_bytes + allocation.alloc_zeroed_bytes,
        allocation.alloc_failures + allocation.alloc_zeroed_failures,
        "fresh allocations",
    );
    allocation_row(
        out,
        "realloc in",
        allocation.realloc_calls,
        allocation.realloc_old_bytes,
        allocation.realloc_failures,
        "old layouts replaced",
    );
    allocation_row(
        out,
        "realloc out",
        allocation.realloc_calls,
        allocation.realloc_new_bytes,
        allocation.realloc_failures,
        "new requested sizes",
    );
    allocation_row(
        out,
        "dealloc",
        allocation.dealloc_calls,
        allocation.dealloc_bytes,
        0,
        "released layouts",
    );

    appends!(out, "  ");
    append_padded(out, "net delta", 16);
    appends!(out, " ");
    append_padded(out, "n/a", 10);
    appends!(out, "  ");
    write_signed_bytes_padded(out, allocation.net_bytes(), 13);
    appends!(out, "  ");
    write_count_padded(out, allocation.allocation_failures(), 8);
    appends!(out, "  profile-window requested minus released");
    out.push('\n');
}

fn allocation_row(out: &mut String, kind: &str, calls: u64, bytes: u64, failures: u64, note: &str) {
    appends!(out, "  ");
    append_padded(out, kind, 16);
    appends!(out, " ");
    write_count_padded(out, calls, 10);
    appends!(out, "  ");
    write_bytes_u64_padded(out, bytes, 13);
    appends!(out, "  ");
    write_count_padded(out, failures, 8);
    appends!(out, "  ", DIM, note, RESET);
    out.push('\n');
}

fn render_counter_table(out: &mut String, report: &ProfileReport<'_>, title: &str, prefix: &str) {
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

fn render_phase_table(out: &mut String, report: &ProfileReport<'_>) {
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

fn render_file_table(mut out: &mut String, report: &ProfileReport<'_>) {
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

fn render_operation_table(out: &mut String, report: &ProfileReport<'_>) {
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

fn render_latency_table(out: &mut String, report: &ProfileReport<'_>) {
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

fn render_call_volume_table(out: &mut String, report: &ProfileReport<'_>) {
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

fn render_recommendations(out: &mut String, report: &ProfileReport<'_>) {
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

fn append_padded(mut out: &mut String, value: &str, width: usize) {
    append!(out, "{}", value);
    let len = value.chars().count();
    if len < width {
        for _ in 0..(width - len) {
            out.push(' ');
        }
    }
}

fn write_duration(mut out: &mut String, duration: Duration) {
    append!(out, "{:.3}ms", duration.as_secs_f64() * 1000.0);
}

fn write_duration_padded(out: &mut String, duration: Duration, width: usize) {
    let before = out.len();
    write_duration(out, duration);
    let written = out.len() - before;
    if written < width {
        let value = out.split_off(before);
        for _ in 0..(width - written) {
            out.push(' ');
        }
        out.push_str(value.as_str());
    }
}

fn write_percent(mut out: &mut String, percent: f64) {
    append!(out, "{:>5.1}%", percent);
}

fn write_percent_padded(mut out: &mut String, percent: f64, width: usize) {
    append!(out, "{percent:>width$.1}%");
}

fn write_throughput(mut out: &mut String, bytes: usize, duration: Duration) {
    let seconds = duration.as_secs_f64();
    if seconds <= f64::EPSILON {
        append!(out, "n/a");
        return;
    }
    let kb_per_sec = bytes as f64 / 1024.0 / seconds;
    append!(out, "{:.2} KiB/s", kb_per_sec);
}

fn write_rate_padded(out: &mut String, bytes: usize, duration: Duration, width: usize) {
    let before = out.len();
    write_rate(out, bytes, duration);
    let written = out.len() - before;
    if written < width {
        let value = out.split_off(before);
        for _ in 0..(width - written) {
            out.push(' ');
        }
        out.push_str(value.as_str());
    }
}

fn write_rate(mut out: &mut String, bytes: usize, duration: Duration) {
    let seconds = duration.as_secs_f64();
    if seconds <= f64::EPSILON {
        appends!(out, "n/a");
        return;
    }

    let kib_per_sec = bytes as f64 / 1024.0 / seconds;
    if kib_per_sec >= 1024.0 {
        append!(out, "{:.2} MiB/s", kib_per_sec / 1024.0);
    } else {
        append!(out, "{:.2} KiB/s", kib_per_sec);
    }
}

fn write_bytes(out: &mut String, bytes: usize) {
    write_bytes_u64(out, bytes as u64);
}

fn write_bytes_u64(mut out: &mut String, bytes: u64) {
    if bytes >= 1024 * 1024 {
        append!(out, "{:>7.2} MiB", bytes as f64 / 1024.0 / 1024.0);
    } else if bytes >= 1024 {
        append!(out, "{:>7.2} KiB", bytes as f64 / 1024.0);
    } else {
        append!(out, "{:>7} B", bytes);
    }
}

fn write_bytes_u64_padded(out: &mut String, bytes: u64, width: usize) {
    let before = out.len();
    write_bytes_u64(out, bytes);
    pad_recent(out, before, width);
}

fn write_signed_bytes(out: &mut String, bytes: i128) {
    if bytes < 0 {
        appends!(out, "-");
        write_bytes_u64(out, bytes.unsigned_abs() as u64);
    } else {
        write_bytes_u64(out, bytes as u64);
    }
}

fn write_signed_bytes_padded(out: &mut String, bytes: i128, width: usize) {
    let before = out.len();
    write_signed_bytes(out, bytes);
    pad_recent(out, before, width);
}

fn write_count_padded(mut out: &mut String, count: u64, width: usize) {
    append!(out, "{count:>width$}");
}

fn write_ratio_padded(mut out: &mut String, ratio: f64, width: usize) {
    append!(out, "{ratio:>width$.1}x");
}

fn write_calls_per_ms_padded(mut out: &mut String, count: u64, duration: Duration, width: usize) {
    let ms = duration.as_secs_f64() * 1000.0;
    if ms <= f64::EPSILON {
        append!(out, "{:>width$}", "n/a");
    } else {
        append!(out, "{:>width$.2}", count as f64 / ms);
    }
}

fn write_counter_total_padded(out: &mut String, name: &str, value: u64, width: usize) {
    if name.ends_with("bytes") {
        write_bytes_u64_padded(out, value, width);
    } else {
        write_count_padded(out, value, width);
    }
}

fn write_counter_average_padded(mut out: &mut String, name: &str, value: f64, width: usize) {
    if name.ends_with("bytes") {
        if value >= 1024.0 * 1024.0 {
            append!(out, "{:>width$.2} MiB", value / 1024.0 / 1024.0);
        } else if value >= 1024.0 {
            append!(out, "{:>width$.2} KiB", value / 1024.0);
        } else {
            append!(out, "{value:>width$.1} B");
        }
    } else {
        append!(out, "{value:>width$.1}");
    }
}

fn pad_recent(out: &mut String, before: usize, width: usize) {
    let written = out.len() - before;
    if written < width {
        let value = out.split_off(before);
        for _ in 0..(width - written) {
            out.push(' ');
        }
        out.push_str(value.as_str());
    }
}

fn write_bar(out: &mut String, percent: f64) {
    const WIDTH: usize = 18;
    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * WIDTH as f64).round() as usize;
    out.push('[');
    for index in 0..WIDTH {
        out.push(if index < filled { '#' } else { '.' });
    }
    out.push(']');
}

fn percent_of(duration: Duration, total: Duration) -> f64 {
    let total = total.as_secs_f64();
    if total <= f64::EPSILON {
        0.0
    } else {
        duration.as_secs_f64() / total * 100.0
    }
}

fn duration_ratio(duration: Duration, baseline: Duration) -> f64 {
    let baseline = baseline.as_secs_f64();
    if baseline <= f64::EPSILON {
        0.0
    } else {
        duration.as_secs_f64() / baseline
    }
}

fn counter_total(report: &ProfileReport<'_>, name: &str) -> u64 {
    report
        .counters
        .map(|summary| summary.total(name))
        .unwrap_or(0)
}

fn counter_total_matching(report: &ProfileReport<'_>, prefix: &str, suffix: &str) -> u64 {
    report
        .counters
        .map(|summary| summary.total_matching(prefix, suffix))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, render_profile_report,
    };
    use std::path::PathBuf;
    use std::time::Duration;
    use vize_carton::String;
    use vize_carton::profiler::{
        AllocationSnapshot, CounterEntry, CounterSummary, ProfileEntry, ProfileSummary,
    };

    #[test]
    #[allow(clippy::disallowed_macros)]
    fn profile_report_snapshot() {
        let phases = [
            ProfilePhase {
                name: "collect files",
                duration: Duration::from_millis(10),
                kind: ProfilePhaseKind::Wall,
                note: "walked inputs",
            },
            ProfilePhase {
                name: "compile total",
                duration: Duration::from_millis(42),
                kind: ProfilePhaseKind::Cumulative,
                note: "rayon sum",
            },
        ];
        let files = [ProfileFileRow {
            path: PathBuf::from("src/App.vue"),
            bytes: 2048,
            total: Duration::from_millis(31),
            primary_label: "parse",
            primary: Duration::from_millis(8),
            secondary_label: "compile",
            secondary: Duration::from_millis(20),
            note: Some(String::from("1 style block")),
        }];
        let recommendations = [String::from(
            "src/App.vue spent most of its time in compile; inspect template complexity.",
        )];
        let operations = ProfileSummary {
            entries: vec![
                ProfileEntry {
                    name: "atelier.sfc.parse",
                    count: 1,
                    total: Duration::from_millis(8),
                    self_total: Duration::from_millis(7),
                    child_total: Duration::from_millis(1),
                    average: Duration::from_millis(8),
                    self_average: Duration::from_millis(7),
                    min: Duration::from_millis(8),
                    max: Duration::from_millis(8),
                    self_min: Duration::from_millis(7),
                    self_max: Duration::from_millis(7),
                    p50: Duration::from_millis(8),
                    p95: Duration::from_millis(8),
                    p99: Duration::from_millis(8),
                    samples_over_1ms: 1,
                    samples_over_10ms: 0,
                    samples_over_100ms: 0,
                },
                ProfileEntry {
                    name: "atelier.transform.element",
                    count: 24,
                    total: Duration::from_millis(6),
                    self_total: Duration::from_millis(6),
                    child_total: Duration::ZERO,
                    average: Duration::from_micros(250),
                    self_average: Duration::from_micros(250),
                    min: Duration::from_micros(100),
                    max: Duration::from_micros(600),
                    self_min: Duration::from_micros(100),
                    self_max: Duration::from_micros(600),
                    p50: Duration::from_micros(256),
                    p95: Duration::from_micros(512),
                    p99: Duration::from_micros(1024),
                    samples_over_1ms: 0,
                    samples_over_10ms: 0,
                    samples_over_100ms: 0,
                },
            ],
        };
        let counters = CounterSummary {
            entries: vec![
                CounterEntry {
                    name: "io.read.bytes",
                    samples: 1,
                    total: 2048,
                    average: 2048.0,
                    min: 2048,
                    max: 2048,
                },
                CounterEntry {
                    name: "io.read.calls",
                    samples: 1,
                    total: 1,
                    average: 1.0,
                    min: 1,
                    max: 1,
                },
                CounterEntry {
                    name: "syscall.fs.read_to_string.calls",
                    samples: 1,
                    total: 1,
                    average: 1.0,
                    min: 1,
                    max: 1,
                },
            ],
        };
        let report = ProfileReport {
            title: "build",
            summary: "1 file on 4 threads",
            total: Duration::from_millis(50),
            phases: &phases,
            files: &files,
            slow_threshold: Duration::from_millis(30),
            throughput_bytes: Some(2048),
            operations: Some(&operations),
            counters: Some(&counters),
            allocations: Some(AllocationSnapshot {
                alloc_calls: 40,
                alloc_zeroed_calls: 2,
                alloc_failures: 0,
                alloc_zeroed_failures: 0,
                alloc_bytes: 65_536,
                alloc_zeroed_bytes: 4096,
                dealloc_calls: 35,
                dealloc_bytes: 49_152,
                realloc_calls: 6,
                realloc_failures: 0,
                realloc_old_bytes: 12_288,
                realloc_new_bytes: 24_576,
            }),
            recommendations: &recommendations,
        };

        insta::assert_snapshot!(render_profile_report(&report));
    }
}
