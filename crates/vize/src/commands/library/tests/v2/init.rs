//! Project configuration initialization and preservation.

use std::fs;

use super::super::fixture::{Project, ui_v1};

#[test]
fn init_detects_the_layout_and_writes_the_lib_section() {
    let project = Project::new();
    fs::create_dir_all(project.path("src")).unwrap();
    project.write(
        "tsconfig.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );

    let preview = project.json(&[], &["init", "--dry-run"]);
    assert_eq!(preview["action"], "create");
    assert_eq!(preview["sourceDir"], "src");
    assert_eq!(preview["lib"]["uiDir"], "src/components/vize");
    assert!(
        preview["hints"][0]
            .as_str()
            .unwrap()
            .contains("allowImportingTsExtensions")
    );
    assert!(!project.path("vize.config.json").exists());
    assert!(!project.path("vite.config.mjs").exists());

    project.run(&["init", "--ui-dir", "src/ui"]).unwrap();
    let text = project.read("vite.config.mjs");
    let config: serde_json::Value = serde_json::from_str(
        text.trim()
            .strip_prefix("export default ")
            .unwrap()
            .strip_suffix(';')
            .unwrap(),
    )
    .unwrap();
    assert_eq!(config["vue"]["lib"]["uiDir"], "src/ui");
    assert_eq!(
        config["vue"]["lib"]["composableDir"],
        "src/composables/vize"
    );
    assert!(!project.path("vize.config.json").exists());

    let again = project.json(&[], &["init"]);
    assert_eq!(again["action"], "unchanged");

    ui_v1(&project.registry("ui"));
    project.run_with(&["ui"], &["pull", "id"]).unwrap();
    assert!(project.path("src/ui/foundations/id/id.ts").is_file());
}

#[test]
fn init_keeps_existing_vite_config_and_suggests_the_vue_lib_section() {
    let project = Project::new();
    let source = "export default { plugins: [], server: { port: 5173 } };\n";
    project.write("vite.config.mts", source);

    let report = project.json(&[], &["init"]);
    assert_eq!(report["action"], "manual");
    assert_eq!(report["configPath"], "vite.config.mts");
    assert!(
        report["hints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hint| { hint.as_str().unwrap().contains("vue: { lib: { uiDir:") })
    );
    assert_eq!(project.read("vite.config.mts"), source);
    assert!(!project.path("vize.config.json").exists());
    assert!(!project.path("vite.config.mjs").exists());
}

#[test]
fn init_rejects_removed_dedicated_configs_before_writing() {
    for (filename, source) in [
        (
            "vize.config.json",
            "{\n  \"formatter\": { \"semi\": false }\n}\n",
        ),
        (
            "vize.config.ts",
            "export default { formatter: { semi: false } };\n",
        ),
    ] {
        let project = Project::new();
        project.write(filename, source);

        let error = project.run(&["init"]).unwrap_err();

        assert!(
            error
                .message()
                .contains("vize.config.* is no longer supported"),
            "{error}"
        );
        assert!(error.message().contains("vue: in vite.config.*"), "{error}");
        assert_eq!(project.read(filename), source);
        assert!(!project.path("vite.config.mjs").exists());
        assert!(!project.path("vize-lib.lock.json").exists());
        assert!(!project.path("src").exists());
        assert_eq!(fs::read_dir(&project.root).unwrap().count(), 2);
    }
}

#[test]
fn init_preserves_shared_code_configs_even_when_force_requests_replacement() {
    for filename in ["vite.config.mjs", "vite.config.ts"] {
        let project = Project::new();
        let source = "export default { vue: { formatter: { semi: false }, lib: { uiDir: \"src/custom\" } } };\n";
        project.write(filename, source);

        let existing = project.json(&[], &["init"]);
        assert_eq!(existing["action"], "unchanged");
        assert_eq!(existing["configPath"], filename);
        assert_eq!(project.read(filename), source);

        let replacement = project.json(&[], &["init", "--force", "--ui-dir", "src/ui"]);
        assert_eq!(replacement["action"], "manual");
        assert_eq!(replacement["configPath"], filename);
        assert_eq!(replacement["lib"]["uiDir"], "src/ui");
        assert!(replacement["hints"].as_array().unwrap().iter().any(|hint| {
            hint.as_str()
                .unwrap()
                .contains("vue: { lib: { uiDir: \"src/ui\"")
        }));
        assert_eq!(project.read(filename), source);
        assert!(!project.path("vize.config.json").exists());
        assert!(!project.path("vize-lib.lock.json").exists());
        assert_eq!(fs::read_dir(&project.root).unwrap().count(), 2);
    }
}

#[cfg(unix)]
#[test]
fn init_rejects_symlinked_config_outside_project() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("vite.config.mjs");
    let original = "export default { vue: { formatter: { semi: false } } };\n";
    fs::write(&target, original).unwrap();
    symlink(&target, project.path("vite.config.mjs")).unwrap();

    let error = project.run(&["init", "--force"]).unwrap_err();
    assert!(error.message().contains("symbolic link"), "{error}");
    assert_eq!(fs::read_to_string(target).unwrap(), original);
}

#[test]
fn init_preserves_unparseable_vite_config() {
    let project = Project::new();
    let invalid = "export default { vue: { invalid syntax\n";
    project.write("vite.config.mjs", invalid);

    let error = project.run(&["init"]).unwrap_err();

    assert!(error.message().contains("failed to parse"), "{error}");
    assert!(error.message().contains("vite.config.mjs"), "{error}");
    assert_eq!(project.read("vite.config.mjs"), invalid);
    assert!(!project.path("vize.config.json").exists());
    assert!(!project.path("vize-lib.lock.json").exists());
    assert_eq!(fs::read_dir(&project.root).unwrap().count(), 2);
}
