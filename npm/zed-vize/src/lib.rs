use zed_extension_api::{self as zed, settings::LspSettings, Result};

struct VizeExtension;

impl VizeExtension {
    const SERVER_NAME: &'static str = "vize";
    const SERVER_BINARY: &'static str = "vize";
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
        let binary = settings.binary;
        let command = binary
            .as_ref()
            .and_then(|binary| binary.path.clone())
            .or_else(|| worktree.which(Self::SERVER_BINARY))
            .ok_or_else(|| {
                format!(
                    "Could not find the `{}` binary. Install the Vize CLI or configure lsp.{}.binary.path.",
                    Self::SERVER_BINARY,
                    Self::SERVER_NAME
                )
            })?;

        let args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_else(|| vec!["lsp".to_string()]);

        let mut env = worktree.shell_env();
        if let Some(custom_env) = binary.and_then(|binary| binary.env) {
            env.extend(custom_env);
        }

        Ok(zed::Command { command, args, env })
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
