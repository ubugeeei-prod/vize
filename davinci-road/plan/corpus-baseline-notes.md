# Corpus baseline notes (P0-5)

> [!NOTE]
> Companion to `tests/_fixtures/davinci-baseline.json` — the whole-corpus
> output fingerprint that `tools/davinci/corpus-diff.mjs` (suite TS-11)
> gates every later Davinci phase against. This file records the hash
> contract, the surface-list derivation, and every known source of
> nondeterminism in current tool output, filed here per the P0-5 rule:
> file it, do not fix it in this task.

## Regeneration

```sh
cargo build --release -p vize
node tools/davinci/corpus-baseline.mjs --clean-fixtures   # rewrite the committed baseline
node tools/davinci/corpus-diff.mjs --clean-fixtures       # gate a fresh run against it
```

**Both tools refuse to sweep contaminated fixtures.** A run only produces
comparable hashes when it starts from the pinned tree, so each tool
pre-flights it: every fixture submodule must sit at its recorded sha, and no
`node_modules` may be materialized inside a checkout. A sweep leaves ~142 of
them behind (see Re-record 2 below), so the next run without
`--clean-fixtures` stops immediately with the offending paths instead of
spending minutes producing hashes nobody can trust. `--clean-fixtures`
removes them first; `--allow-dirty-fixtures` sweeps anyway and is only for
deliberate experiments, since its hashes are not comparable to the committed
baseline. Drifted submodules are never auto-repaired — the tool prints the
`git submodule update` line and stops, because silently re-hydrating a
fixture would change what the corpus measures.

Both tools spawn `tools/fixtures/tool-matrix-report.mjs` across all shards
(4 parallel shard processes by default; the harness is serial inside a
shard) and reduce each project's per-surface payload to a
`{surface, project, file_count, content_hash}` row.

## Surface list — derived from the harness, not the plan prose

The phase-0 task text names "compile dom/vapor/ssr" as compile surfaces.
The harness this baseline wraps emits exactly four tool lanes per project,
and those lanes are the baseline's surfaces:

| Surface       | Harness command                                                                                            |
| ------------- | ---------------------------------------------------------------------------------------------------------- |
| `compiler`    | `vize build <globs> --format json --output <tmp> --template-syntax quirks --continue-on-error --no-config` |
| `typechecker` | `vize check <globs> --format json --no-config [--tsconfig <path>]`                                         |
| `linter`      | `vize lint <globs> --format json --preset ecosystem --no-config`                                           |
| `formatter`   | `vize fmt <globs> --check --no-config`                                                                     |

There is a single compile lane — the default (DOM) backend. The harness has
no vapor or ssr lanes today, so the baseline cannot fingerprint them;
per-backend compile surfaces join the baseline when the harness grows those
lanes.

## Hash contract

`content_hash` is the sha256 of a canonical JSON (object keys sorted
recursively) of the following fields of the harness run payload
(`<project>-<tool>.json`, schema `vize.fixtureToolRun`):

| Surface       | Hashed fields                                         |
| ------------- | ----------------------------------------------------- |
| `compiler`    | `compilerArtifacts`, `exitCode`, `stdout`             |
| `typechecker` | `exitCode`, `stderr`, `stdout`, `typecheckerCoverage` |
| `linter`      | `exitCode`, `stderr`, `stdout`                        |
| `formatter`   | `exitCode`, `formatterCheck`, `stdout`                |

- `stdout` carries the actual tool output for `typechecker` (check JSON)
  and `linter` (lint JSON). The payload's `parsed` field is its
  `JSON.parse` and is deliberately not hashed twice.
- The compiled artifacts themselves are covered byte-for-byte by
  `compilerArtifacts.sha256` — the path + content digest the harness
  computes over every emitted compile artifact
  (`tools/fixtures/tool-matrix-run.mjs`, `inspectCompilerArtifacts`),
  taken before the temporary output directory is deleted.
- `file_count` mirrors `tools/fixtures/tool-matrix-metrics.mjs`:
  compiler `inputFileCount`, typechecker `fileCount` (requested +
  transitive authored), linter file-entry count, formatter
  `checkedFileCount`.

## Filed nondeterminism: compiler and formatter `stderr` are excluded

Two payload fields a successful run produces are excluded from the
fingerprint because they are machine- and run-varying by construction.
Both were verified empirically before the baseline landed: two
back-to-back single-project matrix runs on the same tree differ in
exactly these two fields and nothing else. Filed here per the P0-5
rule — do not fix the tools in this task; a later change can make these
streams deterministic and fold them into the hash with a schema-version
bump.

**Compiler `stderr`** (`crates/vize/src/commands/build/` at the P0-5
baseline commit) varies four ways:

1. **Absolute temporary output paths** — every compiled file logs
   `Built: <input> -> <output>` (`runner/output.rs:199-204`), and the
   harness points `--output` at a fresh `mkdtemp` directory
   (`vize-fixture-compiler-XXXXXX`), so every line embeds a random
   absolute path that changes each run.
2. **Wall-clock banner** — every run ends with
   `✓ N files compiled in {:.4}s` (`runner.rs:412-419`) or
   `✗ N file(s) failed, M compiled in {:.4}s` (`runner.rs:400-410`).
3. **Load-dependent slow-file warnings** — any file whose compile
   crosses `--slow-threshold` (default 100 ms) adds a
   `⚠ N slow file(s) detected` block with per-file millisecond timings
   (`runner.rs:241-272`); whether a file crosses 100 ms under a parallel
   sweep depends on machine load.
4. **Error-listing order** — with `--continue-on-error`, per-file errors
   are pushed into a `Mutex<Vec<_>>` from rayon workers
   (`runner/fallback.rs`, `record_error`) and printed in push order,
   which is thread-completion order, not input order.

None of this is compile _output_ — the artifacts, diagnostics, and exit
code are all hashed via `compilerArtifacts`.

**Formatter `stderr`** (`crates/vize/src/commands/fmt.rs`): the
`Would reformat: <path>` lines are printed directly from rayon worker
threads (`eprintln!` at line 415 inside the `files.par_iter()` loop at
line 144), so the same set of paths arrives in a different order on
every run. The deterministic form of the same evidence is hashed
instead: `formatterCheck.changedPathsSha256` digests the sorted path
set, plus the checked/changed/unchanged counts.

`stderr` stays hashed on the remaining two surfaces: lint and check
emit nothing there on a clean corpus run, and any future stderr chatter
on those lanes should surface as drift, not be masked.

## Reproducibility verdict (two full sweeps, same tree)

Recorded 2026-08-14 on the baseline machine (macOS arm64, Apple M2 Max,
12 logical CPUs; release binary built from the P0-5 branch):

- Sweep 1: `node tools/davinci/corpus-baseline.mjs` — wrote the committed
  artifact (251 s wall for 134 projects x 4 surfaces, 4 shard processes).
- Sweep 2: `node tools/davinci/corpus-diff.mjs --write-fresh <tmp>` —
  the two artifact files are **not byte-identical**: they differ in
  exactly one of 536 rows, `typechecker/element-plus`
  (`0ed6b624d6ee…` vs `9ec7a085ab15…`); the other 535 rows and the whole
  scope block are byte-equal.

The divergence, pinned from the kept raw payloads of both sweeps: the
check JSON differs in a single file entry,
`packages/components/slot/index.ts`, which in sweep 1 carries one extra
diagnostic — `TS6307 File '…/slot/src/only-child.tsx' is not listed
within the file list of project '…/tsconfig.shard0.json'` — and in
sweep 2 carries none (corpus totals 3704 vs 3703 errors).
`tsconfig.shard0.json` is not a file on disk; it is the virtual shard
project `vize check` builds internally (corsa), so whether
`only-child.tsx` lands in the shard's file list before its importer is
checked races between runs. A third full sweep produced yet another
distinct hash for the same row (`cfc6a995efad…`), so the race has more
than two outcomes; every other row stayed byte-equal in all three
sweeps. **Filed, not fixed** (P0-5 is instrumentation-only): the row is
shard-scoped in `corpus-baseline-unstable.json`, which `corpus-diff`
reports as "unstable (filed, not gating)" while every other row still
gates. The committed baseline carries sweep 1's value. Fixing the race
in `vize check` and deleting the sidecar entry is the follow-up.

Two machine-boundedness facts, also filed (run-stable on one machine,
so they do not affect this verdict, but they anchor the artifact to the
machine that produced it):

- 132 of 134 typechecker payloads carry a stderr progress banner
  (`Building Corsa virtual project for N files under <absolute path>…`)
  that embeds the absolute fixture path.
- 10 projects embed absolute paths inside check diagnostics themselves
  (stdout): `directus`, `element-plus`, `element-plus-x`,
  `lx-music-desktop`, `mealie`, `scalar`, `vant`, `vue-cropper`,
  `vue-draggable-next`, `vue-router`.

A baseline regenerated on a different machine (or a different worktree
path) will therefore drift on typechecker rows even with identical tool
behavior; regenerate on the reference runner (or an identical path) when
refreshing the artifact.

## Corpus repair required before the first baseline

The first full sweep failed on exactly one project: `vue-storefront`,
all four lanes (`compiler matched no Vue files`, and the other three
validators rejecting a zero-file run for a fixture that does not declare
`expectedVueFileCount: 0`). The unattended submodule bump #4236
(2026-08-12) had followed upstream to revision `16167a4c`, whose commit
message is "chore: remove all code, keep README only" — the upstream
repository was emptied, and the bump also downgraded the manifest
license to `NONE`. P0-5 restored the previous pin `220341f4` (48 `.vue`
files, MIT license) in both the gitlink and the manifest entry; the
baseline covers the restored pin. Follow-up for the bump lane: a pin
that zeroes out a project's `vueGlobs` matches should fail the bump PR,
not the next corpus consumer.

## Scope proof (TS-11)

An "empty" diff is only meaningful if the run actually covered the
corpus, so the artifact embeds its scope and `corpus-diff` re-proves it
on both sides before reporting success:

- the baseline artifact exists and parses;
- baseline and fresh runs each cover every project in
  `tests/_fixtures/vue-ecosystem-fixtures.json` on every gated surface
  (missing or unknown projects are itemized);
- row count equals projects x surfaces;
- the total file count is nonzero (a zero-file run fails), and a
  zero-file row is only legal for projects whose manifest entry declares
  `expectedVueFileCount: 0` (`docsify`, `vue-native-core`);
- `--surface` narrows which lanes are gated for the fresh run, but the
  committed baseline is always validated against the full manifest scope;
- rows listed in `corpus-baseline-unstable.json` (filed nondeterminism)
  are still compared and reported, but their hash drift does not gate;
  missing rows always gate, and the sidecar rejects unknown surfaces,
  unknown projects, and entries without a reason.

The baseline covers the 134-project manifest as of the P0-5 sweep. Corpus
expansion round 1 (#4324) has since added 8 projects to the manifest
(`dho-web-client`, `petite-vue`, `vue-core-vapor`, `vue-jsx-vapor`,
`vue3-admin-design`, `vue3-antd-admin`, `wakapi`, `wave-ui`), so against
the current 142-project manifest `corpus-diff` fails its scope proof up
front — `manifest_project_count 134 != manifest 142` plus the eight
missing rows per surface — and will keep failing until the artifact is
regenerated on the reference runner. That re-baseline is P0-6 work per the
phase plan (P0-6 is still open): run `node tools/davinci/corpus-baseline.mjs`
once the new fixtures are checked out, and the row count moves to
142 x 4 = 568. The failure is deliberate: a stale-scope baseline reports
loudly instead of gating a subset of the corpus and calling it green.
_(Resolved: the phase-0 exit gate (#4331) executed exactly that re-record —
568 rows at the phase-final head, corpus-diff verified twice.)_

## Re-record 2 — post-main-sync materialized tsgo sessions (2026-08-14)

The first main → davinci sync after the phase-0 exit gate (#4333) imported
`2b56b13ed` _fix(canon): share tsgo editor project state_, which
materializes project session state — including a `node_modules` directory —
**inside the checked project's working tree** during batch `vize check`.
Against the exit-gate baseline this drifted two surfaces deterministically:
typechecker 137/142 rows (resolution now sees the materialized state) and
compiler 15/142 rows (`vize build` project sniffing flips on `node_modules`
presence — exact 15/15 correlation, likely unintended coupling; filed as
[#4340](https://github.com/ubugeeei-prod/vize/issues/4340) with the
maintainer questions).

Evidence recorded before re-recording (TS-11: investigate, never average):

- drift is deterministic — two independent clean-fixture sweeps reproduce
  identical drift sets and identical fresh hashes;
- drift is binary-bracketed — the committed baseline _is_ the pre-sync run
  (verified twice at the exit gate), and the post-sync head drifts
  identically with and without the P1-1 allocator change (hash-identical
  on all 568 rows), so phase-1 work contributes zero drift;
- one sweep from clean fixtures leaves 142 fixture projects with a
  materialized `node_modules` (dirty submodule worktrees) — corpus runs
  must start from clean fixtures (`git submodule status` all-clean, no
  `node_modules`) for reproducible hashes; a runner-side contamination
  guard is filed as follow-up work.

This section's baseline (568 rows at `d96852a18` + this branch) blesses
the post-sync behavior so the TS-11 gate stays meaningful for phase-1 PRs.
If #4340 changes the materialization behavior, that PR re-records again.
