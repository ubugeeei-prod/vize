#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

// tool-host: 5b3636aad1ecb421
#[path = "../../../rust/tool_host.rs"]
mod tool_host;

fn main() -> std::process::ExitCode {
    tool_host::run(
        tool_host::Runtime::Node,
        "tools/editor-e2e/real-vue-workspace.mjs",
    )
}
