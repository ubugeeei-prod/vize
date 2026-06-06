//! Snapshot tests for TypeScript output mode
//!
//! This module tests the TypeScript output generation for Vue SFCs.
//! It uses the insta crate for snapshot testing, with snapshots stored
//! in tests/snapshots/sfc/ts/.
//!
//! The test cases are loaded from PKL or TOML fixtures in tests/fixtures/sfc/.
#![allow(clippy::disallowed_macros)]

use crate::{SfcCompileOptions, compile_sfc, parse_sfc};
use pklrust::{EvaluatorManager, EvaluatorOptions, ModuleSource};
use serde::Deserialize;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use vize_carton::{String, ToCompactString};

/// A test case from a snapshot fixture.
#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    input: String,
    #[allow(dead_code)]
    expected: Option<String>,
}

/// A fixture file containing multiple test cases.
#[derive(Debug, Deserialize)]
struct Fixture {
    #[allow(dead_code)]
    mode: Option<String>,
    cases: Vec<TestCase>,
}

/// Get the path to the tests/fixtures directory
fn fixtures_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
}

/// Get the path to the tests/snapshots directory
fn snapshots_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("snapshots")
}

/// Resolve an SFC snapshot fixture by preferring PKL over legacy TOML.
///
/// PKL is the preferred authoring format because it gives fixture files a real
/// typed module language instead of a flat data encoding. TOML remains as a
/// fallback so existing fixtures can migrate gradually without widening this
/// test harness change.
fn fixture_file(name: &str) -> PathBuf {
    let base = fixtures_path().join("sfc").join(name);
    let pkl = base.with_extension("pkl");
    if pkl.exists() {
        pkl
    } else {
        base.with_extension("toml")
    }
}

/// Load a fixture from a PKL or TOML file.
fn load_fixture(path: &Path) -> Result<Fixture, Box<dyn std::error::Error>> {
    if path.extension().and_then(|ext| ext.to_str()) == Some("pkl") {
        return load_pkl_fixture(path);
    }

    let content = std::fs::read_to_string(path)?;
    let fixture: Fixture = toml::from_str(&content)?;
    Ok(fixture)
}

/// Evaluate a PKL fixture with a project-local `pkl` binary when available.
///
/// Cargo test does not automatically put `node_modules/.bin` on PATH, so the
/// loader checks ancestor directories before falling back to the user's PATH.
fn load_pkl_fixture(path: &Path) -> Result<Fixture, Box<dyn std::error::Error>> {
    let command = pkl_command(path);
    let command = command.to_string_lossy();
    let mut manager = EvaluatorManager::with_command(command.as_ref())?;
    let options = pkl_evaluator_options(path);
    let evaluator = manager.new_evaluator(options)?;
    let result = manager.evaluate_module_typed::<Fixture>(&evaluator, ModuleSource::file(path));
    let _ = manager.close_evaluator(&evaluator);
    Ok(result?)
}

fn pkl_evaluator_options(path: &Path) -> EvaluatorOptions {
    let Some(root_dir) = path.parent() else {
        return EvaluatorOptions::preconfigured();
    };

    let root_dir = root_dir.to_string_lossy();
    EvaluatorOptions::preconfigured().root_dir(root_dir.as_ref())
}

fn pkl_command(path: &Path) -> PathBuf {
    for ancestor in path.ancestors() {
        for candidate in [
            ancestor.join("node_modules/.bin/pkl"),
            ancestor.join("node_modules/@pkl-community/pkl/pkl"),
        ] {
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from("pkl")
}

/// Normalize a test case name to a valid snapshot file name
fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace([' ', '-'], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

fn build_snapshot_name(prefix: &str, name: &str) -> String {
    let mut result = String::with_capacity(prefix.len() + name.len());
    result.push_str(prefix);
    result.push_str(name);
    result
}

/// Compile an SFC to TypeScript output
fn compile_sfc_ts(input: &str) -> String {
    let descriptor = match parse_sfc(input, Default::default()) {
        Ok(d) => d,
        Err(e) => {
            let mut msg = String::from("Parse error: ");
            let _ = write!(&mut msg, "{:?}", e);
            return msg;
        }
    };

    let mut options = SfcCompileOptions::default();
    // Enable TypeScript output mode
    options.script.is_ts = true;
    options.template.is_ts = true;
    options.script.id = Some("test.vue".to_compact_string());

    match compile_sfc(&descriptor, options) {
        Ok(result) => result.code,
        Err(e) => {
            let mut msg = String::from("Compile error: ");
            let _ = write!(&mut msg, "{:?}", e);
            msg
        }
    }
}

/// Compile an SFC to JavaScript output (TypeScript stripped)
fn compile_sfc_js(input: &str) -> String {
    let descriptor = match parse_sfc(input, Default::default()) {
        Ok(d) => d,
        Err(e) => {
            let mut msg = String::from("Parse error: ");
            let _ = write!(&mut msg, "{:?}", e);
            return msg;
        }
    };

    let mut options = SfcCompileOptions::default();
    // Disable TypeScript output mode - transpile to JavaScript
    options.script.is_ts = false;
    options.template.is_ts = false;
    options.script.id = Some("test.vue".to_compact_string());

    match compile_sfc(&descriptor, options) {
        Ok(result) => result.code,
        Err(e) => {
            let mut msg = String::from("Compile error: ");
            let _ = write!(&mut msg, "{:?}", e);
            msg
        }
    }
}

fn assert_sfc_snapshots(fixture_name: &str, snapshot_kind: &str, prefix: &str) {
    let snapshot_path = snapshots_path().join("sfc").join(snapshot_kind);
    std::fs::create_dir_all(&snapshot_path).ok();

    let fixture_path = fixture_file(fixture_name);
    let fixture = load_fixture(&fixture_path).expect("Failed to load fixture");

    for case in &fixture.cases {
        let normalized_name = normalize_name(&case.name);
        let output = match snapshot_kind {
            "js" => compile_sfc_js(&case.input),
            _ => compile_sfc_ts(&case.input),
        };

        insta::with_settings!({
            snapshot_path => &snapshot_path,
            prepend_module_to_snapshot => false,
            snapshot_suffix => "",
        }, {
            let snapshot_name = build_snapshot_name(prefix, &normalized_name);
            insta::assert_snapshot!(snapshot_name.as_str(), output.as_str());
        });
    }
}

#[test]
fn test_script_setup_ts_snapshots() {
    assert_sfc_snapshots("script-setup", "ts", "script_setup__");
}

#[test]
fn test_basic_sfc_ts_snapshots() {
    assert_sfc_snapshots("basic", "ts", "basic__");
}

#[test]
fn test_patches_ts_snapshots() {
    assert_sfc_snapshots("patches", "ts", "patches__");
}

#[test]
fn test_patches_js_snapshots() {
    assert_sfc_snapshots("patches", "js", "patches__");
}

#[test]
fn test_directives_ts_snapshots() {
    assert_sfc_snapshots("directives", "ts", "directives__");
}

#[test]
fn test_directives_js_snapshots() {
    assert_sfc_snapshots("directives", "js", "directives__");
}
