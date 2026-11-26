pub(crate) mod atomic_write;
pub mod build;
pub mod check;
#[cfg(unix)]
pub mod check_server;
pub mod clean;
pub(crate) mod config_stdin;
pub mod content_mapper;
pub mod curator;
pub(crate) mod davinci_ice;
pub mod doctor;
pub mod dump;
pub mod env_info;
pub mod explain;
#[cfg(feature = "glyph")]
pub mod fmt;
#[cfg(feature = "maestro")]
pub mod ide;
pub mod inspector;
pub mod library;
pub mod lint;
#[cfg(feature = "maestro")]
pub mod lsp;
pub mod musea;
mod node_module;
pub mod plugin;
pub mod profile_export;
mod project_nuxt;
mod project_ssr;
#[cfg(feature = "glyph")]
pub mod ready;
pub mod reduce;
pub mod repro;
pub mod upgrade;

/// Stack reserved for each parallel worker. A worker parses and walks one
/// authored file whose nesting the author does not bound; the reservation is
/// address space, and only the pages a file needs are committed.
pub(crate) const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Give the global worker pool [`WORKER_STACK_BYTES`] stacks unless a command
/// has already sized it.
pub(crate) fn start_worker_pool() {
    let _ = rayon::ThreadPoolBuilder::new()
        .stack_size(WORKER_STACK_BYTES)
        .build_global();
}
