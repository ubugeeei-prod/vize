//! Strict audit summary and allocation-pressure renderers.

use std::time::Duration;

use vize_carton::{String, append, appendln, appends};

use super::format::{
    append_padded, counter_total, counter_total_matching, duration_ratio, percent_of,
    write_bytes_u64, write_bytes_u64_padded, write_count_padded, write_duration,
    write_signed_bytes, write_signed_bytes_padded,
};
use super::{BOLD, CYAN, DIM, GREEN, ProfilePhaseKind, ProfileReport, RED, RESET, YELLOW};

pub(super) fn render_strict_audit(out: &mut String, report: &ProfileReport<'_>) {
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

pub(super) fn render_allocation_table(out: &mut String, report: &ProfileReport<'_>) {
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
