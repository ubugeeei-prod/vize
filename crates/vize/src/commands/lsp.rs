//! LSP command - Language Server Protocol server

use clap::Args;
use std::path::{Path, PathBuf};
use vize_l0::{String, cstr};

#[derive(Args)]
pub struct LspArgs {
    /// Use stdio for communication (default)
    #[arg(long, default_value = "true")]
    pub stdio: bool,

    /// TCP port for socket communication
    #[arg(long)]
    pub port: Option<u16>,

    /// File holding a token the client's `initialize` must carry as
    /// `initializationOptions.vizeToken` (with --port; overrides VIZE_LSP_TOKEN)
    #[arg(long, value_name = "PATH", requires = "port")]
    pub token_file: Option<PathBuf>,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,

    #[command(flatten)]
    pub profile_export: super::profile_export::ProfileExportArgs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LspTransport {
    Stdio,
    Tcp(u16),
}

pub fn run(args: LspArgs) {
    args.profile_export.begin(false);
    let result = match transport_from_port(args.port) {
        LspTransport::Stdio => vize_maestro::serve_blocking(),
        LspTransport::Tcp(port) => serve_tcp(port, args.token_file.as_deref()),
    };
    args.profile_export.finish("lsp", false);

    if let Err(e) = result {
        eprintln!("LSP server error: {}", e);
        std::process::exit(1);
    }
}

/// Environment variable holding the pairing token when no `--token-file` is
/// given.
const TOKEN_ENV: &str = "VIZE_LSP_TOKEN";

pub(crate) fn serve_tcp(
    port: u16,
    token_file: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let token = pairing_token(token_file, std::env::var_os(TOKEN_ENV))
        .map_err(|message| Box::<dyn std::error::Error + Send + Sync>::from(message.as_str()))?;
    vize_maestro::serve_tcp_blocking(port, token)
}

/// The pairing token from `token_file`, else from the environment value.
pub(crate) fn pairing_token(
    token_file: Option<&Path>,
    env: Option<std::ffi::OsString>,
) -> Result<Option<vize_maestro::PairingToken>, String> {
    let Some(path) = token_file else {
        let Some(value) = env else {
            return Ok(None);
        };
        let token = value
            .to_str()
            .and_then(vize_maestro::PairingToken::new)
            .ok_or_else(|| cstr!("{TOKEN_ENV} must hold a non-empty UTF-8 token"))?;
        return Ok(Some(token));
    };
    let bytes = std::fs::read(path)
        .map_err(|error| cstr!("cannot read --token-file {}: {error}", path.display()))?;
    std::str::from_utf8(&bytes)
        .ok()
        .and_then(vize_maestro::PairingToken::new)
        .map(Some)
        .ok_or_else(|| {
            cstr!(
                "--token-file {} must hold a non-empty UTF-8 token",
                path.display()
            )
        })
}

pub(crate) fn transport_from_port(port: Option<u16>) -> LspTransport {
    match port {
        Some(port) => LspTransport::Tcp(port),
        None => LspTransport::Stdio,
    }
}

#[cfg(test)]
mod tests {
    use super::{LspArgs, LspTransport, pairing_token, transport_from_port};
    use clap::Parser;

    #[test]
    fn lsp_defaults_to_stdio_transport() {
        let args = LspArgs {
            debug: false,
            port: None,
            token_file: None,
            stdio: true,
            profile_export: Default::default(),
        };

        assert_eq!(transport_from_port(args.port), LspTransport::Stdio);
    }

    #[test]
    fn lsp_port_selects_tcp_transport() {
        let args = LspArgs {
            debug: false,
            port: Some(9257),
            token_file: None,
            stdio: true,
            profile_export: Default::default(),
        };

        assert_eq!(transport_from_port(args.port), LspTransport::Tcp(9257));
    }

    #[test]
    fn lsp_transport_is_independent_from_debug_flag() {
        let args = LspArgs {
            debug: true,
            port: Some(9258),
            token_file: None,
            stdio: true,
            profile_export: Default::default(),
        };

        assert_eq!(transport_from_port(args.port), LspTransport::Tcp(9258));
    }

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        lsp: LspArgs,
    }

    #[test]
    fn a_token_file_needs_a_port() {
        assert!(Cli::try_parse_from(["vize", "--token-file", "token"]).is_err());
        let cli = Cli::try_parse_from(["vize", "--port", "9527", "--token-file", "token"]).unwrap();
        assert_eq!(
            cli.lsp.token_file.as_deref(),
            Some(std::path::Path::new("token"))
        );
    }

    #[test]
    fn the_token_file_wins_over_the_environment() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("token");
        std::fs::write(&file, "from-file\n").unwrap();

        let token = |text| vize_maestro::PairingToken::new(text);
        assert_eq!(pairing_token(None, None).unwrap(), None);
        assert_eq!(
            pairing_token(None, Some("from-env".into())).unwrap(),
            token("from-env")
        );
        assert!(pairing_token(None, Some(" ".into())).is_err());
        assert_eq!(
            pairing_token(Some(&file), Some("from-env".into())).unwrap(),
            token("from-file")
        );

        std::fs::write(&file, "\n").unwrap();
        assert!(pairing_token(Some(&file), None).is_err());
        assert!(pairing_token(Some(&dir.path().join("missing")), None).is_err());
    }
}
