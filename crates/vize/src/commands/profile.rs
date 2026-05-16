//! Shared CLI profile report rendering.

use std::path::PathBuf;
use std::time::Duration;

use vize_carton::profiler::ProfileSummary;
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
    render_phase_table(&mut out, report);
    render_file_table(&mut out, report);
    render_operation_table(&mut out, report);
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
    let (operation_count, operation_total) = report
        .operations
        .map(|summary| {
            summary
                .entries
                .iter()
                .fold((0u64, Duration::ZERO), |(count, total), entry| {
                    (count + entry.count, total + entry.total)
                })
        })
        .unwrap_or((0, Duration::ZERO));

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
        "  #  total       share   breakdown                          size      rate       status       file",
        RESET
    );

    let mut displayed = 0usize;
    for (index, file) in report.files.iter().take(20).enumerate() {
        displayed += 1;
        let is_slow = file.total > report.slow_threshold;
        let color = if is_slow { YELLOW } else { GREEN };
        let status = file_status(file, report.slow_threshold);

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
        "  operation                         count   total       share   avg         min         max       max/avg  status",
        RESET
    );

    let displayed = summary.entries.len().min(48);
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
        write_percent_padded(out, percent_of(entry.total, report.total), 6);
        appends!(out, "  ");
        write_duration_padded(out, entry.average, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.min, 10);
        appends!(out, "  ");
        write_duration_padded(out, entry.max, 10);
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

fn write_bytes(mut out: &mut String, bytes: usize) {
    if bytes >= 1024 * 1024 {
        append!(out, "{:>7.2} MiB", bytes as f64 / 1024.0 / 1024.0);
    } else if bytes >= 1024 {
        append!(out, "{:>7.2} KiB", bytes as f64 / 1024.0);
    } else {
        append!(out, "{:>7} B", bytes);
    }
}

fn write_count_padded(mut out: &mut String, count: u64, width: usize) {
    append!(out, "{count:>width$}");
}

fn write_ratio_padded(mut out: &mut String, ratio: f64, width: usize) {
    append!(out, "{ratio:>width$.1}x");
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

#[cfg(test)]
mod tests {
    use super::{
        ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, render_profile_report,
    };
    use std::path::PathBuf;
    use std::time::Duration;
    use vize_carton::String;
    use vize_carton::profiler::{ProfileEntry, ProfileSummary};

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
                    average: Duration::from_millis(8),
                    min: Duration::from_millis(8),
                    max: Duration::from_millis(8),
                },
                ProfileEntry {
                    name: "atelier.transform.element",
                    count: 24,
                    total: Duration::from_millis(6),
                    average: Duration::from_micros(250),
                    min: Duration::from_micros(100),
                    max: Duration::from_micros(600),
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
            recommendations: &recommendations,
        };

        insta::assert_snapshot!(render_profile_report(&report));
    }
}
