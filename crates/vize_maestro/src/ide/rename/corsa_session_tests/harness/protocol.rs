use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use vize_s0::cstr;

mod executable;
mod proxy;
mod readiness;

const SHUTDOWN_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(30);

pub(in crate::ide) struct ShutdownGate {
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    sentinel: PathBuf,
}

impl ShutdownGate {
    pub(in crate::ide::rename::corsa_session_tests) fn arm(mut self) -> Result<Self, String> {
        fs::write(&self.sentinel, b"armed").map_err(|error| {
            cstr!(
                "arm editor shutdown gate {}: {error}",
                self.sentinel.display()
            )
        })?;
        self.stream = None;
        Ok(self)
    }

    pub(in crate::ide::rename::corsa_session_tests) fn wait_until_observed(
        &mut self,
    ) -> Result<(), String> {
        let listener = self
            .listener
            .take()
            .ok_or_else(|| "editor shutdown gate was already observed".to_owned())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let acceptor = std::thread::spawn(move || {
            let accepted = listener.accept().map(|(stream, _)| stream);
            let _ = sender.send(accepted);
        });
        let mut stream = match receiver.recv_timeout(SHUTDOWN_OBSERVATION_TIMEOUT) {
            Ok(Ok(stream)) => stream,
            Ok(Err(error)) => {
                let _ = acceptor.join();
                return Err(cstr!("accept editor shutdown observation: {error}").into());
            }
            Err(error) => {
                // Wake the owned accept thread so the test reports the
                // missing protocol phase instead of leaking a blocked helper.
                let _ = TcpStream::connect(address);
                let _ = acceptor.join();
                return Err(cstr!(
                    "editor shutdown was not observed within the bridge deadline: {error}"
                )
                .into());
            }
        };
        acceptor
            .join()
            .map_err(|_| "editor shutdown observer thread panicked".to_owned())?;
        stream
            .set_read_timeout(Some(SHUTDOWN_OBSERVATION_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let mut marker = [0_u8; 1];
        stream
            .read_exact(&mut marker)
            .map_err(|error| cstr!("read editor shutdown observation: {error}"))?;
        if marker != *b"S" {
            return Err(cstr!("invalid editor shutdown marker: {marker:?}").into());
        }
        self.stream = Some(stream);
        Ok(())
    }

    pub(in crate::ide::rename::corsa_session_tests) fn release(&mut self) -> Result<(), String> {
        let Some(mut stream) = self.stream.take() else {
            return Err("editor shutdown gate was not observed".to_owned());
        };
        stream
            .write_all(b"R")
            .map_err(|error| cstr!("release editor shutdown gate: {error}").into())
    }
}

impl Drop for ShutdownGate {
    fn drop(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.write_all(b"R");
        }
    }
}

pub(in crate::ide) fn traced_corsa_executable(
    root: &Path,
    corsa_path: &Path,
    observe_shutdown: bool,
) -> Result<(PathBuf, PathBuf, Option<ShutdownGate>), String> {
    let trace_dir = root.join("protocol-traces");
    fs::create_dir(&trace_dir).map_err(|error| error.to_string())?;
    let actual = corsa_path.canonicalize().map_err(|error| {
        cstr!(
            "canonicalize Corsa executable {}: {error}",
            corsa_path.display()
        )
    })?;
    let actual = actual
        .to_str()
        .filter(|path| !path.contains('\n'))
        .ok_or_else(|| {
            cstr!(
                "Corsa executable path is not shell-safe: {}",
                actual.display()
            )
        })?;
    fs::write(root.join("actual-tsgo.path"), actual).map_err(|error| error.to_string())?;
    let wrapper = root.join("traced-tsgo");
    let shutdown_gate = if observe_shutdown {
        assert_shutdown_gate_runtime()?;
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
        let gate_port = listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port();
        let gate_sentinel = trace_dir.join("shutdown-gate.enabled");
        fs::write(trace_dir.join("shutdown-gate.port"), gate_port.to_string())
            .map_err(|error| error.to_string())?;
        Some(ShutdownGate {
            listener: Some(listener),
            stream: None,
            sentinel: gate_sentinel,
        })
    } else {
        None
    };
    fs::write(root.join("trace-stdio.pl"), proxy::TRACE_STDIO_PROXY)
        .map_err(|error| error.to_string())?;
    executable::link_session_wrapper(&wrapper)?;
    Ok((wrapper, trace_dir, shutdown_gate))
}

/// The shutdown-gated wrapper proxies the editor stream through Perl, so a
/// missing interpreter or `IO::Socket::INET` would otherwise surface as an
/// opaque bridge spawn failure.
fn assert_shutdown_gate_runtime() -> Result<(), String> {
    let probe = std::process::Command::new("perl")
        .args([
            "-MIO::Socket::INET",
            "-MIO::Select",
            "-MIPC::Open3",
            "-e",
            "1",
        ])
        .output()
        .map_err(|error| {
            cstr!("editor shutdown gate requires perl with IO::Socket::INET: {error}")
        })?;
    if !probe.status.success() {
        return Err(cstr!(
            "editor shutdown gate requires perl with IO::Socket::INET: {}",
            String::from_utf8_lossy(&probe.stderr).trim()
        )
        .into());
    }
    Ok(())
}

pub(super) fn assert_graceful_lsp_lifecycle(
    trace_dir: &Path,
    canonical_root_uri: &str,
    logical_root_uri: &str,
    expected_readiness_generations: usize,
) -> Result<(), String> {
    let mut lsp_traces = Vec::new();
    for entry in fs::read_dir(trace_dir).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("client-") && name.ends_with(".raw"))
        {
            continue;
        }
        let trace = fs::read(&path).map_err(|error| error.to_string())?;
        if find_bytes(&trace, b"textDocument/rename").is_some() {
            lsp_traces.push((path, trace));
        }
    }
    if lsp_traces.len() != 1 {
        return Err(cstr!(
            "expected one editor LSP trace containing rename, found {} in {}",
            lsp_traces.len(),
            trace_dir.display()
        )
        .into());
    }
    let (path, trace) = &lsp_traces[0];
    if find_bytes(trace, canonical_root_uri.as_bytes()).is_none() {
        return Err(cstr!(
            "editor LSP trace {} did not use canonical root URI {canonical_root_uri}",
            path.display()
        )
        .into());
    }
    if logical_root_uri != canonical_root_uri
        && find_bytes(trace, logical_root_uri.as_bytes()).is_some()
    {
        return Err(cstr!(
            "editor LSP trace {} mixed logical root URI {logical_root_uri} with canonical {canonical_root_uri}",
            path.display()
        ).into());
    }
    let did_open = find_bytes(trace, b"textDocument/didOpen")
        .ok_or_else(|| cstr!("missing didOpen in {}", path.display()))?;
    let shutdown = find_bytes(trace, b"\"shutdown\"").ok_or_else(|| {
        cstr!(
            "raw-closed editor LSP without shutdown in {}",
            path.display()
        )
    })?;
    let exit = find_bytes(trace, b"\"exit\"")
        .ok_or_else(|| cstr!("closed editor LSP without exit in {}", path.display()))?;
    readiness::assert_generation_order(
        path,
        trace,
        did_open,
        shutdown,
        exit,
        expected_readiness_generations,
    )?;
    let pid = path
        .file_stem()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("client-"))
        .ok_or_else(|| cstr!("unexpected protocol trace name: {}", path.display()))?;
    assert_clean_stderr(trace_dir, pid)?;
    assert_reaped(trace_dir, pid)
}

fn assert_clean_stderr(trace_dir: &Path, pid: &str) -> Result<(), String> {
    let path = trace_dir.join(cstr!("server-{pid}.stderr"));
    let stderr = fs::read_to_string(&path)
        .map_err(|error| cstr!("read editor LSP stderr {}: {error}", path.display()))?;
    for forbidden in [
        "RequestCancelled",
        "error handling method 'textDocument/didOpen': context canceled",
        "error handling method 'textDocument/rename': context canceled",
    ] {
        if stderr.contains(forbidden) {
            return Err(cstr!(
                "editor LSP emitted {forbidden:?} in {}: {stderr}",
                path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn assert_reaped(trace_dir: &Path, pid: &str) -> Result<(), String> {
    let path = trace_dir.join(cstr!("process-{pid}.reaped"));
    let status = fs::read_to_string(&path).map_err(|error| {
        cstr!(
            "editor LSP wrapper or descendant was not reaped before shutdown returned ({}): {error}",
            path.display()
        )
    })?;
    status.trim().parse::<i32>().map_err(|error| {
        cstr!(
            "invalid editor LSP reap marker {} ({status:?}): {error}",
            path.display()
        )
    })?;
    Ok(())
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
