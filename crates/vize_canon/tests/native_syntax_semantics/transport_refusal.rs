//! Instrumented denial delegates ordinary CLI calls to the real native runtime.

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub(super) fn without_transport(root: &Path, native: &Path) -> PathBuf {
    // Match the actual runtime's async JSON-RPC authority. An arbitrary shell
    // filename would instead select the legacy msgpack transport.
    fs::create_dir_all(root.join(".bin")).unwrap();
    let wrapper = root.join(".bin/native-without-diagnostic-transport");
    let attempts = root.join(".bin/denied-transports.log");
    fs::write(&attempts, "").unwrap();
    let quoted = native.to_string_lossy().replace('\'', "'\"'\"'");
    let quoted_attempts = attempts.to_string_lossy().replace('\'', "'\"'\"'");
    // One short append records each actual denied transport invocation. CLI
    // calls delegate unchanged to the same real native executable.
    fs::write(&wrapper, format!("#!/bin/sh\ncase \"$1\" in --api|--lsp) printf '%s\\n' \"$1\" >> '{quoted_attempts}'; echo 'native diagnostic transport intentionally unavailable' >&2; exit 41;; esac\nexec '{quoted}' \"$@\"\n")).unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    wrapper
}
