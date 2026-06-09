//! Low-level value formatting and padding helpers for profile reports.

use std::time::Duration;

use vize_carton::{String, append, appends};

use super::ProfileReport;

pub(super) fn append_padded(mut out: &mut String, value: &str, width: usize) {
    append!(out, "{}", value);
    let len = value.chars().count();
    if len < width {
        for _ in 0..(width - len) {
            out.push(' ');
        }
    }
}

pub(super) fn write_duration(mut out: &mut String, duration: Duration) {
    append!(out, "{:.3}ms", duration.as_secs_f64() * 1000.0);
}

pub(super) fn write_duration_padded(out: &mut String, duration: Duration, width: usize) {
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

pub(super) fn write_percent(mut out: &mut String, percent: f64) {
    append!(out, "{:>5.1}%", percent);
}

pub(super) fn write_percent_padded(mut out: &mut String, percent: f64, width: usize) {
    append!(out, "{percent:>width$.1}%");
}

pub(super) fn write_throughput(mut out: &mut String, bytes: usize, duration: Duration) {
    let seconds = duration.as_secs_f64();
    if seconds <= f64::EPSILON {
        append!(out, "n/a");
        return;
    }
    let kb_per_sec = bytes as f64 / 1024.0 / seconds;
    append!(out, "{:.2} KiB/s", kb_per_sec);
}

pub(super) fn write_rate_padded(out: &mut String, bytes: usize, duration: Duration, width: usize) {
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

pub(super) fn write_rate(mut out: &mut String, bytes: usize, duration: Duration) {
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

pub(super) fn write_bytes(out: &mut String, bytes: usize) {
    write_bytes_u64(out, bytes as u64);
}

pub(super) fn write_bytes_u64(mut out: &mut String, bytes: u64) {
    if bytes >= 1024 * 1024 {
        append!(out, "{:>7.2} MiB", bytes as f64 / 1024.0 / 1024.0);
    } else if bytes >= 1024 {
        append!(out, "{:>7.2} KiB", bytes as f64 / 1024.0);
    } else {
        append!(out, "{:>7} B", bytes);
    }
}

pub(super) fn write_bytes_u64_padded(out: &mut String, bytes: u64, width: usize) {
    let before = out.len();
    write_bytes_u64(out, bytes);
    pad_recent(out, before, width);
}

pub(super) fn write_signed_bytes(out: &mut String, bytes: i128) {
    if bytes < 0 {
        appends!(out, "-");
        write_bytes_u64(out, bytes.unsigned_abs() as u64);
    } else {
        write_bytes_u64(out, bytes as u64);
    }
}

pub(super) fn write_signed_bytes_padded(out: &mut String, bytes: i128, width: usize) {
    let before = out.len();
    write_signed_bytes(out, bytes);
    pad_recent(out, before, width);
}

pub(super) fn write_count_padded(mut out: &mut String, count: u64, width: usize) {
    append!(out, "{count:>width$}");
}

pub(super) fn write_ratio_padded(mut out: &mut String, ratio: f64, width: usize) {
    append!(out, "{ratio:>width$.1}x");
}

pub(super) fn write_calls_per_ms_padded(
    mut out: &mut String,
    count: u64,
    duration: Duration,
    width: usize,
) {
    let ms = duration.as_secs_f64() * 1000.0;
    if ms <= f64::EPSILON {
        append!(out, "{:>width$}", "n/a");
    } else {
        append!(out, "{:>width$.2}", count as f64 / ms);
    }
}

pub(super) fn write_counter_total_padded(out: &mut String, name: &str, value: u64, width: usize) {
    if name.ends_with("bytes") {
        write_bytes_u64_padded(out, value, width);
    } else {
        write_count_padded(out, value, width);
    }
}

pub(super) fn write_counter_average_padded(
    mut out: &mut String,
    name: &str,
    value: f64,
    width: usize,
) {
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

pub(super) fn pad_recent(out: &mut String, before: usize, width: usize) {
    let written = out.len() - before;
    if written < width {
        let value = out.split_off(before);
        for _ in 0..(width - written) {
            out.push(' ');
        }
        out.push_str(value.as_str());
    }
}

pub(super) fn write_bar(out: &mut String, percent: f64) {
    const WIDTH: usize = 18;
    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * WIDTH as f64).round() as usize;
    out.push('[');
    for index in 0..WIDTH {
        out.push(if index < filled { '#' } else { '.' });
    }
    out.push(']');
}

pub(super) fn percent_of(duration: Duration, total: Duration) -> f64 {
    let total = total.as_secs_f64();
    if total <= f64::EPSILON {
        0.0
    } else {
        duration.as_secs_f64() / total * 100.0
    }
}

pub(super) fn duration_ratio(duration: Duration, baseline: Duration) -> f64 {
    let baseline = baseline.as_secs_f64();
    if baseline <= f64::EPSILON {
        0.0
    } else {
        duration.as_secs_f64() / baseline
    }
}

pub(super) fn counter_total(report: &ProfileReport<'_>, name: &str) -> u64 {
    report
        .counters
        .map(|summary| summary.total(name))
        .unwrap_or(0)
}

pub(super) fn counter_total_matching(
    report: &ProfileReport<'_>,
    prefix: &str,
    suffix: &str,
) -> u64 {
    report
        .counters
        .map(|summary| summary.total_matching(prefix, suffix))
        .unwrap_or(0)
}
