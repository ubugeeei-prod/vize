# Strictly typed TypeScript tooling, batch by batch

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) record sets
how repository tooling moves from `.mjs` to `.ts` that Node runs directly
through native type stripping. It extends the
[owned TypeScript migration](./2026-10-08-owned-typescript-migration.md) with
the batch method and typing rules every directory follows. The pilot is
`tools/benchmarks/scripts`: 129 `.mjs` files and 25,657 lines.

## Why batches follow the import graph

`tsc` type-checks every file that a checked file imports. A typed file that
imports an untyped `.mjs` module either fails strict checking or quietly
receives `any`. A batch may therefore contain only files whose `.mjs`
imports are already typed `.ts` or move in the same batch. Each batch stays
at roughly 3,000-5,000 lines so it can be reviewed.

The graph covers more than `import` statements. Benchmark scripts also name
each other in places that only behave like edges at runtime:

- `new URL("./x.mjs", import.meta.url)` and paths passed to `node`;
- provenance lists that hash scripts by filename;
- workflow `paths:` filters and the release-input pins that mirror them;
- `FULL_SWEEP_PATHS`-style sets that decide which suites run.

The planning script resolves static and dynamic imports and string
specifiers. `rg --hidden` then finds every other reference, including
workflows, actions, Vite+ tasks, package scripts, tests and docs. Tests also
assert on regex-escaped forms such as `enforce-pr-budget\.mjs`, so the
search covers those too.

## Batch method

1. **Move.** A move-only commit runs `git mv x.mjs x.ts` from an explicit path
   map, recorded in the commit message. Every rename is 100% similar, so
   in-flight fixes stay rebasable. After a conflict, rerun the map on `main`.
2. **Type.** A second commit adds types and rewrites `.mjs` specifiers in every
   importer anywhere in the repository. It also updates every live caller and
   reference: workflow and action commands and path filters, Vite+ tasks,
   package scripts, tests and living documentation.
3. **Check.** The same commit lists each migrated file explicitly in
   `tsconfig.benchmark-scripts.json`. That project extends the strict
   `tsconfig.node.json` and runs in `check:repo` beside the other checked
   projects. It has no include glob and no exclusions. Files that are not
   typed yet are simply absent.
4. **Verify.** The strict project, `vp fmt`, `vp lint`, every test that
   imports or names a moved file, and the source-length budget must all pass.
   Run `vp lint` on directories: given a single file it printed no warnings
   in the pilot, while the directory run reported five.

## Typing rules

- Use erasable syntax only. Type-only imports use `import type`, and every
  relative specifier names the `.ts` file.
- Do not use `any`, whether written or leaked. `JSON.parse`, `createRequire()`
  `require` and dynamic `import()` results are assigned to `unknown`.
  `Array.isArray` must not be used to turn `unknown` into `any[]`. Do not use
  `@ts-ignore`, `@ts-expect-error`, `@ts-nocheck`, double casts, or casts of
  parsed data.
- Narrow parsed JSON with guards. `script-types.ts` holds the shared
  `CliArgs` type and the `isRecord`, `isStringArray` and `isOptional` guards.
  A guard
  either restates a check the script already made or rejects input that the
  original would have crashed on or rendered meaninglessly. Every accepted
  input keeps identical output. Guards model what JSON actually stores:
  `NaN` and `Infinity` round-trip as `null`.
- Do not refactor. Duplicated helpers, such as each script's `parseArgs`,
  stay duplicated. Add a runtime expression only when typing requires it and
  it is provably equivalent, such as `value === null ||` before
  `Number.isFinite(value)`.
- Use a non-null assertion only where the original already depended on the
  invariant and would have thrown a `TypeError` without it. For example, a
  replay tool's binary is non-null because the tool was selected.
- The project uses `strict`. `noUncheckedIndexedAccess` and
  `exactOptionalPropertyTypes` stay off, because enabling them would force
  logic edits. They can ratchet on once the directory is typed.

## What changes and what stays

Live references follow the file: commands, `paths:` filters, test imports
and assertions, usage strings, code comments, `docs/content` in every
locale, `docs/release`, plan pages and crate inventories.

Recorded history keeps its bytes: dated decision records, and evidence and
fixtures (`tests/_fixtures`, `tools/benchmarks/results`, decision evidence
and compatibility JSON).

Published benchmark text is output, not a reference. `benchmark-notes` and
`benchmark-provenance` render sentences that name `nuxt-bridge-transform`,
`musea-corpus`, `musea`, `check-gate` and `benchmark-provenance` as
`.mjs`, and `publish-snapshot` stamps its own name into every locale page.
`publish-snapshot --check` re-renders `README.md` and the performance pages
from the stored results, and the Japanese measurement-custody fixture hashes
the rendered backend sentence. The typing commit for those batches therefore
keeps the rendered strings byte-identical. Renaming them is a separate commit
that regenerates the pages with `publish-snapshot` and re-pins the custody
fixture.

`typechecker-native-phases.mjs` pins the SHA-256 of
`type-snapshot-cli-{corpus,leaf-corpus,protocol}.mjs` and hashes its helpers
by name, as does the `type-snapshot-cli-runner` provenance list.
`typechecker-native-source-custody.mjs` names the same scripts in its bridge
and infrastructure path sets. A batch that changes any byte of a pinned file
re-pins it in the same commit, even when the only change is an import
specifier, and every batch renames its files in those lists. Batch 3 therefore
moves the gate foundations together with `type-snapshot-cli-*`, so the
copied corpus is re-pinned once rather than twice.

## Runtime before setup

Some jobs run a script with the runner image's default `node`, before any
pinned setup step. Type stripping needs Node 22.18 or later, and the image
default is unspecified. Such entrypoints move only together with a pinned
Node step. The pilot adds SHA-pinned `actions/setup-node` to
`pr-benchmark-budget`, following the
[dependency gate precedent](./2026-10-13-level-dependency-node-setup.md).

Three entrypoints remain in batch 5:

- `comment-pr.mjs`, which runs from the trusted base in both comment jobs and
  also needs an old/new path transition;
- `comment-test-report.mjs` in `check.yml`;
- the `sfc-parse-replay-contract.mjs` import, which runs before `setup-vp`.

## Source-length budget

`tools/commands/ci/source-file-lengths.rs` diffs the base against the work
tree with `git diff --find-renames`. It then reads a renamed file's old path
in the base. A rename is therefore compared with its original, not treated
as new, but only while Git still detects the rename at 50% similarity
across the whole pull request.

Files at or under 350 lines never fail. A typed file must stay at 350 lines
or fewer. A file already over 350 lines (`compare-tools` at 1,198 lines,
`generate` at 842 and `comment-test-report` at 515) must stay recognisable
as a rename and must not grow. Type declarations that do not fit move to
sibling modules.

Rename detection also keeps in-flight fixes rebasable after a squash merge,
so a typed file should stay well above 50% similarity even when it is
short. Large narrowing blocks move to a sibling module for this reason too:
the pilot moved the PR artifact guards into `pr-benchmark-artifact.ts`, which
raised `enforce-pr-budget` from an add/delete pair to a 71% rename. Every
pilot rename stays at or above 55%.

## Batch plan

| Batch | Scope                                                                                                                                                                                                 | Files | Lines |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----: | ----: |
| 1     | PR budget, Criterion, dialect guard, production summary, Vapor compare, vue-benchmarks replay, doctor filter, test inventory                                                                          |    18 | 3,267 |
| 2     | `scale/**`, Nuxt and Vite cache helpers, native batch, Nuxt bridge, benchmark notes, type-snapshot pairing                                                                                            |    16 | 2,138 |
| 3     | Gate foundations (`benchmark-binary`, `check-gate-*`, `typecheck-*`, provenance), `generate`, `type-snapshot-cli-*`; the one re-pin of the copied corpus hashes                                       |    14 | 3,383 |
| 4     | Musea lane, `compare-tools` and its helpers                                                                                                                                                           |    13 | 3,274 |
| 5     | `render-results`, published snapshot, pre-setup entrypoints (`comment-pr`, `comment-test-report`, SFC parse replay), `vite-hmr-owner-lookup` after its fix, vue-benchmarks current lint and typecheck |    20 | 3,464 |
| 6     | `canon-alias-*`, native canon-lib, graph archive, profile replay                                                                                                                                      |    16 | 3,031 |
| 7     | Native profile corpus, phase membership, report, capture and runner, retirement, release metadata, source custody                                                                                     |    14 | 2,741 |
| 8     | Native phases driver, dependency link, source recipe and preparation, receipts                                                                                                                        |    10 | 2,539 |
| 9     | `instruction-counts*`, `level-memory*`, after `tools/support/compat/davinci/toml-lite.mjs`                                                                                                            |     8 | 1,820 |

Every batch imports only earlier batches or already typed `.ts` files.
Batch 9 alone depends on a provider outside this directory.

## Unfinished

- Batches 1 and 2 (34 files, 5,405 original lines) are typed and checked.
  Batches 3-9 remain, as do the other directories in the #8840 scope.
- `tests/tooling` callers now import typed modules, but they are not yet in a
  checked project. They join it when their own directory migrates.
- The benchmark scripts that already were `.ts` (`run`, `vite`, `lint`,
  `fmt`, `check`, `instruction-hotspots`, `typecheck-real-world`,
  `css-documentation`) are not yet in a checked project.
- `vite-hmr-owner-lookup.mjs` imports `../npm/builder/vite/...`, a path that has
  not existed since the #5535 consolidation, so it cannot run or type-check.
  It moved from batch 2 to batch 5 and needs a separate fix commit first.
- `scale/corpus.ts` writes its vendor package to `node_modules/@bench/ui` but
  imports it as `@tools/ui`, so a real scale build fails on `main` too. The
  typed harness keeps that behaviour; fixing it is separate work.
- Exact-head Actions, the issue comment that pairs with this record, and
  protected merge all remain required.
