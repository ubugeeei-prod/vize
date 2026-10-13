# Process-sharded canonical corpus observers

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision
restructures the three canonical corpus observers (`dom`, `ssr-pug` and
`reach`) so each one fits in about two and a half minutes of wall time. Every
committed corpus file is still swept by every observer, and every existing
assertion still runs. The evidence keeps the same meaning: the recorder proves
that the shards cover the canonical manifest exactly and that their counters
add up to a complete sweep.

## Why

[PR run 38355807070](https://github.com/ubugeeei-prod/vize/actions/runs/38355807070)
took 730 s for `reach`, 501 s for `ssr-pug` and 194 s for `dom`, and
`s2 dom corpus` waited for the slowest of them. Rust compilation was only
about 23 s per job. The time went to two places.

- **Single-threaded sweeps.** Production reach compiles 39,063 files in four
  shapes in one loop (510 s). The SSR emitter and production sweep is another
  loop (318 s). Both clear and read the process-wide profiler
  (`davinci/vize_l0/src/profiler/core.rs`) around every compile, so threads
  inside one process would mix counters.
- **Fixed costs repeated in every observer job.** Hydration took 64 s, because
  the fixture object cache was only saved by trusted runs that never call this
  workflow. Capturing the corpus identity took 31 s and recording the evidence
  took 28 s. Both ran Git once per fixture repository, one repository at a
  time, and the record step repeats the capture.

## Process shards

`davinci_test_support::corpus::resolve_env_shard` wraps the existing
`resolve_env_sweep`. Without `VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD` it
returns the complete sweep unchanged. With `index/count` it keeps the files
whose position in the complete collected vector is congruent to `index`
modulo `count`. It writes their logical paths, NUL-terminated, to
`VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD_VECTOR` before sweeping, and reports
`davinci-differential corpus shard: ... planned=<all> selected=<slice>`. The
production reach and SSR sweeps use it. The pug, DOM and other corpus lanes are
unchanged.

`tools/support/compat/github/canonical-corpus-shards.ts` runs each sharded
observer:

1. It builds the test binary once with the original package, features and
   test target.
2. It runs concurrent copies of that binary with the working directory and
   corpus variable that `cargo test` used:
   - one base process runs every test except the canonical sweep
     (`--exact --skip <sweep>`);
   - 16 reach shards or 12 SSR shards each run only the sweep
     (`--exact <sweep>`);
   - for `ssr-pug`, the complete pug corpus runs alongside them.
3. It writes one log, `reach.log` or `ssr-pug.log`, with a delimited segment
   per process and its exit status.

The DOM sweep takes 35 s and stays a single process.

## The evidence keeps its meaning

The recorder (`canonical-corpus-observer.mjs`) and the `s2 dom corpus`
finalizer validate a sharded log in `validateShardedObserver`.

- **Process set.** The set of processes must be exactly the planned one, every
  process must exit 0 without a signal, and no evidence may appear outside a
  segment.
- **Test coverage.**
  - The base process passes every test but one and filters out exactly one.
  - Each shard passes exactly the sweep test, and filters out as many tests as
    the base passed, so it ran the same binary.
  - The total still meets the existing minimum test count.
- **Scope.** Each shard carries one canonical closure line with the committed
  gitlink count, and one shard line for its own index and count over the whole
  manifest.
- **Coverage.**
  - The shard vectors are pairwise disjoint.
  - Their union is exactly the path set of `files.json`.
  - Each shard's counted files equal its vector length.
  - The summed files and native-acceptance totals equal the committed file
    count.
- **Counters.**
  - Every zero-failure counter is checked in every shard. These are
    rejections, divergences, unreadable inputs, parse failures and pug
    refusals.
  - Every "proved something" counter is checked on the sum, as before. These
    are templates, comparisons and emitted output.

The receipt pins the SHA-256 of every vector (`vectorSha256`), and the
finalizer validates the vectors again from the downloaded artifacts. A log
without process segments is still validated exactly as before.

On the fully hydrated corpus at `c4f74b48f`, the summed shard counters equal
the single-process counters of run 38355807070:

- **SSR emitter:** files and parsed 39,063, templates 38,767, compared 38,750,
  `s4` 38,717, legacy error skips 17.
- **SSR production:** compared 38,767, `s4` 38,699.
- **Reach, accepted templates:** `dom_inline` 33,640, `dom_module` 33,641,
  `ssr` 38,699 and `vapor` 18,010.

Every rejection and divergence count was zero. The interleaved vectors also
reproduce the order of `files.json`.

## Fixed costs

- **Background build.** Each observer starts its `cargo test --no-run` before
  hydration, as a background process. The observer command's own Cargo
  invocation waits on the build lock and then finds the binaries fresh.
- **Concurrent Git.** Capture and record still run the same read-only Git
  commands for every fixture. `canonical-corpus-batch.ts` runs them as
  per-repository chains, 32 at a time, and the results are checked in
  committed gitlink order with the same errors as before. The finalizer
  validates its three observer artifacts in parallel child processes that run
  the same `validateObserverArtifact`.
- **Fixture objects.** The new `canonical-corpus-objects.yml` workflow seeds
  the object cache from trusted `main`:
  - it runs on a push that changes `.gitmodules` or a gitlink, daily, and on
    dispatch;
  - candidates restore the exact pin set first, then the newest entry with the
    same `.gitmodules` hash, and fetch only the pins that changed;
  - a restored object store grants nothing, because every checked-out byte is
    still verified against the committed tree;
  - hydration now clones with `--jobs 32` instead of 8.

## Selection

`canonical-corpus-selection.mjs` used to select every workflow, every action
and everything under `tests/_fixtures/`. It now selects only paths the
canonical jobs provably read:

- `check.yml`, `pr-source-checks.yml` and the corpus workflow;
- their local actions, and `package.json` for `setup-node`;
- the gitlinks, every committed fixture SFC (production reach sweeps them), the
  pug fixtures, and the three `differential/compiler` folders that the
  observer binaries include;
- the fixture registry and baseline documents the recorder reads.

The same audit found inputs the list had missed: the `vendor/` crates,
`vize_croquis_cf`, `vize_curator`, `reach-budgets.toml`,
`tools/support/common.rs` and the recorder's helper modules. A changed symlink
or gitlink anywhere under `tests/_fixtures/` also selects the corpus, because
the committed sweep follows links.

`tests/tooling/davinci-canonical-corpus-selection-inputs.test.ts` keeps the
list honest. It derives the read set instead of restating it, from:

- the workflow call chain and its transitive local actions;
- the script and import closure of every command the corpus workflow runs;
- the literal fixture and document paths in those scripts and in the observer
  test sources;
- the Cargo path dependencies of the observer binaries.

Replayed over the last two weeks of `main` (1,037 squash merges), the PRs that
select the corpus fall from 766 to 478.

## Expected wall time

These are estimates from the step timings above, not measurements on Actions.
With a warm fixture cache:

| Job                            | Before | Expected                                     |
| ------------------------------ | ------ | -------------------------------------------- |
| `canonical observer (reach)`   | 730 s  | about 140 s; the base process is about 56 s  |
| `canonical observer (ssr-pug)` | 501 s  | about 120 s; the shards take about 30 s each |
| `canonical observer (dom)`     | 194 s  | about 110 s                                  |
| `s2 dom corpus`                | 42 s   | about 30 s                                   |

A cold fixture cache adds roughly 40 s.

## Unfinished

- **Not measured on Actions.** The shard counts (16 and 12) assume that
  Linux scales across 32 vCPUs. A local macOS run scaled poorly because the
  kernel serialized the path lookups. Tune the counts after the first runs.
- **Cold cache.** The object cache stays cold until `canonical-corpus-objects`
  first runs on `main` after this merges. A cold or changed pin still pays
  Git's serial fetch of each pinned commit.
- **Toolchain.** The Rust changes were checked locally with Rust 1.98.1, not
  the pinned 1.99.0.
