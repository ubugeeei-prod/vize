This is step 2 of the #6830 redesign record (`docs/davinci/decisions/2026-11-19-6830-pr-check-redesign.md`, landing in #13396). The maintainer decided to fix tooling selection accuracy. Three rules change in `plan-tooling-tests.mjs` (new `memberReferences`):

1. **Third-party sources** (any import-closure member under `node_modules/`) contribute no path references and are not read. Before, a test whose closure reached `npm/cli/node_modules/typescript/lib/typescript.js` was selected by every change, because that file contains the string `**/*`.
2. **Task configuration modules** (`tools/config/**`) select only by exact file names. Their globs and directory strings (`tools/**`, `{tools,tests,npm,docs,.github}/**/*.ts`, `tools/support/`) describe task inputs and commands, not files their importers read.
3. **Widely shared helpers** (more than `SHARED_HELPER_IMPORTERS` importers) no longer select by glob strings. They already did not select by top-level names.

The release contracts audit covers the `tests/tooling/release/**` tests that sat on the any-input `tooling` scope, which was pending its owners' audit:

- 5 tests that execute the source CLI or native addon keep `tooling`: `computed-event-release`, `published-vite-startup`, `release-editor-host-workflow`, `release-smoke-musl` and `rolling-quota-ownership`.
- The rest use the new `toolingReleaseSurfaces` input:
  - release scripts, commands, tests and fixtures;
  - `docs/release/**` and `.github/**`;
  - the root, crate, npm and editor manifests and the lockfiles.

  Their import closures and literally named paths still select them as before.

**Where the deselected tests still run:** the merge queue uses the same PR-tier planner on the merge-group diff. Nightly Full qualification (`full-tooling-plan.yml`, `--tier merge`) runs all 1,923 tooling files. A new test pins that the merge tier still returns every file.

## Measurement (selection replayed on the last 25 `main` commits)

Same planner, base `754ee0e8a5` against head; per-commit rows are in the commit-by-commit replay:

| | before | after |
|---|---|---|
| tooling tests selected (sum of 25) | 10,349 | **9,462** (−8.6%) |
| estimated test seconds (sum) | 29,987 | **27,574** (−8.0%) |
| shards (sum) | 263 | **243** (−20 jobs) |
| Rust-only fix-history PR `dc783944a3` | 244 tests / 9 shards | 172 / 6 |
| `1925dbcdb9` (perf(canon), 8 paths) | 261 / 9 | 189 / 6 |
| docs-only `a74306cee7` (1 path) | 170 / 4 | 132 / 3 |

The 25-commit replay removed 887 test selections and no test was added, apart from this PR's own new test.

## Tests

- New `tests/tooling/tooling-reference-accuracy.test.ts` checks:
  - each of the three reference rules;
  - the audited release scopes: crate manifests stay inputs, Rust sources and general docs do not, and the product-executing release tests keep `tooling`;
  - that the merge tier still selects every file.
- I ran locally the 60 tooling test files that import the planner, `task-inputs` or `tooling-test-scopes`: 289 pass, 13 fail.
  - All 13 are `davinci-plugin-*` sandbox/SDK cases and two fixture files. They need the built native addon or plugin sandbox, and none of them imports the changed modules; they only use a plugin field named `cacheInputs`.

## After

The exact-head Check is pending; I will add its tooling shard count and wall time.

## Product binaries

Unchanged. Only CI selection scripts, task-input config and a tooling test change. No file under `crates/`, `davinci/` or `npm/` changes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
