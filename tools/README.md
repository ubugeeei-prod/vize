# Tools

`tools/commands` is the canonical command surface for repository automation.
New commands use stable Node's native TypeScript stripping and run directly with `node`.
The remaining Rust Script entrypoints are migration debt and cannot grow.

Reusable helpers belong in `tools/support`; user-facing automation belongs in
`tools/commands`. Keep command files thin and put testable behavior in focused
support modules.

## Layout

- `tools/commands/agents`: agent-facing local automation.
- `tools/commands/ci`: GitHub Actions, fuzz, and hosted verification helpers.
- `tools/commands/levels`: Levels roadmap, matrix, corpus, and budget commands.
- `tools/commands/editors`: editor extension packaging and real-server checks.
- `tools/commands/fixtures`: real-project and generated fixture orchestration.
- `tools/commands/release`: release and npm publication commands.
- `tools/support`: shared command support modules grouped by use case.
- `tools/support/github` and `tools/support/tasks`: strictly typed TypeScript commands for
  GitHub Actions job steps and root Vite+ tasks, run directly with `node`.

Run `node tools/commands/ci/verify-tool-layout.ts` after adding or removing tool
entrypoints. It verifies the stable Node command surface, prevents the remaining
Rust Script budget from growing, and keeps retired command runners absent.

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
