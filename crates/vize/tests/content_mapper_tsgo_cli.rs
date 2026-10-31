#![cfg(test)]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::json;
use vize_l0::{String as CompactString, cstr};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const JAVASCRIPT_TSC_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

mod const_enum_import;
mod contextual_prop_anchors;
mod dynamic_component_event_names;
mod dynamic_template_refs;
mod generic_constructor_slots;
mod global_components;
mod incomplete_tags;
mod javascript_check_js;
mod long_prop_kebab_case;
mod optional_prop_spacing;
mod options_api_diagnostics;
mod props_binding;
mod pug_refusal;
mod quoted_generic_parameters;
mod support;
mod template_callback_event_navigation;
mod template_expression_syntax;
mod type_only_import_use;
mod typed_context_undefined;
mod unicode_prop_bindings;
mod unknown_options_10218;

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
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );
    let javascript_tsc = std::env::var_os(JAVASCRIPT_TSC_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{JAVASCRIPT_TSC_ENV} must accompany {TSGO_ENV}"));
    assert!(
        javascript_tsc.is_file(),
        "{JAVASCRIPT_TSC_ENV} is not a file: {}",
        javascript_tsc.display()
    );

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-tsgo-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let check = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&check);

    let options_enabled = check_project(&tsgo, project.path(), "tsconfig.options-api-enabled.json");
    assert_success(&options_enabled);

    let options_disabled =
        check_project(&tsgo, project.path(), "tsconfig.options-api-disabled.json");
    assert!(
        !options_disabled.status.success(),
        "Options API disabled fixture passed unexpectedly"
    );
    assert_eq!(
        options_api_diagnostics::raw_lines(&options_disabled),
        options_api_diagnostics::DISABLED_OPTIONS_API,
        "Options API opt-out diagnostics must retain their exact authored identity: {}",
        output_text(&options_disabled)
    );

    let broken = check_project(&tsgo, project.path(), "tsconfig.error.json");
    assert!(
        !broken.status.success(),
        "broken fixture passed unexpectedly"
    );
    let broken_output = output_text(&broken);
    assert!(
        broken_output.contains("errors/Broken.vue"),
        "{broken_output}"
    );
    assert!(
        broken_output.contains("TS2322")
            && broken_output.contains("not assignable to type 'number'"),
        "{broken_output}"
    );
    for script_error in ["errors/JavaScriptConsumer.js", "errors/JsxConsumer.jsx"] {
        assert!(
            broken_output
                .lines()
                .any(|line| line.contains(script_error) && line.contains("TS2322")),
            "{script_error} was not checked:\n{broken_output}"
        );
    }
    assert!(
        broken_output.contains("src/Unused.vue")
            && broken_output.contains("TS6133")
            && broken_output.contains("'unused' is declared but its value is never read"),
        "{broken_output}"
    );
    assert!(
        !broken_output.contains("'used' is declared"),
        "{broken_output}"
    );
    assert_eq!(
        broken_output.matches("TS6133").count(),
        1,
        "{broken_output}"
    );
    assert!(!broken_output.contains("TS6196"), "{broken_output}");

    let emit = run_tsgo(
        &tsgo,
        project.path(),
        &[
            "--runExternalCode",
            "-p",
            "tsconfig.emit.json",
            "--pretty",
            "false",
        ],
    );
    assert_success(&emit);

    let app_declaration = emitted_vue_declaration(project.path(), "App");
    let child_declaration = emitted_vue_declaration(project.path(), "Child");
    let options_declaration = emitted_vue_declaration(project.path(), "Options");
    let public_declaration = emitted_vue_declaration(project.path(), "Public");
    for declaration in [&app_declaration, &child_declaration] {
        let text = std::fs::read_to_string(declaration).unwrap();
        assert!(
            text.contains("$props"),
            "{}:\n{text}",
            declaration.display()
        );
        assert!(
            text.contains("count: number"),
            "{}:\n{text}",
            declaration.display()
        );
    }
    let options_text = std::fs::read_to_string(&options_declaration).unwrap();
    assert!(options_text.contains("$props"), "{options_text}");
    assert!(
        options_text.contains("declare const Options: typeof __vize_component__")
            && options_text.contains("export default Options"),
        "{options_text}"
    );
    let public_text = std::fs::read_to_string(&public_declaration).unwrap();
    for public_surface in ["$props", "$emit", "$slots", "focus", "modelValue"] {
        assert!(
            public_text.contains(public_surface),
            "missing {public_surface} in {}:\n{public_text}",
            public_declaration.display()
        );
    }

    let main_declaration = project.path().join("dist/main.d.ts");
    let main_text = std::fs::read_to_string(&main_declaration).unwrap();
    assert!(main_text.contains("./App.vue"), "{main_text}");
    assert!(main_text.contains("export type AppProps"), "{main_text}");

    std::fs::copy(
        project.path().join("consumer/verify.ts"),
        project.path().join("dist/verify.ts"),
    )
    .unwrap();
    let consume = run_tsgo(
        &tsgo,
        project.path(),
        &[
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--module",
            "preserve",
            "--moduleResolution",
            "bundler",
            "--allowArbitraryExtensions",
            "--pretty",
            "false",
            "dist/verify.ts",
        ],
    );
    assert_success(&consume);

    let consume = run_tsgo(
        &javascript_tsc,
        project.path(),
        &[
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--module",
            "preserve",
            "--moduleResolution",
            "bundler",
            "--allowArbitraryExtensions",
            "--pretty",
            "false",
            "dist/verify.ts",
        ],
    );
    assert_success(&consume);
}

#[test]
fn standard_tsgo_accepts_surplus_defaults_without_erasing_declared_default_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/content-mapper/with-defaults-surplus.json"
    ))
    .unwrap();
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-surplus-defaults-")
        .tempdir_in(root)
        .unwrap();
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "pr".into());
    let evidence = workspace_root()
        .join("target/nextest")
        .join(profile)
        .join("content-mapper-surplus-defaults");
    std::fs::create_dir_all(&evidence).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let directory = project.path().join(name);
        std::fs::create_dir_all(&directory).unwrap();
        install_mapper_manifest(&directory);
        install_vue_package(&directory);
        for (file, source) in case["files"].as_object().unwrap() {
            std::fs::write(directory.join(file), source.as_str().unwrap()).unwrap();
        }
        std::fs::write(
            directory.join("tsconfig.json"),
            serde_json::to_vec_pretty(&json!({
                "compilerOptions": {
                    "strict": true, "module": "ESNext",
                    "moduleResolution": "bundler", "skipLibCheck": true
                },
                "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                "include": ["*.vue", "*.ts"]
            }))
            .unwrap(),
        )
        .unwrap();
        let output = check_project(&tsgo, &directory, "tsconfig.json");
        std::fs::write(
            evidence.join(cstr!("{name}.json").as_str()),
            serde_json::to_vec_pretty(&json!({
                "case": case, "host": tsgo, "sourceSha": std::env::var("GITHUB_SHA").ok(),
                "exitCode": output.status.code(), "success": output.status.success(),
                "stdoutBytes": output.stdout, "stderrBytes": output.stderr,
            }))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output.stderr, b"", "{}", output_text(&output));
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        if case["codes"] == json!([]) {
            assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
            assert_eq!(stdout, "", "{}", output_text(&output));
        } else {
            let line = if matches!(name.split('-').next(), Some("imported" | "mapped")) {
                4
            } else {
                3
            };
            assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
            let lines = stdout.lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), 1, "{}", output_text(&output));
            let (identity, message) = lines[0].split_once(": error TS2322: ").unwrap();
            assert_eq!(identity, cstr!("App.vue({line},3)").as_str());
            // Vize and Vue keep their own helper names in the RHS type text;
            // the authored error identity must still be exact and singular.
            assert!(!message.is_empty(), "{}", output_text(&output));
        }
    }
}

mod shared_helpers_performance;
