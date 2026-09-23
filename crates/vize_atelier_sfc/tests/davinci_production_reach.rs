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
//! - **Reach floors.** `docs/davinci/plan/reach-budgets.toml` `[reach]` pins, per
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
    pub mod diff;
    pub mod parity;
    #[cfg(test)]
    mod selection_tests;
    pub mod shapes;
    pub mod tally;
}

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use davinci_production_reach::diff::divergence;
use davinci_production_reach::parity::parity_failures;
use davinci_production_reach::shapes::{Shape, compile, explicit_vapor_source};
use davinci_production_reach::tally::{Lane, Tally, classify, classify_route, floors};
use vize_atelier_sfc::{SfcCompileResult, SfcParseOptions, parse_sfc};
use vize_s0::profiler::global_profiler;

// The production reach and focused parity tests all read the global profiler.
static PROFILER_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn production_compiles_report_and_hold_their_davinci_reach() {
    let _guard = PROFILER_TEST_LOCK.lock().unwrap();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(production_reach_body)
        .expect("spawn P3-17 production reach thread")
        .join()
        .expect("P3-17 production reach thread must not panic");
}

#[test]
fn nested_interactive_recoveries_keep_production_dom_parity() {
    let _guard = PROFILER_TEST_LOCK.lock().unwrap();
    const NESTED_ANCHOR: &str =
        "Nested anchor start tag closed the previous anchor before inserting the new one.";
    const IGNORED_END: &str = "HTML tree construction ignored this end tag because the element was already closed before a nested start tag.";
    let cases = [
        (
            "<template><a href='/outer'><a href='/inner'>inner</a></a></template><script setup>const label = 'inner'</script>",
            1,
        ),
        (
            "<template><a><a>one</a></a><a><a>two</a></a></template><script setup>const label = 'inner'</script>",
            2,
        ),
    ];

    for (source, recoveries) in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let shape = Shape::DomInline;
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        let selected = compile(&descriptor, "NestedAnchor.vue", shape).unwrap();
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();
        assert_eq!(
            classify(shape, &counters),
            Ok(Lane::Accepted),
            "{shape:?}: {source}"
        );

        let expected: Vec<_> = [NESTED_ANCHOR, IGNORED_END]
            .into_iter()
            .cycle()
            .take(recoveries * 2)
            .collect();
        let warnings: Vec<_> = selected
            .warnings
            .iter()
            .map(|warning| warning.message.as_str())
            .collect();
        assert_eq!(warnings, expected, "{shape:?}: {source}");

        let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
            compile(&descriptor, "NestedAnchor.vue", shape)
        });
        assert_eq!(divergence(&selected, &legacy), None, "{shape:?}: {source}");
    }
}

#[test]
fn nested_form_recovery_keeps_production_dom_parity() {
    let _guard = PROFILER_TEST_LOCK.lock().unwrap();
    let source = "<template><form><div><form>inner</form></div></form></template><script setup>const label = 'inner'</script>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let shape = Shape::DomInline;
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let selected = compile(&descriptor, "NestedForm.vue", shape);
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    assert_eq!(
        classify(shape, &counters),
        Ok(Lane::Legacy("parse_error".to_owned()))
    );

    let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
        compile(&descriptor, "NestedForm.vue", shape)
    });
    match (selected, legacy) {
        (Ok(selected), legacy) => assert_eq!(divergence(&selected, &legacy), None),
        (Err(selected), Err(legacy)) => {
            assert_eq!(selected.code, legacy.code);
            assert_eq!(selected.message, legacy.message);
        }
        (selected, legacy) => panic!("nested form compile result differs: {selected:?} {legacy:?}"),
    }
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
            if child.file_name().is_some_and(|name| {
                name == "node_modules" || name == "_git" || name == "_git-worktrees"
            }) {
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
    let lane = match classify_route(shape, &counters, explicit_vapor_source(descriptor)) {
        Ok(lane) => lane,
        Err(violation) => {
            tally.violations.push(format!("{name}: {violation}"));
            return;
        }
    };
    let accepted = lane == Lane::Accepted;
    let croquis = lane == Lane::Legacy("croquis".to_owned());
    if lane == Lane::Unrecorded && tally.unrecorded_samples.len() < 5 {
        tally.unrecorded_samples.push(name.to_owned());
    }
    tally.record(lane);
    if accepted && shape.is_dom() {
        compare_with_legacy(descriptor, name, shape, &selected, tally);
    }
    if croquis && shape.is_dom() {
        measure_projection(descriptor, name, shape, tally);
    }
}

/// The Croquis projection the production selector still refuses (perf):
/// how many refused templates S2 would emit, each held to whole-module
/// parity with the legacy lane.
fn measure_projection(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    tally: &mut Tally,
) {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let projected = vize_atelier_dom::differential::with_croquis_projection(|| {
        compile(descriptor, name, shape)
    });
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    let Ok(projected) = projected else {
        return;
    };
    if classify(shape, &counters) == Ok(Lane::Accepted) {
        tally.ready += 1;
        compare_with_legacy(descriptor, name, shape, &projected, tally);
    }
}

fn compare_with_legacy(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    selected: &SfcCompileResult,
    tally: &mut Tally,
) {
    tally.compared += 1;
    let legacy =
        vize_atelier_dom::differential::with_legacy_lane(|| compile(descriptor, name, shape));
    if let Some(divergence) = divergence(selected, &legacy) {
        tally
            .divergences
            .push(format!("{name} [{}]: {divergence}", shape.id()));
    }
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
    let failures = parity_failures(sweep);
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
        "reach-budgets.toml [reach] ids must be exactly the measured shapes"
    );
    let mut failures = Vec::new();
    for (shape, tally) in sweep.tallies.values() {
        let floor = floors[shape.id()];
        if tally.ready < floor.ready_min {
            failures.push(format!(
                "[reach].{}: parity-ready {} < ready_min {} (a projection regression)",
                shape.id(),
                tally.ready,
                floor.ready_min
            ));
        } else if tally.ready > floor.ready_min {
            eprintln!(
                "[reach].{}: parity-ready {} > ready_min {}; raise the floor (ratchet)",
                shape.id(),
                tally.ready,
                floor.ready_min
            );
        }
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
