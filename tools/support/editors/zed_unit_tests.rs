use super::{
    configured_binary_path, merge_env, recommended_initialization_options, zed, CommandSettings,
    LspSettings, VizeExtension,
};

#[test]
fn discovered_binary_defaults_to_lsp() {
    let command = VizeExtension::server_command_from_settings(
        LspSettings::default(),
        Some("/usr/local/bin/vize".to_string()),
        env_vars([("PATH", "/usr/bin")]),
    )
    .unwrap();

    assert_eq!(command.command, "/usr/local/bin/vize");
    assert_eq!(command.args, vec!["lsp"]);
    assert_eq!(command.env, env_vars([("PATH", "/usr/bin")]));
}

#[test]
fn configured_binary_path_wins_over_path_lookup() {
    let command = VizeExtension::server_command_from_settings(
        settings_with_binary(binary(Some(" /opt/vize/bin/vize "), None, &[])),
        Some("/usr/local/bin/vize".to_string()),
        env_vars([]),
    )
    .unwrap();

    assert_eq!(command.command, "/opt/vize/bin/vize");
    assert_eq!(command.args, vec!["lsp"]);
}

#[test]
fn configured_binary_path_trims_surrounding_whitespace() {
    let configured = CommandSettings {
        arguments: None,
        env: None,
        path: Some("\n\t/opt/vize/bin/vize  ".to_string()),
    };

    assert_eq!(
        configured_binary_path(&configured),
        Some("/opt/vize/bin/vize".to_string())
    );
}

#[test]
fn configured_binary_path_rejects_blank_values() {
    for path in ["", " ", "\n\t"] {
        let configured = CommandSettings {
            arguments: None,
            env: None,
            path: Some(path.to_string()),
        };

        assert_eq!(configured_binary_path(&configured), None);
    }
}

#[test]
fn configured_binary_path_rejects_absent_values() {
    let configured = CommandSettings {
        arguments: None,
        env: None,
        path: None,
    };

    assert_eq!(configured_binary_path(&configured), None);
}

#[test]
fn configured_arguments_and_env_override_defaults() {
    let command = VizeExtension::server_command_from_settings(
        settings_with_binary(binary(
            Some("/opt/vize/bin/vize"),
            Some(&["lsp", "--debug"]),
            &[("PATH", "/custom/bin"), ("VIZE_LOG", "trace")],
        )),
        Some("/usr/local/bin/vize".to_string()),
        env_vars([("PATH", "/usr/bin"), ("RUST_LOG", "info")]),
    )
    .unwrap();

    assert_eq!(command.command, "/opt/vize/bin/vize");
    assert_eq!(command.args, vec!["lsp", "--debug"]);
    assert_eq!(
        command.env,
        env_vars([
            ("RUST_LOG", "info"),
            ("PATH", "/custom/bin"),
            ("VIZE_LOG", "trace"),
        ])
    );
}

#[test]
fn explicit_empty_arguments_are_preserved() {
    let command = VizeExtension::server_command_from_settings(
        settings_with_binary(binary(Some("/opt/vize/bin/vize"), Some(&[]), &[])),
        None,
        env_vars([]),
    )
    .unwrap();

    assert_eq!(command.args, Vec::<String>::new());
}

#[test]
fn configured_arguments_without_path_apply_to_discovered_binary() {
    let command = VizeExtension::server_command_from_settings(
        settings_with_binary(binary(None, Some(&["lsp", "--debug"]), &[])),
        Some("/usr/local/bin/vize".to_string()),
        env_vars([]),
    )
    .unwrap();

    assert_eq!(command.command, "/usr/local/bin/vize");
    assert_eq!(command.args, vec!["lsp", "--debug"]);
}

#[test]
fn blank_configured_path_falls_back_to_path_lookup() {
    let command = VizeExtension::server_command_from_settings(
        settings_with_binary(binary(Some(" \t "), None, &[])),
        Some("/usr/local/bin/vize".to_string()),
        env_vars([]),
    )
    .unwrap();

    assert_eq!(command.command, "/usr/local/bin/vize");
    assert_eq!(command.args, vec!["lsp"]);
}

#[test]
fn missing_binary_reports_install_and_settings_guidance() {
    let error = VizeExtension::server_command_from_settings(
        settings_with_binary(binary(None, None, &[])),
        None,
        env_vars([]),
    )
    .unwrap_err();

    assert!(error.contains("Install the Vize CLI"));
    assert!(error.contains("lsp.vize.binary.path"));
}

#[test]
fn merge_env_without_custom_env_preserves_shell_env() {
    assert_eq!(
        merge_env(env_vars([("PATH", "/usr/bin"), ("RUST_LOG", "info")]), None),
        env_vars([("PATH", "/usr/bin"), ("RUST_LOG", "info")])
    );
}

#[test]
fn merge_env_removes_shell_keys_replaced_by_custom_env() {
    assert_eq!(
        merge_env(
            env_vars([("PATH", "/usr/bin"), ("VIZE_LOG", "info")]),
            Some(
                [("PATH".to_string(), "/custom/bin".to_string())]
                    .into_iter()
                    .collect(),
            ),
        ),
        env_vars([("VIZE_LOG", "info"), ("PATH", "/custom/bin")])
    );
}

#[test]
fn merge_env_sorts_custom_env_for_stable_commands() {
    assert_eq!(
        merge_env(
            env_vars([("PATH", "/usr/bin")]),
            Some(
                [
                    ("VIZE_Z".to_string(), "z".to_string()),
                    ("VIZE_A".to_string(), "a".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        ),
        env_vars([("PATH", "/usr/bin"), ("VIZE_A", "a"), ("VIZE_Z", "z")])
    );
}

#[test]
fn default_initialization_options_are_recommended_profile() {
    assert_eq!(
        recommended_initialization_options(),
        zed::serde_json::json!({
            "editor": true,
            "ecosystem": true,
            "lint": true,
            "typecheck": true,
        })
    );
}

fn settings_with_binary(binary: CommandSettings) -> LspSettings {
    LspSettings {
        binary: Some(binary),
        ..LspSettings::default()
    }
}

fn binary(path: Option<&str>, arguments: Option<&[&str]>, env: &[(&str, &str)]) -> CommandSettings {
    CommandSettings {
        arguments: arguments.map(|arguments| {
            arguments
                .iter()
                .copied()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        }),
        env: if env.is_empty() {
            None
        } else {
            Some(
                env.into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
            )
        },
        path: path.map(ToString::to_string),
    }
}

fn env_vars<const N: usize>(env: [(&str, &str); N]) -> zed::EnvVars {
    env.into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}
