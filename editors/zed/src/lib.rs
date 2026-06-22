use std::collections::HashMap;

use zed_extension_api::{
    self as zed, Result,
    settings::{CommandSettings, LspSettings},
};

struct VizeExtension;

impl VizeExtension {
    const SERVER_NAME: &'static str = "vize";
    const SERVER_BINARY: &'static str = "vize";

    fn server_command_from_settings(
        settings: LspSettings,
        discovered_server_path: Option<String>,
        shell_env: zed::EnvVars,
    ) -> Result<zed::Command> {
        let binary = settings.binary;
        let command = binary
            .as_ref()
            .and_then(configured_binary_path)
            .or(discovered_server_path)
            .ok_or_else(Self::missing_server_message)?;

        let args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_else(|| vec!["lsp".to_string()]);
        let env = merge_env(shell_env, binary.and_then(|binary| binary.env));

        Ok(zed::Command { command, args, env })
    }

    fn missing_server_message() -> String {
        format!(
            "Could not find the `{}` binary. Install the Vize CLI or configure lsp.{}.binary.path.",
            Self::SERVER_BINARY,
            Self::SERVER_NAME
        )
    }
}

impl zed::Extension for VizeExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Self::server_command_from_settings(
            settings,
            worktree.which(Self::SERVER_BINARY),
            worktree.shell_env(),
        )
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.settings)
    }
}

zed::register_extension!(VizeExtension);

fn configured_binary_path(binary: &CommandSettings) -> Option<String> {
    let path = binary.path.as_deref()?.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

fn merge_env(shell_env: zed::EnvVars, custom_env: Option<HashMap<String, String>>) -> zed::EnvVars {
    let Some(custom_env) = custom_env else {
        return shell_env;
    };

    let mut env = shell_env
        .into_iter()
        .filter(|(key, _)| !custom_env.contains_key(key))
        .collect::<zed::EnvVars>();
    let mut custom_env = custom_env.into_iter().collect::<Vec<_>>();
    custom_env.sort_by(|(left, _), (right, _)| left.cmp(right));
    env.extend(custom_env);
    env
}

#[cfg(test)]
mod tests {
    use super::{CommandSettings, LspSettings, VizeExtension, zed};

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

    fn settings_with_binary(binary: CommandSettings) -> LspSettings {
        LspSettings {
            binary: Some(binary),
            ..LspSettings::default()
        }
    }

    fn binary(
        path: Option<&str>,
        arguments: Option<&[&str]>,
        env: &[(&str, &str)],
    ) -> CommandSettings {
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
}
