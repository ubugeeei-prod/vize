//! `vize-extension-host serve <component.wasm> [--fuel <n>] [--memory <bytes>]`:
//! the child process of the out-of-process hosting mode. Loads the component
//! under wasmtime with the given limits (the defaults of `GuestLimits`
//! otherwise), reports `ready` (or the exact load error) on stdout, then
//! answers wire requests until stdin closes.

use std::ffi::OsString;
use std::io::{self, BufReader};
use std::path::Path;
use std::process::ExitCode;

use vize_extension_host::wasm::WasmGuest;
use vize_extension_host::wire::{Response, serve, write_message};
use vize_extension_host::{GuestError, GuestLimits};

const USAGE: &str =
    "usage: vize-extension-host serve <component.wasm> [--fuel <n>] [--memory <bytes>]";

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn limits(flags: &[OsString]) -> Option<GuestLimits> {
    let mut limits = GuestLimits::default();
    for pair in flags.chunks(2) {
        let [flag, value] = pair else { return None };
        let value: u64 = value.to_str()?.parse().ok()?;
        match flag.to_str()? {
            "--fuel" => limits.fuel_per_call = value,
            "--memory" => limits.max_memory_bytes = value,
            _ => return None,
        }
    }
    Some(limits)
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let [command, component, flags @ ..] = args.as_slice() else {
        return usage();
    };
    if command != "serve" {
        return usage();
    }
    let Some(limits) = limits(flags) else {
        return usage();
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut guest = match WasmGuest::load_with(Path::new(component), limits) {
        Ok(guest) => guest,
        Err(error) => {
            let message = match error {
                GuestError::Instantiate(message)
                | GuestError::Trap(message)
                | GuestError::Transport(message) => message,
                other => vize_s0::cstr!("{other}"),
            };
            return match write_message(&mut out, &Response::LoadError(message)) {
                Ok(()) => ExitCode::from(1),
                Err(_) => ExitCode::from(3),
            };
        }
    };
    match serve(&mut guest, BufReader::new(io::stdin().lock()), out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("vize-extension-host: {error}");
            ExitCode::from(3)
        }
    }
}
