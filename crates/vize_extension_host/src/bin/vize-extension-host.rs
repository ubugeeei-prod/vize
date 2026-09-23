//! `vize-extension-host serve <component.wasm> [--world <world>] [--fuel <n>]
//! [--memory <bytes>]`: the child process of the out-of-process hosting mode.
//! Loads the component under wasmtime as the given world (`input-dialect`
//! unless `--world` names another) with the given limits (the defaults of
//! `GuestLimits` otherwise), reports `ready` (or the exact load error) on
//! stdout, then answers wire requests until stdin closes.

use std::ffi::OsString;
use std::io::{self, BufReader, StdoutLock};
use std::path::Path;
use std::process::ExitCode;

use vize_extension_host::wasm::{WasmExpressionGuest, WasmGuest, WasmOutputGuest};
use vize_extension_host::wire::{
    Response, answer_expression, answer_input, answer_output, serve, write_message,
};
use vize_extension_host::{GuestError, GuestLimits};

const USAGE: &str = "usage: vize-extension-host serve <component.wasm> [--world <input-dialect|expression-dialect|output-target>] [--fuel <n>] [--memory <bytes>]";

enum World {
    Input,
    Expression,
    Output,
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

/// The world and the limits the flags ask for.
fn options(flags: &[OsString]) -> Option<(World, GuestLimits)> {
    let mut world = World::Input;
    let mut limits = GuestLimits::default();
    for pair in flags.chunks(2) {
        let [flag, value] = pair else { return None };
        match (flag.to_str()?, value.to_str()?) {
            ("--world", "input-dialect") => world = World::Input,
            ("--world", "expression-dialect") => world = World::Expression,
            ("--world", "output-target") => world = World::Output,
            ("--fuel", value) => limits.fuel_per_call = value.parse().ok()?,
            ("--memory", value) => limits.max_memory_bytes = value.parse().ok()?,
            _ => return None,
        }
    }
    Some((world, limits))
}

fn refuse(mut out: StdoutLock<'_>, error: GuestError) -> ExitCode {
    let message = match error {
        GuestError::Instantiate(message)
        | GuestError::Trap(message)
        | GuestError::Transport(message) => message,
        other => vize_s0::cstr!("{other}"),
    };
    match write_message(&mut out, &Response::LoadError(message)) {
        Ok(()) => ExitCode::from(1),
        Err(_) => ExitCode::from(3),
    }
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let [command, component, flags @ ..] = args.as_slice() else {
        return usage();
    };
    let Some((world, limits)) = options(flags).filter(|_| command == "serve") else {
        return usage();
    };
    let path = Path::new(component);
    let out = io::stdout().lock();
    let input = BufReader::new(io::stdin().lock());
    let served = match world {
        World::Expression => match WasmExpressionGuest::load_with(path, limits) {
            Ok(mut guest) => serve(|request| answer_expression(&mut guest, request), input, out),
            Err(error) => return refuse(out, error),
        },
        World::Output => match WasmOutputGuest::load_with(path, limits) {
            Ok(mut guest) => serve(|request| answer_output(&mut guest, request), input, out),
            Err(error) => return refuse(out, error),
        },
        World::Input => match WasmGuest::load_with(path, limits) {
            Ok(mut guest) => serve(|request| answer_input(&mut guest, request), input, out),
            Err(error) => return refuse(out, error),
        },
    };
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("vize-extension-host: {error}");
            ExitCode::from(3)
        }
    }
}
