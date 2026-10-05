use super::{
    initialization_options::{WORKSPACE_CONFIG_FILES, initialization_options},
    recommended_initialization_options,
    zed::serde_json::json,
};

#[test]
fn every_supported_root_config_defers_to_the_server() {
    for present in WORKSPACE_CONFIG_FILES {
        assert_eq!(
            initialization_options(None, |filename| filename == present),
            json!({}),
            "{present} must keep the authored languageServer settings"
        );
    }
}

#[test]
fn explicit_options_are_preserved_without_probing_the_worktree() {
    for explicit in [
        json!({}),
        json!({ "formatting": true }),
        json!({ "hover": false, "typecheck": false, "custom": [1, "x"] }),
        json!(null),
        recommended_initialization_options(),
    ] {
        assert_eq!(
            initialization_options(Some(explicit.clone()), |_| {
                panic!("explicit settings must not read workspace files")
            }),
            explicit
        );
    }
}

#[test]
fn absent_configs_keep_the_recommended_default() {
    let mut probed = Vec::new();
    assert_eq!(
        initialization_options(None, |filename| {
            probed.push(filename.to_string());
            false
        }),
        recommended_initialization_options()
    );
    assert_eq!(probed, WORKSPACE_CONFIG_FILES);
}

#[test]
fn unrelated_or_parent_paths_do_not_select_workspace_config() {
    for unrelated in [
        "vize.config.yaml",
        "../vize.config.ts",
        "src/vize.config.ts",
    ] {
        assert_eq!(
            initialization_options(None, |filename| filename == unrelated),
            recommended_initialization_options()
        );
    }
}
