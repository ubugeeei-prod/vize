//! Profile report rendering for local CLI tools.

mod audit;
mod format;
mod tables;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::time::Duration;

use vize_carton::profiler::{AllocationSnapshot, CounterSummary, ProfileSummary};
use vize_carton::{String, appendln, appends};

use self::audit::{render_allocation_table, render_strict_audit};
use self::format::{write_duration, write_throughput};
use self::tables::{
    render_call_volume_table, render_counter_table, render_file_table, render_latency_table,
    render_operation_table, render_phase_table, render_recommendations,
};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[90m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";

#[derive(Debug, Clone, Copy)]
pub enum ProfilePhaseKind {
    Wall,
    Cumulative,
}

#[derive(Debug, Clone, Copy)]
pub struct ProfilePhase {
    pub name: &'static str,
    pub duration: Duration,
    pub kind: ProfilePhaseKind,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub struct ProfileFileRow {
    pub path: PathBuf,
    pub bytes: usize,
    pub total: Duration,
    pub primary_label: &'static str,
    pub primary: Duration,
    pub secondary_label: &'static str,
    pub secondary: Duration,
    pub note: Option<String>,
}

pub struct ProfileReport<'a> {
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

pub fn print_profile_report(report: &ProfileReport<'_>) {
    eprint!("{}", render_profile_report(report));
}

pub fn render_profile_report(report: &ProfileReport<'_>) -> String {
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
