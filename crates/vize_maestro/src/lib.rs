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
//! vize_maestro::serve_blocking().unwrap();
//! ```

pub mod document;
pub mod ide;
pub mod runtime;
pub mod server;
pub mod utils;
pub mod virtual_code;

/// Legacy Vue (v0.10 / v0.11 / v1 / v2) editor / LSP surface. Gated behind the `legacy`
/// feature and dropped from the default Vue 3 build; opt-in only.
#[cfg(feature = "legacy")]
pub mod legacy;

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
    use std::fs::{OpenOptions, create_dir_all};
    use std::sync::Once;
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let log_dir = std::env::current_dir()
            .ok()
            .map(|p| p.join("node_modules/.vize"))
            .unwrap_or_else(|| std::path::PathBuf::from(".").join("node_modules/.vize"));

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

    let stdin = runtime::threaded_reader("vize-lsp-stdin", std::io::stdin())?;
    let stdout = runtime::threaded_writer("vize-lsp-stdout", std::io::stdout())?;

    let (service, socket) = LspService::new(MaestroServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// Start the stdio LSP server on the current thread.
pub fn serve_blocking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    runtime::block_on(serve())
}

/// Start the LSP server on a TCP socket.
///
/// This is useful for debugging and testing.
pub async fn serve_tcp(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::net::TcpListener;

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting vize_maestro LSP server on port {}", port);

    let addr = vize_carton::cstr!("127.0.0.1:{port}");
    let listener = TcpListener::bind(addr.as_str())?;
    tracing::info!("Listening on 127.0.0.1:{}", port);

    let (stream, addr) = runtime::accept_tcp("vize-lsp-tcp-accept", listener).await?;
    tracing::info!("Accepted connection from {}", addr);

    let _ = stream.set_nodelay(true);
    let read = runtime::threaded_reader("vize-lsp-tcp-read", stream.try_clone()?)?;
    let write = runtime::threaded_writer("vize-lsp-tcp-write", stream)?;

    let (service, socket) = LspService::new(MaestroServer::new);

    Server::new(read, write, socket).serve(service).await;

    Ok(())
}

/// Start the TCP LSP server on the current thread.
pub fn serve_tcp_blocking(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    runtime::block_on(serve_tcp(port))
}
