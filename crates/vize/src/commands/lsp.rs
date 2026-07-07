//! LSP command - Language Server Protocol server

use clap::Args;

#[derive(Args)]
pub struct LspArgs {
    /// Use stdio for communication (default)
    #[arg(long, default_value = "true")]
    pub stdio: bool,

    /// TCP port for socket communication
    #[arg(long)]
    pub port: Option<u16>,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LspTransport {
    Stdio,
    Tcp(u16),
}

pub fn run(args: LspArgs) {
    let result = match transport_from_port(args.port) {
        LspTransport::Stdio => vize_maestro::serve_blocking(),
        LspTransport::Tcp(port) => vize_maestro::serve_tcp_blocking(port),
    };

    if let Err(e) = result {
        eprintln!("LSP server error: {}", e);
        std::process::exit(1);
    }
}

pub(crate) fn transport_from_port(port: Option<u16>) -> LspTransport {
    match port {
        Some(port) => LspTransport::Tcp(port),
        None => LspTransport::Stdio,
    }
}

#[cfg(test)]
mod tests {
    use super::{LspArgs, LspTransport, transport_from_port};

    #[test]
    fn lsp_defaults_to_stdio_transport() {
        let args = LspArgs {
            debug: false,
            port: None,
            stdio: true,
        };

        assert_eq!(transport_from_port(args.port), LspTransport::Stdio);
    }

    #[test]
    fn lsp_port_selects_tcp_transport() {
        let args = LspArgs {
            debug: false,
            port: Some(9257),
            stdio: true,
        };

        assert_eq!(transport_from_port(args.port), LspTransport::Tcp(9257));
    }

    #[test]
    fn lsp_transport_is_independent_from_debug_flag() {
        let args = LspArgs {
            debug: true,
            port: Some(9258),
            stdio: true,
        };

        assert_eq!(transport_from_port(args.port), LspTransport::Tcp(9258));
    }
}
