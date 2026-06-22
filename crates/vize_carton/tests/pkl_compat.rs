#![allow(clippy::disallowed_macros)]

use vize_carton::config::load_config_with_features_and_source;

#[test]
fn loads_documented_pkl_compat_schema_without_deserialize_warnings() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/vize.pkl"

formatter {
  printWidth = 100
}

linter {
  preset = "opinionated"
  rules = new Mapping {
    ["vue/no-v-html"] = "off"
  }
}

typeChecker {
  checkProps = false
}

languageServer {
  enabled = true
}
"#,
    )
    .unwrap();

    let loaded = load_config_with_features_and_source(Some(dir.path()));
    let linter = vize_carton::config::load_linter_config(Some(dir.path()));

    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert_eq!(loaded.config.formatter.print_width, 100);
    assert!(!loaded.config.type_checker.check_props);
    assert_eq!(loaded.config.language_server.enabled, Some(true));
    assert_eq!(linter.preset.as_deref(), Some("opinionated"));
    assert_eq!(linter.disabled_rules(), ["vue/no-v-html"]);
}

fn install_pkl_modules(root: &std::path::Path) {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm/cli/pkl");
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
