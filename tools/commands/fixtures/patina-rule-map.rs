#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde_json::Value;
use std::{env, process::ExitCode};

#[path = "../../rust/common.rs"]
mod common;

const RULE_MAP_REL: &str = "tests/_fixtures/patina-eslint-vue-rule-map.json";
const UPSTREAM_PACKAGE: &str = "eslint-plugin-vue";
const TRACKING_ISSUE: u64 = 3223;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() > 1 {
        return Err(usage());
    }
    let mode = args.first().map(String::as_str).unwrap_or("--check");
    match mode {
        "--check" => {
            validate_rule_map(&common::read_json(root.join(RULE_MAP_REL))?, &root)?;
            Ok(())
        }
        "--write" => {
            let rule_map = common::read_json(root.join(RULE_MAP_REL))?;
            validate_rule_map(&rule_map, &root)?;
            common::write_json_pretty(root.join(RULE_MAP_REL), &rule_map)?;
            Ok(())
        }
        "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn validate_rule_map(rule_map: &Value, root: &std::path::Path) -> Result<(), String> {
    if rule_map.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("rule map schemaVersion must be 1".to_string());
    }
    let bench_manifest = common::read_json(root.join("tools/benchmarks/scripts/package.json"))?;
    let pinned_version = bench_manifest
        .pointer("/devDependencies/eslint-plugin-vue")
        .and_then(Value::as_str)
        .ok_or_else(|| "tools/benchmarks/scripts/package.json must pin eslint-plugin-vue".to_string())?;
    let upstream = rule_map
        .get("upstream")
        .ok_or_else(|| "rule map must contain upstream".to_string())?;
    expect_string(upstream, "package", UPSTREAM_PACKAGE)?;
    expect_string(upstream, "version", pinned_version)?;
    let entries = rule_map
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| "rule map entries must be an object".to_string())?;
    let rule_count = upstream
        .get("ruleCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| "upstream.ruleCount must be a number".to_string())?;
    if entries.len() as u64 != rule_count {
        return Err(format!(
            "every pinned eslint-plugin-vue rule must have exactly one map entry: {} entries != {rule_count}",
            entries.len()
        ));
    }
    let keys = entries.keys().cloned().collect::<Vec<_>>();
    let sorted = {
        let mut sorted = keys.clone();
        sorted.sort();
        sorted
    };
    if keys != sorted {
        return Err("rule map entries must be codepoint sorted".to_string());
    }

    let mut mapped = 0u64;
    let mut unimplemented = 0u64;
    let mut intentional_divergence = 0u64;
    for (rule_id, entry) in entries {
        let status = entry
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{rule_id} must record status"))?;
        match status {
            "mapped" => {
                expect_non_empty_string(entry, "patinaRule")?;
                expect_enum_string(entry, "patinaSeverity", &["error", "warning"])?;
                let presets = entry
                    .get("patinaPresets")
                    .and_then(Value::as_array)
                    .ok_or_else(|| format!("{rule_id} needs patinaPresets"))?;
                let preset_strings = presets
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .filter(|value| !value.is_empty())
                            .map(str::to_string)
                            .ok_or_else(|| format!("{rule_id} patinaPresets must be strings"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut sorted_presets = preset_strings.clone();
                sorted_presets.sort();
                if preset_strings != sorted_presets {
                    return Err(format!("{rule_id} must list presets in sorted order"));
                }
                mapped += 1;
            }
            "unimplemented" => {
                if entry.get("issue").and_then(Value::as_u64) != Some(TRACKING_ISSUE) {
                    return Err(format!("{rule_id} must link the scorecard issue"));
                }
                unimplemented += 1;
            }
            "intentional-divergence" => {
                expect_non_empty_string(entry, "reason")?;
                intentional_divergence += 1;
            }
            _ => return Err(format!("{rule_id} has unsupported status {status:?}")),
        }
    }
    let summary = rule_map
        .get("summary")
        .ok_or_else(|| "rule map must contain summary".to_string())?;
    expect_u64(summary, "mapped", mapped)?;
    expect_u64(summary, "unimplemented", unimplemented)?;
    expect_u64(summary, "intentionalDivergence", intentional_divergence)?;
    if mapped + unimplemented + intentional_divergence != rule_count {
        return Err("the scorecard may not hide rules behind an uncounted status".to_string());
    }
    Ok(())
}

fn expect_string(value: &Value, key: &str, expected: &str) -> Result<(), String> {
    let actual = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))?;
    if actual != expected {
        return Err(format!("{key}: expected {expected}, got {actual}"));
    }
    Ok(())
}

fn expect_non_empty_string(value: &Value, key: &str) -> Result<(), String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{key} must be a non-empty string"))
        .map(|_| ())
}

fn expect_enum_string(value: &Value, key: &str, allowed: &[&str]) -> Result<(), String> {
    let actual = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))?;
    if allowed.contains(&actual) {
        Ok(())
    } else {
        Err(format!("{key}: unsupported value {actual}"))
    }
}

fn expect_u64(value: &Value, key: &str, expected: u64) -> Result<(), String> {
    let actual = value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("summary.{key} must be a number"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("summary.{key}: expected {expected}, got {actual}"))
    }
}

fn usage() -> String {
    "Usage: rust-script tools/commands/fixtures/patina-rule-map.rs [--check|--write]".to_string()
}
