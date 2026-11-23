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
- `tools/commands/davinci`: Davinci roadmap, matrix, corpus, and budget commands.
- `tools/commands/editors`: editor extension packaging and real-server checks.
- `tools/commands/fixtures`: real-project and generated fixture orchestration.
- `tools/commands/release`: release and npm publication commands.
- `tools/support`: shared command support modules grouped by use case.
- `tools/support/github` and `tools/support/tasks`: strictly typed TypeScript commands for
  GitHub Actions job steps and root Vite+ tasks, run directly with `node`.

Run `node tools/commands/ci/verify-tool-layout.ts` after adding or removing tool
entrypoints. It verifies the stable Node command surface, prevents the remaining
Rust Script budget from growing, and keeps retired command runners absent.

## Editing repository workflows and scripts

The Pkl modules in `tools/config/pkl/` own the repository's workflows, composite
actions, shell entrypoints, pnpm workspace, and service configuration. The
[workflow catalog](../.github/workflows/README.md) links to each workflow group.

| To change                    | Edit under `tools/config/pkl/`      |
| ---------------------------- | ----------------------------------- |
| GitHub workflow              | `workflows/<area>/<workflow>.pkl`   |
| Shared action or runner pin  | `Actions.pkl`                       |
| Composite action             | `actions/<name>.pkl`                |
| First-party shell entrypoint | `scripts/<output path>.pkl`         |
| Multi-command package script | `package-tasks/<package>/tasks.pkl` |
| Other YAML configuration     | `repository/<output path>.pkl`      |

After installing workspace dependencies, regenerate the checked-in outputs and verify them:

```sh
node tools/commands/generate/repository-config.ts
node tools/commands/generate/repository-config.ts --check
```

The command uses the workspace's pinned Pkl CLI on Linux and macOS. On Windows,
install the same Pkl version on PATH. Evaluation uses local modules. The required
`check-js` job verifies new files have a source, Actions values and step order
match, other configurations retain key order, and shell text and executable modes
match on Unix. The renderer keeps existing YAML comments and layout where possible.
pnpm owns lockfiles. Upstream reproductions and historical fixture inputs keep
their original bytes.

Group workflows by purpose in the catalog. Dedicated benchmark workflows use the
display name `Benchmark - <subject>`. Use `ShellScript.pkl` to quote command
arguments and verify source revisions, for example `module.requireCheckout("SOURCE_SHA")`.
When workflows use the same program, put it in `parts/shell/` and amend it from
each entrypoint module. Put runtime logic that needs process I/O in a typed
driver under `tools/` and call it from the recipe.

The generator checks every first-party package script for inline commands.
A multi-command package script invokes its generated `scripts/package-tasks.cjs` entrypoint.
Declare each command's executable and argument list in the package's `tasks.pkl`.
Use `directory` and `environment` for package-local setup. Root Vite+ sequences use
`steps(...)` in `tools/config/vite-plus/tasks/`, and Nix shell hooks read the
generated files under `tools/nix/scripts/`.

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
