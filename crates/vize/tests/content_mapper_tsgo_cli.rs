#![cfg(test)]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::json;
use vize_l0::{String as CompactString, cstr};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const JAVASCRIPT_TSC_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

mod commented_vif_11378;
mod component_listener_narrowing_11730;
mod component_v_for_source;
mod const_enum_import;
mod contextual_prop_anchors;
mod directive_tables;
mod dynamic_component_event_names;
mod dynamic_template_refs;
mod expect_element_coverage_11486;
mod fallthrough_value_12458;
mod generic_component_cast_11966;
mod generic_constructor_slots;
mod global_component_listener_11373;
mod global_components;
mod incomplete_tags;
mod inference_copy_bodies_11451_11452;
mod javascript_check_js;
mod let_anchor_assignment;
mod listener_anchor_11657;
mod long_prop_kebab_case;
mod memo_expression;
mod module_default_export;
mod nav_index_signature_access_12531;
mod nested_callback_11361;
mod nested_function_guard_11853;
mod optional_prop_spacing;
mod options_api_diagnostics;
mod project_declarations;
mod projection_modes;
mod props_binding;
mod pug_refusal;
mod quoted_generic_parameters;
mod recovered_attribute_boundary_11731;
mod replayed_pattern_duplicates;
mod reported_dynamic_targets;
mod reported_expression_duplicates_11374;
mod reported_generic_components;
mod reported_model_events_11390_11391;
mod reported_model_props_11359;
mod runtime_null_prop;
mod semantic_compile_errors_12522;
mod setup_local_public_types;
mod setup_verbatim_lines_12481;
mod support;
mod surplus_defaults;
mod template_callback_event_navigation;
mod template_expression_syntax;
mod template_model_props;
mod tsx_script_syntax;
mod type_only_import_use;
mod typed_context_undefined;
mod unicode_prop_bindings;
mod unknown_options_10218;
mod vfor_const_listener_narrowing_11973;

use support::{
    assert_success, check_project, copy_fixture, emitted_vue_declaration, install_mapper_manifest,
    install_vue_package, output_text, run_tsgo, workspace_root,
};

#[test]
fn standard_tsgo_reports_each_component_prop_problem_once() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/content_mapper_diagnostic_parity");
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/content_mapper_diagnostic_parity/expectations.json"
    ))
    .unwrap();
    assert_eq!(contract["issue"], 10187);
    assert_eq!(
        contract["typescriptSourceRevision"],
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799"
    );

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-diagnostic-parity-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert!(!output.status.success(), "fixture unexpectedly passed");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let diagnostic_lines = stdout
        .lines()
        .filter(|line| line.contains(": error TS"))
        .collect::<Vec<_>>();
    let diagnostics = contract["diagnostics"].as_array().unwrap();
    let expected_count = diagnostics
        .iter()
        .map(|diagnostic| diagnostic["count"].as_u64().unwrap() as usize)
        .sum::<usize>();
    assert_eq!(
        diagnostic_lines.len(),
        expected_count,
        "{}",
        output_text(&output)
    );

    for diagnostic in diagnostics {
        let line = diagnostic["line"].as_u64().unwrap();
        let column = diagnostic["column"].as_u64().unwrap();
        let code = diagnostic["code"].as_u64().unwrap();
        let count = diagnostic["count"].as_u64().unwrap() as usize;
        let needle = cstr!("src/App.vue({},{}): error TS{}:", line, column, code);
        assert_eq!(stdout.matches(needle.as_str()).count(), count, "{stdout}");
    }
    for diagnostic in contract["forbiddenPositions"].as_array().unwrap() {
        let needle = cstr!(
            "src/App.vue({},{}): error TS{}:",
            diagnostic["line"].as_u64().unwrap(),
            diagnostic["column"].as_u64().unwrap(),
            diagnostic["code"].as_u64().unwrap()
        );
        assert!(!stdout.contains(needle.as_str()), "{stdout}");
    }
    for term in contract["forbiddenMessageTerms"].as_array().unwrap() {
        assert!(!stdout.contains(term.as_str().unwrap()), "{stdout}");
    }
    assert!(stdout.contains("Property 'count' is missing"), "{stdout}");
}

#[test]
fn standard_tsgo_contextually_types_multiline_callback_prop() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/content_mapper_multiline_callback");
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/content_mapper_multiline_callback/expectations.json"
    ))
    .unwrap();
    assert_eq!(contract["issue"], 10191);
    assert_eq!(
        contract["typescriptSourceRevision"],
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799"
    );
    assert_eq!(contract["expectedDiagnostics"], json!([]));

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-multiline-callback-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&output);
    assert!(output.stdout.is_empty(), "{}", output_text(&output));
}

#[test]
fn standard_tsgo_resolves_setup_value_in_define_slots_type() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/content_mapper_define_slots_typeof");
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/content_mapper_define_slots_typeof/expectations.json"
    ))
    .unwrap();
    assert_eq!(contract["issue"], 10197);
    assert_eq!(
        contract["typescriptSourceRevision"],
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799"
    );
    assert_eq!(contract["expectedDiagnostics"], json!([]));

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-define-slots-typeof-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&output);
    assert!(output.stdout.is_empty(), "{}", output_text(&output));
}

#[test]
fn standard_tsgo_reuses_nested_v_for_branch_guard() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/content_mapper_nested_vfor_guard");
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/content_mapper_nested_vfor_guard/expectations.json"
    ))
    .unwrap();
    assert_eq!(contract["issue"], 10216);
    assert_eq!(
        contract["typescriptSourceRevision"],
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799"
    );
    assert_eq!(contract["expectedDiagnostics"], json!([]));

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-nested-vfor-guard-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&output);
    assert!(output.stdout.is_empty(), "{}", output_text(&output));
}

#[test]
fn standard_tsgo_keeps_musea_art_in_the_pinned_project_contract() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture_root = workspace_root()
        .join("tests/_fixtures/differential/typechecker/art-variant-implicit-component-9854");
    let fixture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture_root.join("cases.json")).unwrap()).unwrap();
    assert_eq!(
        fixture["sourceRevision"],
        "f07065df35d25bc76f355e5f106c80dbefa922b5"
    );
    let case = &fixture["cases"][0];
    assert_eq!(case["id"], "implicit-and-explicit-props");

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-musea-art-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    for input in case["inputs"].as_array().unwrap() {
        let target = project.path().join(input["file"].as_str().unwrap());
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture_root.join(input["source"].as_str().unwrap()), target).unwrap();
    }
    let config = project.path().join("tsconfig.json");
    let mut config_value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config).unwrap()).unwrap();
    config_value["contentMappers"] = json!([{ "package": "vize", "extensions": [".vue"] }]);
    std::fs::write(&config, serde_json::to_vec_pretty(&config_value).unwrap()).unwrap();

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(!stdout.contains("TS2304"), "{stdout}");
    // Pin the semantic diagnostic set independently of byte-identical reporter duplicates.
    let unique_diagnostics = |code: &str| {
        stdout
            .lines()
            .filter(|line| line.contains("error TS") && line.contains(code))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    assert_eq!(unique_diagnostics("TS2322"), 2, "{stdout}");
    assert_eq!(unique_diagnostics("TS2345"), 2, "{stdout}");
    for diagnostic in case["diagnostics"].as_array().unwrap() {
        let file = diagnostic["file"].as_str().unwrap();
        let line = diagnostic["line"].as_u64().unwrap();
        let column = diagnostic["column"].as_u64().unwrap();
        let code = diagnostic["code"].as_u64().unwrap();
        let needle = cstr!("{file}({line},{column}): error TS{code}:");
        assert!(
            stdout
                .lines()
                .any(|output| output.starts_with(needle.as_str())),
            "missing authored diagnostic {needle}:\n{stdout}"
        );
    }

    for name in ["MyCard.art.vue", "MyCardExplicit.art.vue"] {
        let path = project.path().join("src").join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let valid = source
            .replace(r#":title="123""#, r#"title="hello""#)
            .replace(
                "<MyCard elevated />",
                r#"<MyCard title="hello" elevated />"#,
            );
        std::fs::write(path, valid).unwrap();
    }
    assert_success(&check_project(&tsgo, project.path(), "tsconfig.json"));
}

#[test]
fn standard_tsgo_keeps_filename_component_exports_out_of_setup_globals() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-filename-globals-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.filename-globals.json");
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
}

#[test]
fn standard_tsgo_checks_vue_project_and_emits_consumable_declarations() {
    project_declarations::check_project_and_declarations();
}

mod shared_helpers_performance;
