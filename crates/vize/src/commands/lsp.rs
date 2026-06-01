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

pub fn run(args: LspArgs) {
    let result = if let Some(port) = args.port {
        vize_maestro::serve_tcp_blocking(port)
    } else {
        vize_maestro::serve_blocking()
    };

    if let Err(e) = result {
        eprintln!("LSP server error: {}", e);
        std::process::exit(1);
    }
}
