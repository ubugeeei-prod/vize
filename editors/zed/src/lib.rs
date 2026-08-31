use std::collections::HashMap;

use zed_extension_api::{
    self as zed,
    settings::{CommandSettings, LspSettings},
    Result,
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
        Ok(settings
            .initialization_options
            .or_else(|| Some(recommended_initialization_options())))
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

fn recommended_initialization_options() -> zed::serde_json::Value {
    zed::serde_json::json!({
        "editor": true,
        "ecosystem": true,
        "lint": true,
        "typecheck": true,
    })
}

#[cfg(test)]
#[path = "../../../tools/rust/zed_extension_unit_tests.rs"]
mod tests;
