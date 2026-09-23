//! The `-O` tier table equals `[optimization]` and `[target.p3-10]` in
//! `docs/davinci/plan/budgets.toml`, field for field. The file is the source
//! of truth: loosening or tightening it without the Rust table (or the
//! reverse) fails here.

use std::path::Path;

use vize_impeto::extract::{Metric, OptTier, OptimizationBudget, TiePolicy};

fn budgets() -> toml::Value {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let text = std::fs::read_to_string(repo.join("docs/davinci/plan/budgets.toml"))
        .expect("budgets.toml reads");
    toml::from_str(&text).expect("budgets.toml parses")
}

fn u32_field(row: &toml::Value, field: &str) -> u32 {
    row.get(field)
        .and_then(toml::Value::as_integer)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_else(|| panic!("[optimization] row lacks u32 `{field}`"))
}

fn str_field<'a>(row: &'a toml::Value, field: &str) -> &'a str {
    row.get(field)
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("row lacks string `{field}`"))
}

fn parse_row(row: &toml::Value) -> OptimizationBudget {
    let tier = OptTier::from_str(str_field(row, "tier")).expect("known tier");
    let tie_policy = match str_field(row, "tie_policy") {
        "reject" => TiePolicy::Reject,
        other => panic!("unknown tie policy `{other}`"),
    };
    OptimizationBudget {
        tier,
        candidate_budget: u32_field(row, "candidate_budget"),
        emitted_size_epsilon_pct: u32_field(row, "emitted_size_epsilon_pct"),
        reactive_edge_epsilon_pct: u32_field(row, "reactive_edge_epsilon_pct"),
        update_path_epsilon_pct: u32_field(row, "update_path_epsilon_pct"),
        required_improvements_min: u32_field(row, "required_improvements_min"),
        tie_policy,
    }
}

#[test]
fn every_tier_equals_its_budgets_toml_row() {
    let budgets = budgets();
    let table = budgets
        .get("optimization")
        .and_then(toml::Value::as_table)
        .expect("[optimization] table");
    let parsed: Vec<(&str, OptimizationBudget)> = table
        .iter()
        .map(|(key, row)| (key.as_str(), parse_row(row)))
        .collect();
    assert_eq!(
        parsed,
        [
            ("o0", OptTier::O0.budget()),
            ("o1", OptTier::O1.budget()),
            ("o2", OptTier::O2.budget()),
            ("o3", OptTier::O3.budget()),
        ]
    );
}

#[test]
fn metric_roles_equal_the_task_target() {
    let budgets = budgets();
    let target = budgets
        .get("target")
        .and_then(|target| target.get("p3-10"))
        .expect("[target.p3-10]");
    let constraints: Vec<&str> = target
        .get("constraint_metrics")
        .and_then(toml::Value::as_array)
        .expect("constraint_metrics")
        .iter()
        .map(|metric| metric.as_str().expect("metric name"))
        .collect();
    let objective = str_field(target, "objective_metric");
    let mut order = constraints.clone();
    order.push(objective);

    assert_eq!(
        Metric::CHECK_ORDER.map(Metric::as_str).as_slice(),
        order.as_slice()
    );
    assert_eq!(str_field(target, "tie_policy"), TiePolicy::Reject.as_str());
}
