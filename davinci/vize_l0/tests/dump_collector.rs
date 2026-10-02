//! Typed pass observer and collector contracts retained after host retirement.

#![expect(clippy::expect_used, reason = "committed schemas are test oracles")]

use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};
use vize_l0::dump::collector::Collector;
use vize_l0::pass::{
    BudgetObserver, Fusability, Pair, PassDesc, PassKind, Pipeline, Preserved, TimingObserver,
    run_pipeline,
};
use vize_l0::profiler::global_profiler;

const BUDGET: &str =
    "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n";
const ALPHA: PassDesc = PassDesc::new(
    "alpha",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const BETA: PassDesc = PassDesc::new(
    "beta",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const PASSES: &[PassDesc] = &[ALPHA, BETA];
const PLAN: Pipeline = Pipeline::new("l2", PASSES);

fn collect(after_change_only: bool) -> (Collector, BudgetObserver) {
    let mut collector = Collector::new(after_change_only);
    collector.seed(BUDGET);
    let mut budget = BudgetObserver::new();
    run_pipeline(&PLAN, &mut budget, |event| {
        collector.after_pass(event, BUDGET);
        Ok(())
    })
    .expect("no-op body cannot fail");
    (collector, budget)
}

#[test]
fn collector_records_one_canonical_page_per_executed_pass() {
    let (collector, budget) = collect(false);
    assert_eq!(budget.walks, 1);
    assert_eq!(budget.passes, 2);
    assert_eq!(collector.pages.len(), 2);
    assert_eq!(collector.pages[0].name.as_str(), "000-l2.alpha.folio");
    assert_eq!(collector.pages[0].stage.as_str(), "l2");
    assert_eq!(collector.pages[0].pass.as_str(), "alpha");
    assert_eq!(collector.pages[0].text.as_str(), BUDGET);
    assert_eq!(collector.pages[1].name.as_str(), "001-l2.beta.folio");
    assert_eq!(collector.pages[1].text.as_str(), BUDGET);
}

#[test]
fn unchanged_passes_produce_no_gated_pages() {
    let (collector, budget) = collect(true);
    assert_eq!((budget.walks, budget.passes), (1, 2));
    assert_eq!(collector.pages.len(), 0);
}

#[test]
fn timing_json_satisfies_the_p0_11_schema() {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let mut observers = Pair(TimingObserver::new(), BudgetObserver::new());
    run_pipeline(&PLAN, &mut observers, |_event| Ok(())).expect("no-op body cannot fail");
    profiler.disable();
    assert_eq!(observers.0.recorded_walks, 1);
    assert_eq!((observers.1.walks, observers.1.passes), (1, 2));

    let export = export_report(
        profiler,
        &ProfileExportOptions {
            command: "typed-pass-test",
            allocation: None,
            budget: ProfileExportBudget::default(),
        },
    );
    let json: serde_json::Value =
        serde_json::from_str(export.to_json().as_str()).expect("export is valid JSON");
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/davinci/plan/profile-export.schema.json");
    let schema =
        serde_json::from_str(&std::fs::read_to_string(path).expect("committed schema reads"))
            .expect("committed schema is valid JSON");
    assert_eq!(schema_check::validate(&schema, &json, "$"), Ok(()));
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "typed-pass-test");
    assert_eq!(json["spans"][0]["key"], "davinci.pass.walk");
    assert_eq!(json["spans"][0]["count"], 1);
    assert_eq!(json["spans"][0]["attribution"]["stage"], "l2");
    assert_eq!(json["spans"][0]["attribution"]["pass"], "alpha");
}
