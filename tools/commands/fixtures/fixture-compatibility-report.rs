#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    process::ExitCode,
};

#[path = "../../support/common.rs"]
mod common;

const CAPABILITY_VALUES: &[(&str, &[&str])] = &[
    ("vue-generation", &["0.x", "1.x", "2.x", "2.7", "3.x"]),
    (
        "api-style",
        &[
            "options-api",
            "class-api",
            "composition-api",
            "script-setup",
        ],
    ),
    (
        "nuxt-macro",
        &["define-page-meta", "use-head", "use-seo-meta"],
    ),
];
const ORACLE_KINDS: &[&str] = &[
    "compiler",
    "formatter-idempotency",
    "linter",
    "typechecker",
    "production-build",
    "authored-lsp",
    "vue-tsc-parity",
    "ssr",
    "hydration",
    "preview",
    "vrt",
    "real-vite-hmr",
];

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let ledger = common::read_json(root.join("tests/_fixtures/fixture-compatibility-ledger.json"))?;
    validate_minimal_ledger(&ledger)?;
    let report = create_report(&ledger)?;
    print!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    println!();
    Ok(())
}

fn create_report(ledger: &Value) -> Result<Value, String> {
    let fixture_map = ledger
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or_else(|| "ledger fixtures must be an array".to_string())?
        .iter()
        .map(|fixture| {
            let path = fixture_string(fixture, "fixturePath")?;
            Ok((path, fixture.clone()))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;

    let capabilities = ledger
        .get("capabilities")
        .and_then(Value::as_array)
        .ok_or_else(|| "ledger capabilities must be an array".to_string())?;
    let mut capability_report = serde_json::Map::new();
    for (dimension, values) in CAPABILITY_VALUES {
        let mut dimension_report = serde_json::Map::new();
        for value in *values {
            dimension_report.insert(
                (*value).to_string(),
                capability_counts(capabilities, dimension, value),
            );
        }
        capability_report.insert((*dimension).to_string(), Value::Object(dimension_report));
    }

    let oracles = ledger
        .get("oracles")
        .and_then(Value::as_array)
        .ok_or_else(|| "ledger oracles must be an array".to_string())?;
    let mut oracle_report = serde_json::Map::new();
    for kind in ORACLE_KINDS {
        let mut fixture_paths = BTreeSet::new();
        for oracle in oracles
            .iter()
            .filter(|oracle| oracle.get("kind").and_then(Value::as_str) == Some(*kind))
        {
            for path in expand_selection(&oracle["selection"], &fixture_map)? {
                fixture_paths.insert(path);
            }
        }
        let fixture_paths = fixture_paths.into_iter().collect::<Vec<_>>();
        oracle_report.insert(
            (*kind).to_string(),
            json!({
                "fixtureCount": fixture_paths.len(),
                "fixturePaths": fixture_paths,
            }),
        );
    }

    let ecosystem = count_membership(&fixture_map, "ecosystem");
    let app = count_membership(&fixture_map, "app");
    let app_only = fixture_map
        .values()
        .filter(|fixture| {
            memberships(fixture).is_ok_and(|items| {
                items.iter().any(|item| item == "app")
                    && !items.iter().any(|item| item == "ecosystem")
            })
        })
        .count();
    let mut unresolved = ledger
        .get("unresolved")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    unresolved.sort_by(|left, right| unresolved_key(left).cmp(&unresolved_key(right)));

    Ok(json!({
        "schema": "vize.fixtureCompatibilityReport",
        "version": 1,
        "inventories": {
            "gitlinks": fixture_map.len(),
            "ecosystem": ecosystem,
            "app": app,
            "appOnly": app_only,
        },
        "capabilities": Value::Object(capability_report),
        "oracles": Value::Object(oracle_report),
        "unresolved": unresolved,
    }))
}

fn capability_counts(capabilities: &[Value], dimension: &str, value: &str) -> Value {
    let claims = capabilities
        .iter()
        .filter(|capability| {
            capability.get("dimension").and_then(Value::as_str) == Some(dimension)
                && capability.get("value").and_then(Value::as_str) == Some(value)
        })
        .collect::<Vec<_>>();
    json!({
        "present": level_summary(&claims, "present"),
        "exercised": level_summary(&claims, "exercised"),
        "runtimeVerified": level_summary(&claims, "runtime"),
    })
}

fn level_summary(claims: &[&Value], level: &str) -> Value {
    let mut fixture_paths = BTreeSet::new();
    for claim in claims {
        if claim
            .get("levels")
            .and_then(Value::as_array)
            .is_some_and(|levels| levels.iter().any(|item| item.as_str() == Some(level)))
        {
            if let Some(path) = claim.get("fixturePath").and_then(Value::as_str) {
                fixture_paths.insert(path.to_string());
            }
        }
    }
    let fixture_paths = fixture_paths.into_iter().collect::<Vec<_>>();
    json!({ "count": fixture_paths.len(), "fixturePaths": fixture_paths })
}

fn expand_selection(
    selection: &Value,
    fixture_map: &BTreeMap<String, Value>,
) -> Result<Vec<String>, String> {
    match selection.get("type").and_then(Value::as_str) {
        Some("membership") => {
            let membership = selection
                .get("membership")
                .and_then(Value::as_str)
                .ok_or_else(|| "membership selection must name membership".to_string())?;
            Ok(fixture_map
                .iter()
                .filter_map(|(path, fixture)| {
                    memberships(fixture)
                        .ok()
                        .filter(|items| items.iter().any(|item| item == membership))
                        .map(|_| path.clone())
                })
                .collect())
        }
        Some("fixtures") => selection
            .get("fixturePaths")
            .and_then(Value::as_array)
            .ok_or_else(|| "fixtures selection must list fixturePaths".to_string())?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "fixturePaths entries must be strings".to_string())
            })
            .collect(),
        Some(other) => Err(format!("unknown oracle selection type {other}")),
        None => Err("oracle selection must name type".to_string()),
    }
}

fn validate_minimal_ledger(ledger: &Value) -> Result<(), String> {
    if ledger.get("schema").and_then(Value::as_str) != Some("vize.fixtureCompatibilityLedger") {
        return Err("unsupported ledger schema".to_string());
    }
    if ledger.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("unsupported ledger version".to_string());
    }
    Ok(())
}

fn count_membership(fixture_map: &BTreeMap<String, Value>, membership: &str) -> usize {
    fixture_map
        .values()
        .filter(|fixture| {
            memberships(fixture).is_ok_and(|items| items.iter().any(|item| item == membership))
        })
        .count()
}

fn memberships(fixture: &Value) -> Result<Vec<String>, String> {
    fixture
        .get("memberships")
        .and_then(Value::as_array)
        .ok_or_else(|| "fixture memberships must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "fixture membership entries must be strings".to_string())
        })
        .collect()
}

fn fixture_string(fixture: &Value, field: &str) -> Result<String, String> {
    fixture
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("fixture is missing {field}"))
}

fn unresolved_key(value: &Value) -> String {
    format!(
        "{}\0{}",
        value.get("dimension").and_then(Value::as_str).unwrap_or(""),
        value.get("value").and_then(Value::as_str).unwrap_or("")
    )
}
