#![allow(clippy::disallowed_macros)]

use vize_carton::config::load_config;

#[test]
fn loads_pkl_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/VizeConfig.pkl"

formatter {
  singleQuote = true
}

languageServer {
  completion = false
}

typeChecker {
  globalsFile = "./globals.d.ts"
}
"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn loads_json_type_checker_settings() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "typeChecker": {
            "strict": true,
            "globalsFile": "./globals.d.ts"
          },
          "formatter": {
            "singleQuote": true,
            "printWidth": 88
          }
        }"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn loads_legacy_json_lsp_alias() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "lsp": {
            "completion": false,
            "tsgo": false
          }
        }"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

fn install_pkl_modules(root: &std::path::Path) {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm/vize/pkl");
    let target = root.join("node_modules/vize/pkl");
    copy_dir_recursive(&source, &target);
}

fn copy_dir_recursive(source: &std::path::Path, target: &std::path::Path) {
    std::fs::create_dir_all(target).unwrap();

    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        let target_path = target.join(entry.file_name());

        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry_path, &target_path);
        } else {
            std::fs::copy(&entry_path, &target_path).unwrap();
        }
    }
}

#[test]
fn loads_legacy_json_aliases() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "fmt": {
            "singleQuote": true
          },
          "check": {
            "globals": "./globals.d.ts"
          }
        }"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn loads_legacy_pkl_lsp_alias() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/VizeConfig.pkl"

lsp {
  completion = false
  tsgo = false
}
"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}
