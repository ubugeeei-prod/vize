use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

mod executable;

const SHUTDOWN_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(30);

pub(in crate::ide::rename::corsa_session_tests) struct ShutdownGate {
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    sentinel: PathBuf,
}

impl ShutdownGate {
    pub(in crate::ide::rename::corsa_session_tests) fn arm(mut self) -> Result<Self, String> {
        fs::write(&self.sentinel, b"armed").map_err(|error| {
            format!(
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
                return Err(format!("accept editor shutdown observation: {error}"));
            }
            Err(error) => {
                // Wake the owned accept thread so the test reports the
                // missing protocol phase instead of leaking a blocked helper.
                let _ = TcpStream::connect(address);
                let _ = acceptor.join();
                return Err(format!(
                    "editor shutdown was not observed within the bridge deadline: {error}"
                ));
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
            .map_err(|error| format!("read editor shutdown observation: {error}"))?;
        if marker != *b"S" {
            return Err(format!("invalid editor shutdown marker: {marker:?}"));
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
            .map_err(|error| format!("release editor shutdown gate: {error}"))
    }
}

impl Drop for ShutdownGate {
    fn drop(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.write_all(b"R");
        }
    }
}

pub(super) fn traced_corsa_executable(
    root: &Path,
    corsa_path: &Path,
    observe_shutdown: bool,
) -> Result<(PathBuf, PathBuf, Option<ShutdownGate>), String> {
    let trace_dir = root.join("protocol-traces");
    fs::create_dir(&trace_dir).map_err(|error| error.to_string())?;
    let actual = root.join("actual-tsgo");
    std::os::unix::fs::symlink(corsa_path, &actual).map_err(|error| error.to_string())?;
    let wrapper = root.join("traced-tsgo");
    let shutdown_gate = if observe_shutdown {
        assert_shutdown_gate_runtime()?;
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
        let gate_port = listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port();
        let gate_sentinel = trace_dir.join("shutdown-gate.enabled");
        fs::write(root.join("trace-client.pl"), TRACE_CLIENT_PROXY)
            .map_err(|error| error.to_string())?;
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
    executable::link_session_wrapper(&wrapper)?;
    Ok((wrapper, trace_dir, shutdown_gate))
}

/// The shutdown-gated wrapper proxies the editor stream through Perl, so a
/// missing interpreter or `IO::Socket::INET` would otherwise surface as an
/// opaque bridge spawn failure.
fn assert_shutdown_gate_runtime() -> Result<(), String> {
    let probe = std::process::Command::new("perl")
        .args(["-MIO::Socket::INET", "-e", "1"])
        .output()
        .map_err(|error| {
            format!("editor shutdown gate requires perl with IO::Socket::INET: {error}")
        })?;
    if !probe.status.success() {
        return Err(format!(
            "editor shutdown gate requires perl with IO::Socket::INET: {}",
            String::from_utf8_lossy(&probe.stderr).trim()
        ));
    }
    Ok(())
}

const TRACE_CLIENT_PROXY: &str = r#"use strict;
use warnings;
use IO::Socket::INET;

my ($trace_path, $gate_sentinel, $gate_port) = @ARGV;
open my $trace, '>:raw', $trace_path or die "open trace: $!";
binmode STDIN;
binmode STDOUT;
my $tail = '';
my $shutdown_seen = 0;

sub write_all {
    my ($handle, $bytes) = @_;
    while (length $bytes) {
        my $written = syswrite $handle, $bytes;
        die "write proxy stream: $!" unless defined $written;
        substr($bytes, 0, $written, '');
    }
}

while (1) {
    my $read = sysread STDIN, my $chunk, 8192;
    die "read client stream: $!" unless defined $read;
    last if $read == 0;
    write_all($trace, $chunk);
    write_all(*STDOUT, $chunk);
    $tail .= $chunk;
    if (!$shutdown_seen && $tail =~ /"method"\s*:\s*"shutdown"/) {
        $shutdown_seen = 1;
        if (-e $gate_sentinel) {
            my $gate = IO::Socket::INET->new(
                PeerAddr => '127.0.0.1',
                PeerPort => $gate_port,
                Proto => 'tcp',
            ) or die "connect shutdown gate: $!";
            write_all($gate, 'S');
            my $read = sysread $gate, my $release, 1;
            die "read shutdown gate release: $!" unless defined $read;
            die "invalid shutdown gate release" unless $read == 1 && $release eq 'R';
        }
    }
    $tail = substr($tail, -128) if length($tail) > 128;
}
"#;

pub(super) fn assert_graceful_lsp_lifecycle(
    trace_dir: &Path,
    canonical_root_uri: &str,
    logical_root_uri: &str,
) -> Result<(), String> {
    let mut lsp_traces = Vec::new();
    for entry in fs::read_dir(trace_dir).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let trace = fs::read(&path).map_err(|error| error.to_string())?;
        if find_bytes(&trace, b"textDocument/rename").is_some() {
            lsp_traces.push((path, trace));
        }
    }
    if lsp_traces.len() != 1 {
        return Err(format!(
            "expected one editor LSP trace containing rename, found {} in {}",
            lsp_traces.len(),
            trace_dir.display()
        ));
    }
    let (path, trace) = &lsp_traces[0];
    if find_bytes(trace, canonical_root_uri.as_bytes()).is_none() {
        return Err(format!(
            "editor LSP trace {} did not use canonical root URI {canonical_root_uri}",
            path.display()
        ));
    }
    if logical_root_uri != canonical_root_uri
        && find_bytes(trace, logical_root_uri.as_bytes()).is_some()
    {
        return Err(format!(
            "editor LSP trace {} mixed logical root URI {logical_root_uri} with canonical {canonical_root_uri}",
            path.display()
        ));
    }
    let rename = find_bytes(trace, b"textDocument/rename")
        .ok_or_else(|| format!("missing rename request in {}", path.display()))?;
    let shutdown = find_bytes(trace, b"\"shutdown\"").ok_or_else(|| {
        format!(
            "raw-closed editor LSP without shutdown in {}",
            path.display()
        )
    })?;
    let exit = find_bytes(trace, b"\"exit\"")
        .ok_or_else(|| format!("closed editor LSP without exit in {}", path.display()))?;
    if !(rename < shutdown && shutdown < exit) {
        return Err(format!(
            "invalid editor LSP lifecycle order in {}: rename={rename}, shutdown={shutdown}, exit={exit}",
            path.display()
        ));
    }
    let pid = path
        .file_stem()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("client-"))
        .ok_or_else(|| format!("unexpected protocol trace name: {}", path.display()))?;
    assert_clean_stderr(trace_dir, pid)?;
    assert_reaped(trace_dir, pid)
}

fn assert_clean_stderr(trace_dir: &Path, pid: &str) -> Result<(), String> {
    let path = trace_dir.join(format!("server-{pid}.stderr"));
    let stderr = fs::read_to_string(&path)
        .map_err(|error| format!("read editor LSP stderr {}: {error}", path.display()))?;
    for forbidden in [
        "RequestCancelled",
        "error handling method 'textDocument/didOpen': context canceled",
        "error handling method 'textDocument/rename': context canceled",
    ] {
        if stderr.contains(forbidden) {
            return Err(format!(
                "editor LSP emitted {forbidden:?} in {}: {stderr}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn assert_reaped(trace_dir: &Path, pid: &str) -> Result<(), String> {
    let path = trace_dir.join(format!("process-{pid}.reaped"));
    let status = fs::read_to_string(&path).map_err(|error| {
        format!(
            "editor LSP wrapper or descendant was not reaped before shutdown returned ({}): {error}",
            path.display()
        )
    })?;
    status.trim().parse::<i32>().map_err(|error| {
        format!(
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
