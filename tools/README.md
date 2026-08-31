# Tools

`tools/commands` is the canonical command surface for repository automation.
Every file there is a Rust Script entrypoint and can be run with `rust-script`.

Older JavaScript files that still carry reusable test helpers remain as
compatibility modules, not command entrypoints. CI/package invocations should
call the Rust Script command path, and new user-facing automation belongs in
`tools/commands`.

## Layout

- `tools/commands/agents`: agent-facing local automation.
- `tools/commands/ci`: GitHub Actions, fuzz, and hosted verification helpers.
- `tools/commands/davinci`: Davinci roadmap, matrix, corpus, and budget commands.
- `tools/commands/editors`: editor extension packaging and real-server checks.
- `tools/commands/fixtures`: real-project and generated fixture orchestration.
- `tools/commands/release`: release, npm, and changelog commands.
- `tools/support`: shared command support modules grouped by use case.
- `tools/moon`: MoonBit command packages that are still built with `moon run`.

Run `rust-script tools/commands/ci/verify-tool-layout.rs` after adding or removing tool
entrypoints. It verifies that the canonical command surface is Rust Script,
that compatibility modules are no longer executable legacy commands, and that
the legacy command runner does not return.
