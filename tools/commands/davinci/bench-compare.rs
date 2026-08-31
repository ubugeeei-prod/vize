#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! toml = "0.9"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../support/common.rs"]
mod common;

use regex::Regex;
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};
use toml::Value as TomlValue;

const BUDGET_FIELDS: &[&str] = &["wall_p50_ns", "allocs", "rss_peak_bytes", "wall_tolerance"];
const ALLOCATION_PEAK_PLATFORMS: &[&str] = &["linux", "macos"];
const REPORT_FIELDS: &[&str] = &[
    "alloc_bytes_peak",
    "allocs",
    "bench_id",
    "fixture",
    "harness_version",
    "platform",
    "rss_peak_bytes",
    "wall_ns",
];
const WALL_NS_FIELDS: &[&str] = &["p50", "p95"];

#[derive(Clone, Debug)]
struct Options {
    budgets: PathBuf,
    baseline: PathBuf,
    results: PathBuf,
    benches: Vec<String>,
    update_baseline: bool,
}

#[derive(Clone, Debug)]
struct Budget {
    wall_p50_ns: u64,
    allocs: Option<u64>,
    wall_tolerance_bp: u64,
    alloc_bytes_peak_by_platform: BTreeMap<String, Option<u64>>,
}

#[derive(Clone, Debug)]
struct BenchReport {
    file: PathBuf,
    fixture: String,
    platform: String,
    harness_version: String,
    wall_p50: u64,
    allocs: Option<u64>,
    alloc_bytes_peak: Option<u64>,
    rss_peak_bytes: Option<u64>,
}

#[derive(Debug)]
struct ConfigError(String);

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(ConfigError(error)) => {
            eprintln!("bench-compare: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, ConfigError> {
    let root = common::repo_root()
        .or_else(|_| script_repo_root())
        .map_err(ConfigError)?;
    let options = parse_args(&root, env::args().skip(1).collect())?;

    if options.update_baseline && env::var("DAVINCI_BASELINE_REFRESH").ok().as_deref() != Some("1")
    {
        eprint!(
            "bench-compare: refusing --update-baseline without DAVINCI_BASELINE_REFRESH=1.\n\
             The committed baseline is the reference every PR is gated against; refreshing it\n\
             must be a deliberate act on the reference runner, not a side effect. Re-run with\n\
             DAVINCI_BASELINE_REFRESH=1 in the environment to proceed.\n"
        );
        return Ok(2);
    }

    let all_budgets = load_budgets(&options.budgets)?;
    let selected = options
        .benches
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    for id in &selected {
        if !all_budgets.contains_key(id) {
            return Err(ConfigError(format!(
                "--bench {id} has no budgets.toml entry"
            )));
        }
    }
    let budgets = select_budgets(&all_budgets, &selected);
    let current_reports = select_reports(load_reports(&options.results, "current")?, &selected);
    if current_reports.is_empty() {
        return Err(ConfigError(format!(
            "no current bench reports (*.json) under {} - run the davinci benches first (vp bench:davinci)",
            options.results.display()
        )));
    }

    println!(
        "bench-compare: budgets={} baseline={} results={}",
        options.budgets.display(),
        options.baseline.display(),
        options.results.display()
    );

    if options.update_baseline {
        return update_baseline(&options, &budgets, &current_reports);
    }

    let baseline_reports = select_reports(load_reports(&options.baseline, "baseline")?, &selected);
    let verdict = compare(&budgets, &baseline_reports, &current_reports)?;
    for row in &verdict.rows {
        println!("{row}");
    }
    println!(
        "bench-compare: breaches={} gated_ok={} alloc_gated={} registered={}",
        verdict.breaches,
        verdict.gated_ok,
        verdict.alloc_gated,
        budgets.len()
    );
    Ok(if verdict.breaches > 0 { 1 } else { 0 })
}

fn parse_args(root: &Path, argv: Vec<String>) -> Result<Options, ConfigError> {
    let mut options = Options {
        budgets: root.join("davinci-road/plan/budgets.toml"),
        baseline: root.join("tools/benchmarks/results/davinci/baseline"),
        results: root.join("tools/benchmarks/results/davinci"),
        benches: Vec::new(),
        update_baseline: false,
    };
    let mut index = 0;
    while index < argv.len() {
        let arg = &argv[index];
        match arg.as_str() {
            "--update-baseline" => options.update_baseline = true,
            "--bench" => {
                index += 1;
                let value = argv
                    .get(index)
                    .ok_or_else(|| ConfigError("--bench requires a bench id".to_string()))?;
                options.benches.push(value.clone());
            }
            "--budgets" | "--baseline" | "--results" => {
                index += 1;
                let value = argv
                    .get(index)
                    .ok_or_else(|| ConfigError(format!("{arg} requires a path argument")))?;
                let path = PathBuf::from(value);
                match arg.as_str() {
                    "--budgets" => options.budgets = path,
                    "--baseline" => options.baseline = path,
                    "--results" => options.results = path,
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(ConfigError(format!(
                    "unknown argument {arg:?} (expected --budgets/--baseline/--results/--bench/--update-baseline)"
                )));
            }
        }
        index += 1;
    }
    Ok(options)
}

fn load_budgets(path: &Path) -> Result<BTreeMap<String, Budget>, ConfigError> {
    let text = fs::read_to_string(path)
        .map_err(|_| ConfigError(format!("cannot read budgets file {}", path.display())))?;
    let root = toml::from_str::<TomlValue>(&text)
        .map_err(|error| ConfigError(format!("{}: {error}", path.display())))?;
    let bench = root
        .get("bench")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| ConfigError(format!("{}: missing [bench] section", path.display())))?;
    let mut budgets = BTreeMap::new();
    for (id, value) in bench {
        if !is_valid_bench_id(id) {
            return Err(ConfigError(format!(
                "{}: [bench.{id}] is not a valid bench id",
                path.display()
            )));
        }
        let table = value.as_table().ok_or_else(|| {
            ConfigError(format!("{}: [bench.{id}] is not a table", path.display()))
        })?;
        let keys = table.keys().map(String::as_str).collect::<BTreeSet<_>>();
        let expected = BUDGET_FIELDS.iter().copied().collect::<BTreeSet<_>>();
        if keys != expected {
            return Err(ConfigError(format!(
                "{}: [bench.{id}] must have exactly the fields {} (found: {})",
                path.display(),
                BUDGET_FIELDS.join(", "),
                keys.into_iter().collect::<Vec<_>>().join(", ")
            )));
        }
        let wall_p50_ns = toml_nonnegative_integer(path, id, table, "wall_p50_ns")?;
        let allocs = Some(toml_nonnegative_integer(path, id, table, "allocs")?);
        let rss_peak_bytes = toml_nonnegative_integer(path, id, table, "rss_peak_bytes")?;
        let _ = rss_peak_bytes;
        let tolerance = table
            .get("wall_tolerance")
            .and_then(TomlValue::as_float)
            .ok_or_else(|| {
                ConfigError(format!(
                    "{}: [bench.{id}] wall_tolerance must be a number in (0, 1)",
                    path.display()
                ))
            })?;
        if !(tolerance > 0.0 && tolerance < 1.0) {
            return Err(ConfigError(format!(
                "{}: [bench.{id}] wall_tolerance must be a number in (0, 1)",
                path.display()
            )));
        }
        let tolerance_bp = (tolerance * 10000.0).round();
        if (tolerance * 10000.0 - tolerance_bp).abs() > 1e-6 {
            return Err(ConfigError(format!(
                "{}: [bench.{id}] wall_tolerance must be a whole number of basis points",
                path.display()
            )));
        }
        budgets.insert(
            id.clone(),
            Budget {
                wall_p50_ns,
                allocs,
                wall_tolerance_bp: tolerance_bp as u64,
                alloc_bytes_peak_by_platform: BTreeMap::new(),
            },
        );
    }

    if let Some(allocation_peak) = root.get("allocation_peak") {
        let table = allocation_peak.as_table().ok_or_else(|| {
            ConfigError(format!(
                "{}: [allocation_peak] must be a table",
                path.display()
            ))
        })?;
        for (id, value) in table {
            let budget = budgets.get_mut(id).ok_or_else(|| {
                ConfigError(format!(
                    "{}: [allocation_peak] has unknown bench {id}",
                    path.display()
                ))
            })?;
            let platform_table = value.as_table().ok_or_else(|| {
                ConfigError(format!(
                    "{}: [allocation_peak.{id}] must be a platform table",
                    path.display()
                ))
            })?;
            let platforms = platform_table
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let expected = ALLOCATION_PEAK_PLATFORMS
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            if platforms != expected {
                return Err(ConfigError(format!(
                    "{}: [allocation_peak.{id}] must have exactly the platforms {} (found: {})",
                    path.display(),
                    ALLOCATION_PEAK_PLATFORMS.join(", "),
                    platforms.into_iter().collect::<Vec<_>>().join(", ")
                )));
            }
            for (platform, peak) in platform_table {
                let peak = peak
                    .as_integer()
                    .filter(|value| *value >= 0)
                    .ok_or_else(|| {
                        ConfigError(format!(
                            "{}: [allocation_peak.{id}.{platform}] must be a non-negative integer",
                            path.display()
                        ))
                    })? as u64;
                budget
                    .alloc_bytes_peak_by_platform
                    .insert(platform.clone(), Some(peak));
            }
        }
    }
    Ok(budgets)
}

fn toml_nonnegative_integer(
    path: &Path,
    id: &str,
    table: &toml::map::Map<String, TomlValue>,
    field: &str,
) -> Result<u64, ConfigError> {
    table
        .get(field)
        .and_then(TomlValue::as_integer)
        .filter(|value| *value >= 0)
        .map(|value| value as u64)
        .ok_or_else(|| {
            ConfigError(format!(
                "{}: [bench.{id}] {field} must be a non-negative integer",
                path.display()
            ))
        })
}

fn load_reports(dir: &Path, label: &str) -> Result<BTreeMap<String, BenchReport>, ConfigError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => {
            let suffix = if error.raw_os_error() == Some(20) {
                " (not a directory)"
            } else {
                ""
            };
            return Err(ConfigError(format!(
                "{label} reports directory {} cannot be read: {error}{suffix}",
                dir.display()
            )));
        }
    };
    let platform_re = Regex::new(r"^[a-z0-9_]+$").unwrap();
    let mut reports = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            ConfigError(format!(
                "{label} reports directory {} cannot be read: {error}",
                dir.display()
            ))
        })?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let stem = file_name.trim_end_matches(".json").to_string();
        let text = fs::read_to_string(&path).map_err(|_| {
            ConfigError(format!(
                "{label} report {} is not valid JSON",
                path.display()
            ))
        })?;
        let report: JsonValue = serde_json::from_str(&text).map_err(|_| {
            ConfigError(format!(
                "{label} report {} is not valid JSON",
                path.display()
            ))
        })?;
        let object = report.as_object().ok_or_else(|| {
            ConfigError(format!(
                "{label} report {} is not an object",
                path.display()
            ))
        })?;
        assert_no_extra_fields(
            object.keys().map(String::as_str),
            REPORT_FIELDS,
            &format!("{label} report {}", path.display()),
        )?;
        if report.get("bench_id").and_then(JsonValue::as_str) != Some(stem.as_str()) {
            return Err(ConfigError(format!(
                "{label} report {} has bench_id {:?} (must match the file name)",
                path.display(),
                report.get("bench_id")
            )));
        }
        if !is_valid_bench_id(&stem) {
            return Err(ConfigError(format!(
                "{label} report {} has an invalid bench id",
                path.display()
            )));
        }
        let fixture = required_string(&report, "fixture")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ConfigError(format!(
                    "{label} report {} has no valid fixture",
                    path.display()
                ))
            })?
            .to_string();
        let platform = required_string(&report, "platform")
            .filter(|value| platform_re.is_match(value))
            .ok_or_else(|| {
                ConfigError(format!(
                    "{label} report {} has no valid platform",
                    path.display()
                ))
            })?
            .to_string();
        let harness_version = required_string(&report, "harness_version")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ConfigError(format!(
                    "{label} report {} has no valid harness_version",
                    path.display()
                ))
            })?
            .to_string();
        let wall = report
            .get("wall_ns")
            .and_then(JsonValue::as_object)
            .ok_or_else(|| {
                ConfigError(format!(
                    "{label} report {} has no integer wall_ns.p50/p95",
                    path.display()
                ))
            })?;
        assert_no_extra_fields(
            wall.keys().map(String::as_str),
            WALL_NS_FIELDS,
            &format!("{label} report {} wall_ns", path.display()),
        )?;
        let wall_p50 = json_nonnegative_integer(wall.get("p50")).ok_or_else(|| {
            ConfigError(format!(
                "{label} report {} has no integer wall_ns.p50/p95",
                path.display()
            ))
        })?;
        let wall_p95 = json_nonnegative_integer(wall.get("p95")).ok_or_else(|| {
            ConfigError(format!(
                "{label} report {} has no integer wall_ns.p50/p95",
                path.display()
            ))
        })?;
        if wall_p95 < wall_p50 {
            return Err(ConfigError(format!(
                "{label} report {} has wall_ns.p95 below wall_ns.p50",
                path.display()
            )));
        }
        let allocs = json_integer_or_null(report.get("allocs")).ok_or_else(|| {
            ConfigError(format!(
                "{label} report {} has a non-integer, non-null allocs",
                path.display()
            ))
        })?;
        let alloc_bytes_peak =
            json_integer_or_null(report.get("alloc_bytes_peak")).ok_or_else(|| {
                ConfigError(format!(
                    "{label} report {} has a non-integer, non-null alloc_bytes_peak",
                    path.display()
                ))
            })?;
        let rss_peak_bytes =
            json_integer_or_null(report.get("rss_peak_bytes")).ok_or_else(|| {
                ConfigError(format!(
                    "{label} report {} has a non-integer, non-null rss_peak_bytes",
                    path.display()
                ))
            })?;
        reports.insert(
            stem,
            BenchReport {
                file: path,
                fixture,
                platform,
                harness_version,
                wall_p50,
                allocs,
                alloc_bytes_peak,
                rss_peak_bytes,
            },
        );
    }
    Ok(reports)
}

fn assert_no_extra_fields<'a>(
    keys: impl Iterator<Item = &'a str>,
    allowed: &[&str],
    where_: &str,
) -> Result<(), ConfigError> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    let extras = keys
        .filter(|key| !allowed.contains(key))
        .collect::<BTreeSet<_>>();
    if !extras.is_empty() {
        return Err(ConfigError(format!(
            "{where_} has unknown fields {} (allowed: {})",
            extras.into_iter().collect::<Vec<_>>().join(", "),
            allowed.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(())
}

fn required_string<'a>(value: &'a JsonValue, field: &str) -> Option<&'a str> {
    value.get(field).and_then(JsonValue::as_str)
}

fn json_nonnegative_integer(value: Option<&JsonValue>) -> Option<u64> {
    value.and_then(JsonValue::as_u64)
}

fn is_valid_bench_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn json_integer_or_null(value: Option<&JsonValue>) -> Option<Option<u64>> {
    match value {
        Some(JsonValue::Null) => Some(None),
        Some(value) => value.as_u64().map(Some),
        None => None,
    }
}

fn select_budgets(
    entries: &BTreeMap<String, Budget>,
    selected: &[String],
) -> BTreeMap<String, Budget> {
    if selected.is_empty() {
        return entries.clone();
    }
    selected
        .iter()
        .filter_map(|id| entries.get(id).cloned().map(|entry| (id.clone(), entry)))
        .collect()
}

fn select_reports(
    entries: BTreeMap<String, BenchReport>,
    selected: &[String],
) -> BTreeMap<String, BenchReport> {
    if selected.is_empty() {
        return entries;
    }
    entries
        .into_iter()
        .filter(|(id, _)| selected.contains(id))
        .collect()
}

fn update_baseline(
    options: &Options,
    budgets: &BTreeMap<String, Budget>,
    current_reports: &BTreeMap<String, BenchReport>,
) -> Result<u8, ConfigError> {
    let drift_rows = reconciliation_only(budgets, current_reports);
    if !drift_rows.is_empty() {
        for row in &drift_rows {
            println!("{row}");
        }
        println!(
            "bench-compare: refusing baseline update: reconciliation failed (breaches={})",
            drift_rows.len()
        );
        return Ok(1);
    }
    fs::create_dir_all(&options.baseline).map_err(|error| {
        ConfigError(format!(
            "cannot create baseline directory {}: {error}",
            options.baseline.display()
        ))
    })?;
    for (id, report) in current_reports {
        fs::copy(&report.file, options.baseline.join(format!("{id}.json"))).map_err(|error| {
            ConfigError(format!(
                "cannot copy {} to baseline: {error}",
                report.file.display()
            ))
        })?;
        println!("updated baseline {id}");
    }
    println!(
        "bench-compare: baseline updated ({} benches) under {}",
        current_reports.len(),
        options.baseline.display()
    );
    Ok(0)
}

struct Verdict {
    rows: Vec<String>,
    breaches: usize,
    gated_ok: usize,
    alloc_gated: usize,
}

fn compare(
    budgets: &BTreeMap<String, Budget>,
    baseline_reports: &BTreeMap<String, BenchReport>,
    current_reports: &BTreeMap<String, BenchReport>,
) -> Result<Verdict, ConfigError> {
    let ids = budgets
        .keys()
        .chain(current_reports.keys())
        .chain(baseline_reports.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut verdict = Verdict {
        rows: Vec::new(),
        breaches: 0,
        gated_ok: 0,
        alloc_gated: 0,
    };
    for id in ids {
        let budget = budgets.get(&id);
        let current = current_reports.get(&id);
        let baseline = baseline_reports.get(&id);
        if budget.is_none() {
            let where_ = if current.is_some() {
                "current"
            } else {
                "baseline"
            };
            verdict.rows.push(format!(
                "FAIL {id} unregistered bench ({where_} result has no budgets.toml [bench] entry)"
            ));
            verdict.breaches += 1;
            continue;
        }
        let budget = budget.unwrap();
        let Some(current) = current else {
            verdict.rows.push(format!(
                "FAIL {id} bench disappeared (budgets.toml entry has no current result)"
            ));
            verdict.breaches += 1;
            continue;
        };
        assert_comparable_identity(&id, baseline, current)?;
        let mut bench_rows = Vec::new();
        let wall_skipped = wall_report_only_reason(budget, baseline);
        let mut limit = None;
        if wall_skipped.is_none() {
            let baseline = baseline.unwrap();
            let wall_limit = baseline.wall_p50 * (10000 + budget.wall_tolerance_bp) / 10000;
            limit = Some(wall_limit);
            if current.wall_p50 > wall_limit {
                bench_rows.push(format!(
                    "FAIL {id} wall_p50 {}ns > limit {}ns (baseline {}ns + {}% tolerance)",
                    current.wall_p50,
                    wall_limit,
                    baseline.wall_p50,
                    format_tolerance_percent(budget.wall_tolerance_bp)
                ));
            }
        }
        if current.allocs != budget.allocs {
            bench_rows.push(format!(
                "FAIL {id} allocs {} -> {} (exact gate against budgets.toml: allocs are deterministic and machine-independent)",
                format_count(budget.allocs),
                format_count(current.allocs)
            ));
        }
        let peak_budget = budget.alloc_bytes_peak_by_platform.get(&current.platform);
        if !budget.alloc_bytes_peak_by_platform.is_empty() && peak_budget.is_none() {
            bench_rows.push(format!(
                "FAIL {id} alloc_bytes_peak platform {} has no exact budget (registered: {})",
                current.platform,
                budget
                    .alloc_bytes_peak_by_platform
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        } else if let Some(peak_budget) = peak_budget {
            if current.alloc_bytes_peak != *peak_budget {
                bench_rows.push(format!(
                    "FAIL {id} alloc_bytes_peak[{}] {} -> {} (exact platform-aware peak gate)",
                    current.platform,
                    format_count(*peak_budget),
                    format_count(current.alloc_bytes_peak)
                ));
            }
        }
        let peak = if budget.alloc_bytes_peak_by_platform.is_empty() {
            String::new()
        } else {
            format!(
                " alloc_bytes_peak[{}] {}B",
                current.platform,
                format_count(current.alloc_bytes_peak)
            )
        };
        if !bench_rows.is_empty() {
            verdict.breaches += bench_rows.len();
            verdict.rows.extend(bench_rows);
        } else if let Some(reason) = wall_skipped {
            verdict.rows.push(format!(
                "alloc-gated {id} allocs {}{peak} ok (wall_p50 {}ns report-only: {reason}) rss {}",
                format_count(current.allocs),
                current.wall_p50,
                format_rss(current.rss_peak_bytes)
            ));
            verdict.alloc_gated += 1;
        } else {
            verdict.rows.push(format!(
                "ok {id} wall_p50 {}ns (baseline {}ns limit {}ns) allocs {}{peak} rss {}",
                current.wall_p50,
                baseline.unwrap().wall_p50,
                limit.unwrap(),
                format_count(current.allocs),
                format_rss(current.rss_peak_bytes)
            ));
            verdict.gated_ok += 1;
        }
    }
    Ok(verdict)
}

fn reconciliation_only(
    budgets: &BTreeMap<String, Budget>,
    current_reports: &BTreeMap<String, BenchReport>,
) -> Vec<String> {
    budgets
        .keys()
        .chain(current_reports.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|id| {
            if !budgets.contains_key(&id) {
                Some(format!(
                    "FAIL {id} unregistered bench (current result has no budgets.toml [bench] entry)"
                ))
            } else if !current_reports.contains_key(&id) {
                Some(format!(
                    "FAIL {id} bench disappeared (budgets.toml entry has no current result)"
                ))
            } else {
                None
            }
        })
        .collect()
}

fn wall_report_only_reason(
    budget: &Budget,
    baseline: Option<&BenchReport>,
) -> Option<&'static str> {
    if baseline.is_none() {
        Some("no committed baseline report")
    } else if budget.wall_p50_ns == 0 {
        Some("budgets.toml wall baseline not yet recorded")
    } else {
        None
    }
}

fn assert_comparable_identity(
    id: &str,
    baseline: Option<&BenchReport>,
    current: &BenchReport,
) -> Result<(), ConfigError> {
    let Some(baseline) = baseline else {
        return Ok(());
    };
    for (field, left, right) in [
        ("fixture", &baseline.fixture, &current.fixture),
        ("platform", &baseline.platform, &current.platform),
        (
            "harness_version",
            &baseline.harness_version,
            &current.harness_version,
        ),
    ] {
        if left != right {
            return Err(ConfigError(format!(
                "{id} baseline/current {field} mismatch: {left} vs {right}"
            )));
        }
    }
    Ok(())
}

fn format_count(value: Option<u64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |value| value.to_string())
}

fn format_rss(value: Option<u64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |value| format!("{value}B"))
}

fn format_tolerance_percent(bp: u64) -> String {
    if bp % 100 == 0 {
        (bp / 100).to_string()
    } else {
        let mut value = format!("{:.2}", bp as f64 / 100.0);
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
        value
    }
}

fn script_repo_root() -> Result<PathBuf, String> {
    Path::new(file!())
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("pnpm-workspace.yaml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot resolve Vize repository root from script path".to_string())
}

#[cfg(test)]
mod tests {
    use super::is_valid_bench_id;

    #[test]
    fn bench_ids_match_the_legacy_javascript_contract() {
        for id in [
            "armature_parse_stress-deep",
            "s1_to_s2_emit_p2_11_dom_surface",
            "harness-selfcheck",
            "A.B_c-1",
        ] {
            assert!(is_valid_bench_id(id), "{id}");
        }

        for id in ["", "space id", "slash/id", "unicode-✓"] {
            assert!(!is_valid_bench_id(id), "{id}");
        }
    }
}
