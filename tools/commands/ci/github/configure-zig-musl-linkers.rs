#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../rust/legacy_command.rs"]
mod legacy_command;

fn main() -> std::process::ExitCode {
    legacy_command::run(
        legacy_command::Runtime::Bash,
        "tools/github/configure-zig-musl-linkers.sh",
    )
}
