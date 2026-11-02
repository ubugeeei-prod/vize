use super::{
    JAVASCRIPT_TSC_ENV, TSGO_ENV, assert_success, check_project, copy_fixture,
    emitted_vue_declaration, install_mapper_manifest, install_vue_package, options_api_diagnostics,
    output_text, run_tsgo, workspace_root,
};
use std::path::{Path, PathBuf};

pub(super) fn check_project_and_declarations() {
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
