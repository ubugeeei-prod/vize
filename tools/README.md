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
- `tools/support/github` and `tools/support/tasks`: strictly typed TypeScript commands for
  GitHub Actions job steps and root Vite+ tasks, run directly with `node`.
- `tools/moon`: MoonBit command packages that are still built with `moon run`.

Run `rust-script tools/commands/ci/verify-tool-layout.rs` after adding or removing tool
entrypoints. It verifies that the canonical command surface is Rust Script,
that compatibility modules are no longer executable legacy commands, and that
the legacy command runner does not return.

## Typecheck comparison evidence

`tools/commands/fixtures/typecheck-divergence-report.rs` writes
`<project>-typecheck-observations.json` after the reference diagnostic and coverage runs,
before evaluating mutation controls or documented differences. It retains the complete
native report, reference stdout/stderr, source and generated configs, and source/provider
identity. The comparison's hashes bind those original streams.

A stale documented-difference ledger still fails in both enforce and record-only modes.
Its `<project>-typecheck-divergence-failure.json` contains the rejected comparison,
observations, and mutation controls under a distinct failure schema; no normal comparison
artifact is published for that run. Observation and failure packets are diagnostic evidence
and do not establish project acceptance.

Reusing a report directory archives only this collector's previous comparison JSON/Markdown,
observation packet, and failure packet under `previous-typecheck-comparisons/<project>/`.
Each attempt has a manifest with original names, content hashes, and source provenance.
The live normal comparison paths therefore describe only the current accepted attempt;
previous evidence and other projects' reports remain available.
