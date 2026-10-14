# Retirement plan for the lint source-custody fixtures

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

About 16 MiB of fixtures exist only to prove where lint sources came from.
They no longer fail when product files change, so they are not merge
friction, but every checkout and fixture hydration still pays for them. They
are not removed yet because the paired performance harness and the star
listing ledgers still reach them.

## What is there

| Directory under `tests/_fixtures/differential/`            |     Size | Files |
| ---------------------------------------------------------- | -------: | ----: |
| `linter/no-mutating-props-options-uploader/source-custody` | 14.4 MiB |    58 |
| `lint/setup-export-default-8090/source-custody`            |  0.9 MiB |    20 |
| `lint/props-deep-destructure-8090/source-custody`          |  0.5 MiB |    47 |
| `lint/runtime-prop-defaults-8564/source-custody`           |  0.5 MiB |    28 |

No Rust test reads these directories, and no tooling test imports their
readers any more (the last ones were removed in #8943). They are read only
through this chain:

1. Seven modules in `tests/tooling/support/`: `no-mutating-props-source-custody`,
   its `current-caller`, `module-layout` and `test-correction` variants,
   `props-deep-source-custody`, `runtime-props-api-source-custody` and
   `setup-export-preset-source-custody`.
2. `tests/tooling/support/component-value-retained-guards.ts` and
   `component-value-host-custody.ts`.
3. `tests/performance/support/warm-type-backed-component-value-host.ts` and
   its manifest, which `qualifyHostMoves` in
   `warm-type-backed-host-qualifications.ts` calls only when a pull request
   introduces that host file. Since the file is on `main`, the branch is
   unreachable for every current pull request.
4. The manifest and host file are also named by the recorded star-listing and
   git-reader ledgers (`warm-type-backed-empty-vue-star-part-05.json`,
   `warm-type-backed-git-reader-component-*.json`,
   `tools/support/compat/github/empty-vue-star-custody-runner.mjs` and
   `source-counterpart-snapshot.mjs`).

## Steps

1. Remove the `componentValueHost` branch from `qualifyHostMoves`, then delete
   the component-value host, its manifest, the two guard modules and the seven
   custody modules.
2. Drop the component-value entries from the star-listing and git-reader
   ledgers in the same change, or retire those ledgers first if the CI lane has
   moved them to recorded commits by then.
3. Delete the four fixture directories.
4. Check with the paired-original workflow (`warm-type-backed` jobs) and the
   star-listing and git-reader tooling tests on a branch before queueing.
