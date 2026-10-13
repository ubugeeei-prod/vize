//! Process-spawning helpers for the stdio API transports.
//!
//! The public API exposes higher-level configuration types such as
//! [`crate::ApiSpawnConfig`]. This module is responsible for translating those
//! settings into the exact command-line arguments and transport objects used by
//! the JSON-RPC and msgpack clients.

use super::{
    callbacks::{callback_flag, jsonrpc_handlers},
    driver::ClientDriver,
};
use crate::{
    CorsaError, Result,
    jsonrpc::{JsonRpcConnection, JsonRpcConnectionOptions},
    process::{AsyncChildGuard, CorsaCommand},
};
use corsa_core::fast::{CompactString, SmallVec};
use std::{
    io::{BufReader, BufWriter},
    sync::Arc,
};

pub(super) async fn spawn_jsonrpc_stdio(
    command: &CorsaCommand,
    run_external_code: bool,
    filesystem: Option<Arc<dyn super::ApiFileSystem>>,
    request_timeout: Option<std::time::Duration>,
    shutdown_timeout: std::time::Duration,
    outbound_capacity: usize,
    observer: Option<corsa_core::SharedObserver>,
) -> Result<ClientDriver> {
    // JSON-RPC mode is used for callback-capable, async request/response
    // flows. The worker process is wrapped in `AsyncChildGuard` so shutdown
    // always reaps the child.
    let args = stdio_args(command, run_external_code, filesystem.as_deref(), true);
    let mut child = command.spawn_async(args.iter().map(CompactString::as_str))?;
    let stdin = child.stdin.take().ok_or(CorsaError::Closed("api stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or(CorsaError::Closed("api stdout"))?;
    let handlers = filesystem.map(jsonrpc_handlers).unwrap_or_default();
    let rpc = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(stdout),
        BufWriter::new(stdin),
        handlers,
        JsonRpcConnectionOptions::new()
            .with_request_timeout(request_timeout)
            .with_outbound_capacity(outbound_capacity)
            .with_observer_if_some(observer),
    )?;
    Ok(ClientDriver::JsonRpc {
        rpc,
        process: Some(Arc::new(AsyncChildGuard::new(child))),
        shutdown_timeout,
    })
}

pub(super) fn spawn_msgpack_stdio(
    command: &CorsaCommand,
    run_external_code: bool,
    filesystem: Option<Arc<dyn super::ApiFileSystem>>,
    request_timeout: Option<std::time::Duration>,
    outbound_capacity: usize,
    observer: Option<corsa_core::SharedObserver>,
) -> Result<ClientDriver> {
    // Msgpack mode keeps a dedicated worker thread around the blocking stdio
    // pipes. This avoids async framing overhead on the hot path.
    let args = stdio_args(command, run_external_code, filesystem.as_deref(), false);
    let child = command.spawn_blocking(args.iter().map(CompactString::as_str))?;
    let worker = super::msgpack_worker::MsgpackWorker::spawn(
        child,
        filesystem,
        request_timeout,
        outbound_capacity,
        observer,
    )?;
    Ok(ClientDriver::Msgpack {
        worker: Arc::new(worker),
    })
}

fn stdio_args(
    command: &CorsaCommand,
    run_external_code: bool,
    filesystem: Option<&dyn super::ApiFileSystem>,
    async_mode: bool,
) -> SmallVec<[CompactString; 7]> {
    let mut args = SmallVec::<[CompactString; 7]>::new();
    args.push(CompactString::from("--api"));
    if async_mode {
        args.push(CompactString::from("--async"));
    }
    if run_external_code {
        args.push(CompactString::from("--runExternalCode"));
    }
    // Pass the resolved working directory explicitly so downstream tools and
    // diagnostics see the same root the Rust side expects.
    args.push(CompactString::from("--cwd"));
    args.push(CompactString::from(command.cwd().display().to_string()));
    if let Some(filesystem) = filesystem.and_then(callback_flag) {
        args.push(filesystem);
    }
    args
}

#[cfg(test)]
mod tests {
    use super::stdio_args;
    use crate::process::CorsaCommand;

    #[test]
    fn stdio_args_keep_external_code_disabled_by_default() {
        let command = CorsaCommand::new("/opt/bin/corsa").with_cwd("/workspace");
        let args = stdio_args(&command, false, None, false);
        let args = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();

        assert_eq!(args, ["--api", "--cwd", "/workspace"]);
    }

    #[test]
    fn stdio_args_can_enable_content_mapper_external_code() {
        let command = CorsaCommand::new("/opt/bin/corsa").with_cwd("/workspace");
        let args = stdio_args(&command, true, None, true);
        let args = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();

        assert_eq!(
            args,
            [
                "--api",
                "--async",
                "--runExternalCode",
                "--cwd",
                "/workspace"
            ]
        );
    }
}
