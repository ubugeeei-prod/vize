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
    assert_eq!(config["vize"]["lib"]["uiDir"], "src/ui");
    assert_eq!(
        config["vize"]["lib"]["composableDir"],
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
fn init_keeps_existing_vite_config_and_suggests_the_vize_lib_section() {
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
            .any(|hint| { hint.as_str().unwrap().contains("vize: { lib: { uiDir:") })
    );
    assert_eq!(project.read("vite.config.mts"), source);
    assert!(!project.path("vize.config.json").exists());
    assert!(!project.path("vite.config.mjs").exists());
}

#[test]
fn init_appends_to_existing_json_config_and_defers_on_ts_config() {
    let project = Project::new();
    project.write(
        "vize.config.json",
        "{\n  \"formatter\": { \"semi\": false }\n}\n",
    );
    project.run(&["init"]).unwrap();
    let text = project.read("vize.config.json");
    assert!(
        text.starts_with("{\n  \"formatter\": { \"semi\": false },\n  \"lib\": {"),
        "{text}"
    );

    let ts = Project::new();
    ts.write("vize.config.ts", "export default {};\n");
    let report = ts.json(&[], &["init"]);
    assert_eq!(report["action"], "manual");
    assert!(
        report["hints"][0]
            .as_str()
            .unwrap()
            .contains("uiDir: \"src/components/vize\"")
    );
    assert_eq!(ts.read("vize.config.ts"), "export default {};\n");
}

#[cfg(unix)]
#[test]
fn init_rejects_symlinked_config_outside_project() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("vize.config.json");
    let original = "{ \"formatter\": { \"semi\": false } }\n";
    fs::write(&target, original).unwrap();
    symlink(&target, project.path("vize.config.json")).unwrap();

    let error = project.run(&["init", "--force"]).unwrap_err();
    assert!(error.message().contains("symbolic link"), "{error}");
    assert_eq!(fs::read_to_string(target).unwrap(), original);
}

#[test]
fn init_preserves_unparseable_json_config() {
    let project = Project::new();
    let invalid = "{ invalid json\n";
    project.write("vize.config.json", invalid);

    assert!(project.run(&["init"]).is_err());
    assert_eq!(project.read("vize.config.json"), invalid);
}
