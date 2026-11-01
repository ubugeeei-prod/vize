use super::{
    PathBuf, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, json,
    output_text, workspace_root,
};

#[test]
fn standard_tsgo_accepts_surplus_defaults_without_erasing_declared_default_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/content-mapper/with-defaults-surplus.json"
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
