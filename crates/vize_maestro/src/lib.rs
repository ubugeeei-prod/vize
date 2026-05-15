//! # vize_maestro
//!
//! Maestro - Language Server Protocol implementation for Vize.
//!
//! ## Name Origin
//!
//! **Maestro** is a master conductor who coordinates an orchestra,
//! bringing together all the instruments in harmony. Similarly,
//! `vize_maestro` orchestrates all the Vize compiler tools to provide
//! a seamless IDE experience through the Language Server Protocol.
//!
//! ## Architecture
//!
//! ```text
//! +------------------------------------------------------------------+
//! |                        vize_maestro (LSP Server)                  |
//! +------------------------------------------------------------------+
//! |                                                                    |
//! |  +--------------------+     +-------------------+                  |
//! |  |   LSP Transport    |     |   Server Core     |                  |
//! |  |   (tower-lsp)      |<--->|   (request/event) |                  |
//! |  +--------------------+     +-------------------+                  |
//! |                                      |                             |
//! |                                      v                             |
//! |  +-----------------------------------------------------------+    |
//! |  |                   Document Store                           |    |
//! |  |  (Rope-based efficient text storage)                       |    |
//! |  +-----------------------------------------------------------+    |
//! |                                      |                             |
//! |                                      v                             |
//! |  +-----------------------------------------------------------+    |
//! |  |                   Virtual Code Layer                       |    |
//! |  |  SFC → Virtual Documents (template.ts, script.ts, css)     |    |
//! |  |  SourceMap for bidirectional position mapping              |    |
//! |  +-----------------------------------------------------------+    |
//! |                                      |                             |
//! |                                      v                             |
//! |  +-----------------------------------------------------------+    |
//! |  |                    Syntax Analysis Layer                   |    |
//! |  |  vize_atelier_sfc | vize_armature | vize_relief            |    |
//! |  +-----------------------------------------------------------+    |
//! +------------------------------------------------------------------+
//! ```
//!
//! ## Features
//!
//! - LSP server implementation for Vue SFC files
//! - Code completion and IntelliSense
//! - Go to definition and references
//! - Hover information
//! - Diagnostics and error reporting
//! - Code actions and quick fixes
//! - Rename refactoring
//! - Document symbols and outline
//!
//! ## Usage
//!
//! ```no_run
//! #[tokio::main]
//! async fn main() {
//!     vize_maestro::serve().await.unwrap();
//! }
//! ```

pub mod document;
pub mod ide;
pub mod server;
pub mod utils;
pub mod virtual_code;

pub use ide::{
    CodeActionService, CodeLensService, CompletionService, DefinitionService, DiagnosticService,
    HoverService, IdeContext, ReferencesService, RenameService, SemanticTokensService, TypeService,
    WorkspaceSymbolsService,
};
pub use server::MaestroServer;
pub use virtual_code::{VirtualCodeGenerator, VirtualDocuments};

use tower_lsp::{LspService, Server};

/// Initialize file-based logging to node_modules/.vize/lsp.log
fn init_file_logging() {
    use std::fs::{create_dir_all, OpenOptions};
    use std::sync::Once;
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Try to find node_modules/.vize directory
        let log_dir = std::env::current_dir()
            .ok()
            .map(|p| p.join("node_modules/.vize"))
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .map(|p| p.join("__agent_only").join("vize"))
            })
            .unwrap_or_else(|| std::path::PathBuf::from(".").join("__agent_only/vize"));

        let _ = create_dir_all(&log_dir);

        let log_path = log_dir.join("lsp.log");

        // Try to open log file, fall back to stderr
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&log_path) {
            tracing_subscriber::fmt()
                .with_writer(file.and(std::io::stderr))
                .with_ansi(false)
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .init();
        }
    });
}

/// Start the LSP server using stdio transport.
///
/// This is the main entry point for the language server.
/// It creates a tower-lsp service and starts serving on stdin/stdout.
pub async fn serve() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing for logging to file
    init_file_logging();

    tracing::info!("Starting vize_maestro LSP server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(MaestroServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// Start the LSP server on a TCP socket.
///
/// This is useful for debugging and testing.
pub async fn serve_tcp(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::TcpListener;

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting vize_maestro LSP server on port {}", port);

    #[allow(clippy::disallowed_macros)]
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on 127.0.0.1:{}", port);

    let (stream, addr) = listener.accept().await?;
    tracing::info!("Accepted connection from {}", addr);

    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(MaestroServer::new);

    Server::new(read, write, socket).serve(service).await;

    Ok(())
}
