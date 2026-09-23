#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! toml = "0.9"
//! ```
//!
//! Davinci P5-11a (TS-44 baselines): measures the resident `vize lsp` and
//! gates `docs/davinci/plan/budgets.toml [resource]`.
//!
//!   --measure  --preset <id> [--runs N] [--keystrokes K] [--idle-seconds S]
//!              [--server <vize>] [--out <json>]   record one measurement
//!   --check    --measurement <json>                every pinned ceiling holds
//!   --validate                                     every number carries its methodology
//!   --ratchet  <base budgets.toml>                 no ceiling loosened or dropped
//!
//! `--budgets <path>` overrides the budgets file for every mode (tests use it).

#[path = "../../support/common.rs"]
mod common;
#[path = "../../support/editors/e2e.rs"]
mod editor_e2e;
#[path = "../../support/davinci/resource_session.rs"]
mod resource_session;

use resource_session::{ProcSampler, Session, TreeSample};
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// The metric keys a preset may pin, with the unit each is recorded in.
const METRICS: &[(&str, &str)] = &[
    ("cold_start_ms", "ms"),
    ("first_diagnostics_ms", "ms"),
    ("keystroke_p95_ms", "ms"),
    ("rss_peak_mib", "MiB"),
    ("rss_idle_mib", "MiB"),
    ("idle_cpu_pct", "%"),
    ("server_binary_mib", "MiB"),
];
/// Methodology every preset (its `methodology` inline table) and every metric entry must carry.
const PRESET_FIELDS: &[&str] = &["machine", "project", "server_build", "command", "runs", "recorded_at", "recorded_run"];
const METRIC_FIELDS: &[&str] = &["baseline", "ceiling", "headroom", "unit", "statistic", "sampler"];
const DOCUMENT: &str = "src/Scenario.vue";
/// Line 3 of the TS-45 fixture document is `const total = "3";`: keystrokes
/// alternate the initializer between two distinct type errors, so each edit's
/// settled publish is recognizable by its TS 2322 message.
const TOGGLE: [(&str, &str); 2] = [
    ("true", "Type 'boolean' is not assignable to type 'number'."),
    ("\"3\"", "Type 'string' is not assignable to type 'number'."),
];

fn main() -> ExitCode {
    common::main_result(run(std::env::args().skip(1).collect()))
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|arg| arg == name).and_then(|at| args.get(at + 1)).cloned()
}

fn run(args: Vec<String>) -> Result<(), String> {
    let repo = common::repo_root()?;
    let budgets = flag(&args, "--budgets").map_or_else(|| repo.join("docs/davinci/plan/budgets.toml"), PathBuf::from);
    if args.iter().any(|arg| arg == "--measure") {
        return measure(&repo, &args);
    }
    let head = load_resource(&budgets)?;
    let problems = if let Some(measurement) = flag(&args, "--measurement") {
        let mut problems = validate(&head);
        problems.extend(check(&head, &common::read_json(measurement)?));
        problems
    } else if let Some(base) = flag(&args, "--ratchet") {
        ratchet(&load_resource(Path::new(&base))?, &head)
    } else if args.iter().any(|arg| arg == "--validate") {
        validate(&head)
    } else {
        return Err("usage: resource-budgets.rs --measure|--check --measurement <json>|--validate|--ratchet <base>".into());
    };
    if problems.is_empty() {
        println!("resource budgets: ok ({} preset(s))", head.len());
        Ok(())
    } else {
        Err(format!("resource budgets: {} problem(s)\n  {}", problems.len(), problems.join("\n  ")))
    }
}

type Presets = BTreeMap<String, toml::Table>;

/// `[resource.<preset>]` tables of a budgets file (the bare `[resource]` table holds comments only).
fn load_resource(path: &Path) -> Result<Presets, String> {
    let text = common::read_text(path)?;
    let root: toml::Table = toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    let resource = root.get("resource").and_then(toml::Value::as_table).ok_or("budgets file has no [resource] table")?;
    let mut presets = Presets::new();
    for (name, value) in resource {
        let table = value.as_table().ok_or_else(|| format!("[resource] key `{name}` must be a preset table"))?;
        presets.insert(name.clone(), table.clone());
    }
    Ok(presets)
}

fn number(value: Option<&toml::Value>) -> Option<f64> {
    value.and_then(|value| value.as_float().or_else(|| value.as_integer().map(|integer| integer as f64)))
}

fn validate(presets: &Presets) -> Vec<String> {
    let mut problems = Vec::new();
    if presets.is_empty() {
        problems.push("[resource] pins no preset".into());
    }
    for (preset, table) in presets {
        match table.get("methodology").and_then(toml::Value::as_table) {
            Some(methodology) => {
                for field in PRESET_FIELDS {
                    if methodology.get(*field).is_none() {
                        problems.push(format!("{preset}: missing methodology field `{field}`"));
                    }
                }
            }
            None => problems.push(format!("{preset}: missing the `methodology` table ({})", PRESET_FIELDS.join(", "))),
        }
        let mut pinned = 0;
        for (key, value) in table {
            if key == "methodology" {
                continue;
            }
            let Some(unit) = METRICS.iter().find(|(name, _)| name == key).map(|(_, unit)| *unit) else {
                problems.push(format!("{preset}: unknown key `{key}`"));
                continue;
            };
            pinned += 1;
            let Some(entry) = value.as_table() else {
                problems.push(format!("{preset}.{key}: a bare number has no methodology; use {{ {} }}", METRIC_FIELDS.join(", ")));
                continue;
            };
            for field in METRIC_FIELDS {
                if entry.get(*field).is_none() {
                    problems.push(format!("{preset}.{key}: missing methodology field `{field}`"));
                }
            }
            let (baseline, ceiling, headroom) =
                (number(entry.get("baseline")), number(entry.get("ceiling")), number(entry.get("headroom")));
            if entry.get("unit").and_then(toml::Value::as_str).is_some_and(|actual| actual != unit) {
                problems.push(format!("{preset}.{key}: unit must be `{unit}`"));
            }
            if let (Some(baseline), Some(ceiling), Some(headroom)) = (baseline, ceiling, headroom) {
                if !(baseline >= 0.0 && headroom >= 0.0 && ceiling >= baseline) {
                    problems.push(format!("{preset}.{key}: needs 0 <= baseline <= ceiling and headroom >= 0"));
                }
                if ceiling > (baseline * (1.0 + headroom)).ceil() + 1e-9 {
                    problems.push(format!("{preset}.{key}: ceiling {ceiling} exceeds ceil(baseline × (1 + headroom))"));
                }
            }
        }
        if pinned == 0 {
            problems.push(format!("{preset}: pins no metric"));
        }
    }
    problems
}

/// Every pinned metric of the measured preset must be measured and within its ceiling.
fn check(presets: &Presets, measurement: &Value) -> Vec<String> {
    let preset = measurement.get("preset").and_then(Value::as_str).unwrap_or_default();
    let Some(table) = presets.get(preset) else {
        return vec![format!("the measurement is for preset `{preset}`, which [resource] does not pin")];
    };
    let mut problems = Vec::new();
    for (key, _) in METRICS {
        let Some(ceiling) = table.get(*key).and_then(toml::Value::as_table).and_then(|entry| number(entry.get("ceiling"))) else {
            continue;
        };
        match measurement.pointer(&format!("/metrics/{key}/value")).and_then(Value::as_f64) {
            Some(value) if value <= ceiling => println!("{preset}.{key}: {value} <= {ceiling}"),
            Some(value) => problems.push(format!("{preset}.{key}: measured {value} exceeds the ceiling {ceiling}")),
            None => problems.push(format!("{preset}.{key}: pinned but not measured")),
        }
    }
    problems
}

/// Ceilings only tighten: a raised ceiling or headroom, or a dropped metric or preset, is a loosening.
fn ratchet(base: &Presets, head: &Presets) -> Vec<String> {
    let mut problems = Vec::new();
    for (preset, table) in base {
        let Some(current) = head.get(preset) else {
            problems.push(format!("{preset}: preset removed"));
            continue;
        };
        for (key, _) in METRICS {
            let Some(before) = table.get(*key).and_then(toml::Value::as_table) else { continue };
            let Some(after) = current.get(*key).and_then(toml::Value::as_table) else {
                problems.push(format!("{preset}.{key}: pinned metric removed"));
                continue;
            };
            for field in ["ceiling", "headroom"] {
                if let (Some(old), Some(new)) = (number(before.get(field)), number(after.get(field))) {
                    if new > old {
                        problems.push(format!("{preset}.{key}.{field}: loosened from {old} to {new}"));
                    }
                }
            }
        }
    }
    problems
}

fn percentile(values: &[f64], quantile: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    // Nearest-rank: the smallest value with at least `quantile` of samples at or below it.
    let rank = ((quantile * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

fn measure(repo: &Path, args: &[String]) -> Result<(), String> {
    let preset = flag(args, "--preset").ok_or("--measure needs --preset <id>")?;
    let runs: usize = flag(args, "--runs").unwrap_or_else(|| "3".into()).parse().map_err(|_| "bad --runs")?;
    let keystrokes: usize = flag(args, "--keystrokes").unwrap_or_else(|| "20".into()).parse().map_err(|_| "bad --keystrokes")?;
    let idle: u64 = flag(args, "--idle-seconds").unwrap_or_else(|| "60".into()).parse().map_err(|_| "bad --idle-seconds")?;
    let server = match flag(args, "--server") {
        // Absolute: the server is spawned with the fixture workspace as its cwd.
        Some(path) => PathBuf::from(&path).canonicalize().map_err(|error| format!("--server {path}: {error}"))?,
        None => editor_e2e::resolve_real_server_path(repo)?,
    };
    let sampler = ProcSampler::new().map(Arc::new);
    let mut per_run = Vec::new();
    for run in 0..runs {
        let session_root = std::env::temp_dir().join(format!("vize-p5-11a-{}-{run}", std::process::id()));
        let workspace = prepare_workspace(repo, &session_root.join("real-vue"))?;
        let result = measure_run(&server, &workspace, keystrokes, idle, sampler.clone());
        let _ = std::fs::remove_dir_all(&session_root);
        let result = result?;
        eprintln!("run {}/{runs}: {}", run + 1, Value::Object(result.clone()));
        per_run.push(result);
    }
    let series = |key: &str| -> Vec<f64> {
        per_run.iter().flat_map(|run| match run.get(key) {
            Some(Value::Array(values)) => values.iter().filter_map(Value::as_f64).collect::<Vec<_>>(),
            Some(value) => value.as_f64().into_iter().collect(),
            None => Vec::new(),
        }).collect()
    };
    let mut metrics = Map::new();
    let mut put = |key: &str, statistic: &str, samples: Vec<f64>, value: Option<f64>| {
        if let Some(value) = value {
            metrics.insert(key.into(), json!({ "value": value, "statistic": statistic, "samples": samples }));
        }
    };
    let (cold, first, keys) = (series("cold_start_ms"), series("first_diagnostics_ms"), series("keystroke_ms"));
    put("cold_start_ms", "median over runs", cold.clone(), (!cold.is_empty()).then(|| percentile(&cold, 0.5)));
    put("first_diagnostics_ms", "median over runs", first.clone(), (!first.is_empty()).then(|| percentile(&first, 0.5)));
    put("keystroke_p95_ms", "nearest-rank p95 over every keystroke of every run", keys.clone(), (!keys.is_empty()).then(|| percentile(&keys, 0.95)));
    for (key, source) in [("rss_peak_mib", "rss_peak_mib"), ("rss_idle_mib", "rss_idle_mib"), ("idle_cpu_pct", "idle_cpu_pct")] {
        let samples = series(source);
        put(key, "maximum over runs", samples.clone(), samples.iter().copied().reduce(f64::max));
    }
    let binary = std::fs::metadata(&server).map_err(|error| error.to_string())?.len();
    put("server_binary_mib", "file size of the measured server", vec![mib(binary)], Some(mib(binary)));
    let output = json!({
        "preset": preset,
        "server": server.display().to_string(),
        "project": "editors/vscode/test-fixtures/extension-host/real-vue (src/Scenario.vue, the TS-45 fixture)",
        "runs": runs,
        "keystrokes_per_run": keystrokes,
        "idle_seconds": idle,
        "sampler": if sampler.is_some() { "linux /proc/<pid>/stat over the server process tree" } else { "unavailable off Linux: RSS and CPU not measured" },
        "metrics": metrics,
    });
    let text = serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?;
    if let Some(out) = flag(args, "--out") {
        common::write_text(&out, &format!("{text}\n"))?;
    }
    println!("{text}");
    Ok(())
}

fn measure_run(server: &Path, workspace: &Path, keystrokes: usize, idle: u64, sampler: Option<Arc<ProcSampler>>) -> Result<Map<String, Value>, String> {
    let uri = format!("file://{}", workspace.join(DOCUMENT).canonicalize().map_err(|error| error.to_string())?.display());
    let root_uri = format!("file://{}", workspace.canonicalize().map_err(|error| error.to_string())?.display());
    let mut session = Session::spawn(server, workspace)?;
    let (peak, stop) = (Arc::new(Mutex::new(TreeSample::default())), Arc::new(AtomicBool::new(false)));
    let watcher = sampler.clone().map(|sampler| {
        let (peak, stop, pid) = (peak.clone(), stop.clone(), session.pid());
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                if let Some(sample) = sampler.sample(pid) {
                    let mut peak = peak.lock().unwrap();
                    peak.rss_bytes = peak.rss_bytes.max(sample.rss_bytes);
                }
                thread::sleep(Duration::from_millis(50));
            }
        })
    });
    let mut out = Map::new();
    let ms = |from: Instant, to: Instant| ((to - from).as_secs_f64() * 10_000.0).round() / 10.0;
    let (_, initialized) = session.request("initialize", json!({
        "processId": std::process::id(),
        "rootUri": root_uri,
        "capabilities": { "textDocument": { "publishDiagnostics": { "versionSupport": true } } },
        "initializationOptions": { "editor": true, "ecosystem": true, "lint": true, "typecheck": true },
        "workspaceFolders": [{ "uri": root_uri, "name": "real-vue" }],
    }))?;
    out.insert("cold_start_ms".into(), json!(ms(session.spawned_at, initialized.at)));
    session.notify("initialized", json!({}))?;
    let text = std::fs::read_to_string(workspace.join(DOCUMENT)).map_err(|error| error.to_string())?;
    let opened = session.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "vue", "version": 0, "text": text } }))?;
    let settled = |version: i64, message: &'static str| {
        let uri = uri.clone();
        move |value: &Value| {
            value.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && value.pointer("/params/uri").and_then(Value::as_str) == Some(uri.as_str())
                && value.pointer("/params/version").and_then(Value::as_i64) == Some(version)
                && value.pointer("/params/diagnostics").and_then(Value::as_array).is_some_and(|diagnostics| {
                    diagnostics.iter().any(|d| d.get("code") == Some(&json!(2322)) && d.get("message").and_then(Value::as_str) == Some(message))
                })
        }
    };
    let first = session.wait("the first type diagnostics", Duration::from_secs(120), settled(0, TOGGLE[1].1))?;
    out.insert("first_diagnostics_ms".into(), json!(ms(opened, first.at)));
    let line = text.lines().nth(3).ok_or("fixture changed")?.to_string();
    let start = line.find(TOGGLE[1].0).ok_or("fixture changed")?;
    let mut current = TOGGLE[1].0;
    let mut latencies = Vec::new();
    for keystroke in 0..keystrokes {
        let (next, message) = TOGGLE[keystroke % 2];
        let version = keystroke as i64 + 1;
        let range = json!({ "start": { "line": 3, "character": start }, "end": { "line": 3, "character": start + current.len() } });
        let sent = session.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri, "version": version }, "contentChanges": [{ "range": range, "text": next }] }))?;
        let answer = session.wait(&format!("diagnostics for keystroke {version}"), Duration::from_secs(60), settled(version, message))?;
        latencies.push(json!(ms(sent, answer.at)));
        current = next;
    }
    out.insert("keystroke_ms".into(), Value::Array(latencies));
    if let Some(sampler) = sampler {
        thread::sleep(Duration::from_secs(2)); // let the last publish's work finish
        let pid = session.pid();
        let before = sampler.sample(pid).ok_or("the server exited before the idle window")?;
        let window = Instant::now();
        thread::sleep(Duration::from_secs(idle));
        let after = sampler.sample(pid).ok_or("the server exited during the idle window")?;
        let seconds = window.elapsed().as_secs_f64();
        out.insert("idle_cpu_pct".into(), json!(((after.cpu_seconds - before.cpu_seconds) / seconds * 100.0 * 1000.0).round() / 1000.0));
        out.insert("rss_idle_mib".into(), json!(mib(after.rss_bytes)));
        out.insert("processes".into(), json!(after.processes));
    }
    stop.store(true, Ordering::Relaxed);
    if let Some(watcher) = watcher {
        let _ = watcher.join();
        out.insert("rss_peak_mib".into(), json!(mib(peak.lock().unwrap().rss_bytes)));
    }
    let code = session.close()?;
    if code != 0 {
        return Err(format!("the server exited with {code}"));
    }
    Ok(out)
}

/// The TS-45 workspace, prepared by the same helper the conformance runner
/// uses (it resolves Vue and the TypeScript 7 runtime through Node's module
/// resolution, which is what a pnpm install lays out).
fn prepare_workspace(repo: &Path, workspace: &Path) -> Result<PathBuf, String> {
    let helper = repo.join("tools/support/compat/editor-e2e/real-vue-workspace.mjs");
    let script = "const [helper, dir] = process.argv.slice(1); \
        const { prepareRealVueWorkspace } = await import(helper); \
        prepareRealVueWorkspace(dir);";
    let helper_url = format!("file://{}", helper.display());
    let workspace_arg = workspace.display().to_string();
    let output = common::run_capture_in("node", &["--input-type=module", "-e", script, &helper_url, &workspace_arg], repo)?;
    if output.status != 0 {
        return Err(format!("preparing the fixture workspace failed:\n{}", output.stderr));
    }
    Ok(workspace.to_path_buf())
}

fn mib(bytes: u64) -> f64 {
    (bytes as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0
}
