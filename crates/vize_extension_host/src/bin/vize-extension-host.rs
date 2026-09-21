//! `vize-extension-host serve <component.wasm>`: the child process of the
//! out-of-process hosting mode. Loads the component under wasmtime, reports
//! `ready` (or the exact load error) on stdout, then answers wire requests
//! until stdin closes.

use std::io::{self, BufReader};
use std::path::Path;
use std::process::ExitCode;

use vize_extension_host::GuestError;
use vize_extension_host::wasm::WasmGuest;
use vize_extension_host::wire::{Response, serve, write_message};

const USAGE: &str = "usage: vize-extension-host serve <component.wasm>";

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [command, component] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    if command != "serve" {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut guest = match WasmGuest::load(Path::new(component)) {
        Ok(guest) => guest,
        Err(error) => {
            let message = match error {
                GuestError::Instantiate(message)
                | GuestError::Trap(message)
                | GuestError::Transport(message) => message,
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
