//! Davinci P3-17 production reach: how much of a real `compile_sfc` run the
//! Davinci stages actually emit.
//!
//! The per-stage differential lanes (`davinci_dom_corpus*`,
//! `davinci_ssr_corpus`, the Vapor artifact gates) prove parity for the
//! template-compiler entry points. They do not say how many *production*
//! compiles reach those stages: `compile_sfc` attaches a Croquis summary,
//! inline render closures, binding metadata, scoped-style ids and module-mode
//! hoisting, and each backend's selector may route such a compile back to the
//! legacy lane. This gate compiles every committed fixture SFC through each
//! shipping adapter shape ([`shapes::Shape`]) exactly as the adapter does,
//! reads the backend's selection counters around each compile, and reports
//! the fraction of templates each Davinci stage emitted, with a per-reason
//! count for the rest.
//!
//! Two gates ride on the measurement:
//!
//! - **Reach floors.** `davinci-road/plan/budgets.toml` `[reach]` pins, per
//!   shape, the accepted-template count on the committed fixtures. Floors
//!   only rise (the ratchet); a regression fails here.
//! - **Production parity.** Every DOM template the S2 emitter accepted is
//!   compiled again with the legacy lane forced
//!   (`vize_atelier_dom::differential::with_legacy_lane`) and the complete
//!   `compile_sfc` modules — code, CSS, errors, warnings — must be byte
//!   equal (TS-11 over the production path, charter #23).
//!
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` additionally sweeps every `.vue`
//! file under `<dir>` (the hydrated Real Project Matrix corpus) and reports
//! its reach; parity is enforced there too, the floors are not.
//!
//! ```text
//! cargo test -p vize_atelier_sfc --features davinci-dom-differential \
//!     --test davinci_production_reach -- --nocapture
//! ```

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

mod davinci_production_reach {
    pub mod shapes;
    pub mod tally;
}

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use davinci_production_reach::shapes::{Shape, compile};
use davinci_production_reach::tally::{Lane, Tally, classify, floors};
use vize_atelier_sfc::{SfcCompileResult, SfcError, SfcParseOptions, parse_sfc};
use vize_s0::profiler::global_profiler;

#[test]
fn production_compiles_report_and_hold_their_davinci_reach() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(production_reach_body)
        .expect("spawn P3-17 production reach thread")
        .join()
        .expect("P3-17 production reach thread must not panic");
}

fn production_reach_body() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures");
    let mut files = Vec::new();
    collect_committed_fixtures(&root, &mut files);
    assert!(
        files.len() > 400,
        "the committed fixture corpus shrank to {} files",
        files.len()
    );
    let committed = sweep(&root, &files);
    report("committed", &committed);
    assert_parity(&committed);
    assert_floors(&committed);

    let Some(sweep_root) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed fixtures only");
        return;
    };
    let hydrated = sweep(&sweep_root.root, &sweep_root.files);
    report(sweep_root.scope_label(), &hydrated);
    assert_parity(&hydrated);
}

/// Every `.vue` file under `tests/_fixtures`, sorted, skipping the
/// `_git` submodule corpus (swept only through the env root) and
/// `node_modules`.
fn collect_committed_fixtures(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            if child
                .file_name()
                .is_some_and(|name| name == "node_modules" || name == "_git")
            {
                continue;
            }
            collect_committed_fixtures(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

struct Sweep {
    files: u64,
    unreadable: u64,
    parse_errors: u64,
    without_template: u64,
    tallies: BTreeMap<&'static str, (Shape, Tally)>,
}

fn sweep(root: &Path, files: &[PathBuf]) -> Sweep {
    let mut result = Sweep {
        files: 0,
        unreadable: 0,
        parse_errors: 0,
        without_template: 0,
        tallies: Shape::ALL
            .into_iter()
            .map(|shape| (shape.id(), (shape, Tally::default())))
            .collect(),
    };
    for file in files {
        result.files += 1;
        let Ok(source) = fs::read_to_string(file) else {
            result.unreadable += 1;
            continue;
        };
        let name = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .into_owned();
        let parse_options = SfcParseOptions {
            filename: vize_s0::String::from(name.as_str()),
            ..Default::default()
        };
        let Ok(descriptor) = parse_sfc(&source, parse_options) else {
            result.parse_errors += 1;
            continue;
        };
        if descriptor.template.is_none() {
            result.without_template += 1;
            continue;
        }
        for (shape, tally) in result.tallies.values_mut() {
            measure(&descriptor, &name, *shape, tally);
        }
    }
    result
}

fn measure(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    tally: &mut Tally,
) {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let selected = compile(descriptor, name, shape);
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    let selected = match selected {
        Ok(selected) => selected,
        Err(error) => {
            let code = error.code.as_deref().unwrap_or("unknown").to_owned();
            *tally.sfc_errors.entry(code).or_default() += 1;
            return;
        }
    };
    let lane = match classify(shape, &counters) {
        Ok(lane) => lane,
        Err(violation) => {
            tally.violations.push(format!("{name}: {violation}"));
            return;
        }
    };
    let accepted = lane == Lane::Accepted;
    if lane == Lane::Unrecorded && tally.unrecorded_samples.len() < 5 {
        tally.unrecorded_samples.push(name.to_owned());
    }
    tally.record(lane);
    if accepted && shape.is_dom() {
        tally.compared += 1;
        let legacy =
            vize_atelier_dom::differential::with_legacy_lane(|| compile(descriptor, name, shape));
        if let Some(divergence) = divergence(&selected, &legacy) {
            tally
                .divergences
                .push(format!("{name} [{}]: {divergence}", shape.id()));
        }
    }
}

/// The first field on which the S2-selected and forced-legacy modules differ.
fn divergence(
    selected: &SfcCompileResult,
    legacy: &Result<SfcCompileResult, SfcError>,
) -> Option<String> {
    let legacy = match legacy {
        Ok(legacy) => legacy,
        Err(error) => return Some(format!("legacy lane failed: {error:?}")),
    };
    if selected.code != legacy.code {
        return Some(format!(
            "code differs at byte {}:\n--- s2\n{}\n--- legacy\n{}",
            first_diff(&selected.code, &legacy.code),
            window(&selected.code, &legacy.code),
            window(&legacy.code, &selected.code),
        ));
    }
    if selected.css != legacy.css {
        return Some("css differs".to_owned());
    }
    let messages = |errors: &[SfcError]| -> Vec<String> {
        errors
            .iter()
            .map(|error| error.message.to_string())
            .collect()
    };
    if messages(&selected.errors) != messages(&legacy.errors) {
        return Some(format!(
            "errors differ: s2={:?} legacy={:?}",
            messages(&selected.errors),
            messages(&legacy.errors)
        ));
    }
    if messages(&selected.warnings) != messages(&legacy.warnings) {
        return Some(format!(
            "warnings differ: s2={:?} legacy={:?}",
            messages(&selected.warnings),
            messages(&legacy.warnings)
        ));
    }
    None
}

fn first_diff(left: &str, right: &str) -> usize {
    left.bytes()
        .zip(right.bytes())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| left.len().min(right.len()))
}

fn window(source: &str, other: &str) -> String {
    let diff = first_diff(source, other);
    let start = source[..diff]
        .char_indices()
        .rev()
        .nth(120)
        .map_or(0, |(index, _)| index);
    let end = source[diff..]
        .char_indices()
        .nth(200)
        .map_or(source.len(), |(index, _)| diff + index);
    source[start..end].to_owned()
}

fn report(scope: &str, sweep: &Sweep) {
    eprintln!(
        "davinci production reach: scope={scope} files={} unreadable={} parse_errors={} without_template={}",
        sweep.files, sweep.unreadable, sweep.parse_errors, sweep.without_template
    );
    for (shape, tally) in sweep.tallies.values() {
        eprintln!("{}", tally.line(*shape));
    }
}

fn assert_parity(sweep: &Sweep) {
    let mut failures = Vec::new();
    for (shape, tally) in sweep.tallies.values() {
        failures.extend(tally.violations.iter().cloned());
        failures.extend(tally.divergences.iter().take(10).cloned());
        if shape.is_dom() && tally.unrecorded != 0 {
            failures.push(format!(
                "{}: {} DOM template compiles recorded no selection counter: {:?}",
                shape.id(),
                tally.unrecorded,
                tally.unrecorded_samples
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "production reach selection/parity failures:\n{}",
        failures.join("\n\n")
    );
}

fn assert_floors(sweep: &Sweep) {
    let floors = floors();
    let ids: Vec<&str> = Shape::ALL.iter().map(|shape| shape.id()).collect();
    let recorded: Vec<&str> = floors.keys().map(String::as_str).collect();
    assert_eq!(
        recorded, ids,
        "budgets.toml [reach] ids must be exactly the measured shapes"
    );
    let mut failures = Vec::new();
    for (shape, tally) in sweep.tallies.values() {
        let floor = floors[shape.id()];
        if tally.accepted < floor.accepted_min {
            failures.push(format!(
                "[reach].{}: accepted {} < accepted_min {} (a production-reach regression)",
                shape.id(),
                tally.accepted,
                floor.accepted_min
            ));
        } else if tally.accepted > floor.accepted_min {
            eprintln!(
                "[reach].{}: accepted {} > accepted_min {}; raise the floor (ratchet)",
                shape.id(),
                tally.accepted,
                floor.accepted_min
            );
        }
        if tally.templates < floor.templates_min {
            failures.push(format!(
                "[reach].{}: templates {} < templates_min {} (the measured corpus shrank)",
                shape.id(),
                tally.templates,
                floor.templates_min
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
