# Tools

`tools/commands` is the canonical command surface for repository automation.
Every file there is a Rust Script entrypoint and can be run with `rust-script`.

The older JavaScript and shell files remain as compatibility implementation
modules while the command surface migrates. They are still imported by tooling
tests, but new CI/package invocations should call the Rust Script command path.

## Layout

- `tools/commands/agents`: agent-facing local automation.
- `tools/commands/ci`: GitHub Actions, fuzz, and hosted verification helpers.
- `tools/commands/davinci`: Davinci roadmap, matrix, corpus, and budget commands.
- `tools/commands/editors`: editor extension packaging and real-server checks.
- `tools/commands/fixtures`: real-project and generated fixture orchestration.
- `tools/commands/release`: release, npm, and changelog commands.
- `tools/rust`: shared Rust Script support and layout verification.
- `tools/moon`: MoonBit command packages that are still built with `moon run`.

Run `rust-script tools/rust/verify-layout.rs` after adding or removing tool
entrypoints. It verifies that every legacy command entrypoint has a matching
Rust Script wrapper and that wrappers point at the expected runtime.
