# Parallel PR JS package jobs

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision
splits the serial pull-request JS package job into parallel jobs. No check is
removed or selected away. Every package test, the original Oxlint project
qualification and the Fresco and UI conformance checks still run on every PR
and merge-queue candidate that the source plan selects, with the same
commands, plan guards, skip explanations and evidence uploads. The main-branch
`test-js-packages` and `build-js-packages` jobs in `check.yml` are unchanged.

## Why

`pr-js-packages` ran everything in one job. In
[PR run 38355807070](https://github.com/ubugeeei-prod/vize/actions/runs/38355807070)
its [job](https://github.com/ubugeeei-prod/vize/actions/runs/38355807070/job/115158553744)
took 15 minutes on a 32-vCPU Blacksmith runner:

| Step                                          | Wall time |
| --------------------------------------------- | --------: |
| Checkout, toolchains, cache and install       |      25 s |
| `test:js` (cold native debug build: 59 s)     |     277 s |
| Original Oxlint project qualification         |     418 s |
| Evidence uploads and staging cleanup          |      28 s |
| Fresco native declarations and types          |       8 s |
| UI conformance (`build:ci` 42 s, check 106 s) |     148 s |

`test:js` runs the 16 `testedPackages` one at a time after the native build.
From the same log, the Vite plugin took 88 s, the UI package 57 s (with its
pack, size and tree-shaking pretest), Oxlint 44 s, the composables 10 s, the
Musea Vite plugin 7 s and Nuxt 5 s. Every other package took about a second.

## Decision

`pr-js-packages` in `pr-source-checks.yml` now calls the reusable workflow
`.github/workflows/pr-js-checks.yml` with `run-js` set from
`needs.pr-source-plan.outputs.js`. A reusable workflow is needed because
`pr-source-checks.yml` is held to 350 lines by its tooling tests and was
already at that limit. `pr-rust-source` uses the same pattern.

The called workflow has three jobs. None has a job-level `if`, so none can be
skipped. Each starts with the original skip explanation and guards every
other step on `inputs.run-js`, as the old job did with the plan output.
`source-report` still needs `pr-js-packages`. The caller's result is a
failure when any inner job or matrix leg fails or is cancelled, so the
required `test-report` still gates on all of them.

1. `pr-js-package-tests` is a matrix over the groups in
   `tools/config/vite-plus/tested-package-groups.ts`. Each leg runs
   `vp run --workspace-root test:js:<group>` under the existing
   `native-fault-evidence.py` observer. That task is the full `test:js`
   command restricted to the group: the same `build:native:test`, the same
   `test-preparation.mjs` wrapper and `VP_RUN_CONCURRENCY_LIMIT=1`, with the
   group's `--filter` arguments in `testedPackages` order.
2. `pr-oxlint-project-qualification` runs the unchanged
   `node npm/oxlint/scripts/check-project-transport.mjs` with
   `VIZE_N8N_BASELINE_MODE=partitioned`, then the same evidence upload and
   staging cleanup.
3. `pr-js-package-conformance` runs the Fresco declaration and type checks,
   then `npm/native build:ci` and `npm/ui check`. It mounts the
   `build-js-packages` role, whose trusted writer is the `check.yml` job of
   that name on `main`. That job runs the same commands.

The root `test:js` task, used locally and by the main-branch suite, is
byte-identical to before.

## Package groups

| Group      | Packages                                                        | Measured |
| ---------- | --------------------------------------------------------------- | -------: |
| `builders` | CLI, Vite, Musea Vite, unplugin, Rspack, Nuxt lint config, Nuxt |   ~104 s |
| `ui`       | UI, the four composable packages, Fresco, Marquette, Musea MCP  |    ~68 s |
| `oxlint`   | Oxlint                                                          |    ~44 s |

Some package tests read bundles that an earlier package's test packed in the
single job. The Vite plugin loads `vize/config` from the CLI bundle that the
CLI test packs, and the Musea Vite plugin builds the Vite plugin, which loads
the same bundle. Nuxt loads both and may load `oxlint-plugin-vize`. Each group
therefore holds every tested workspace dependency of its packages, except
those listed as `prebuilt`. Only Oxlint is prebuilt, for the `builders`
group: its `build` script is the `vp pack` its own test starts with, so Nuxt
reads the same bundle it read before. `vp run` still orders a group's packages
by their dependencies.

`tests/tooling/js-package-test-groups.test.ts` enforces this:

- the groups partition `testedPackages` exactly, and the packages their
  directory filters select (including nested packages with a `test` script)
  are the same as the full set's, with none tested twice;
- each group's `prebuilt` list is exactly its outside tested dependencies,
  computed transitively from the workspace manifests, and each prebuilt test
  starts with that package's `build` script;
- each `test:js:<group>` command is derived from the full `test:js` command;
- the workflow matrix lists every group once, in order.

## Dependencies on earlier steps

The qualification used to run only after `test:js` passed in the same job. It
reads two outputs of that step: the formatter-history receipt of the native
debug build, which names the binary it qualifies, and `npm/oxlint/dist`,
which the Oxlint test packs. The qualification job builds both itself with
the new `build:oxlint-qualification` task: the same `build:native:test`, then
Oxlint's `build` script. It no longer waits for the package tests. Its result
still gates the report.

UI conformance reads nothing from `test:js`. The trusted `build-js-packages`
job already runs it without the package tests.

## Evidence

Every upload is kept. The names now carry the job and, for the matrix, the
group, because matrix legs share `github.job` and upload names must be unique:

- each package group uploads its own build's `public-native-formatter`,
  `template-style-fixes`, `oxlint-original-locations` and `js-native-fault`
  artifacts, suffixed with the group;
- the qualification job uploads its own build's `public-native-formatter`
  receipt and the `oxlint-original-locations` artifact with the project
  qualification staging, then cleans the staging as before.

Package groups do not upload `target/oxlint-original-project-transport`. Only
the qualification job writes it, and a group would only find stale entries
from a mounted cache there.

The new jobs do not install MoonBit. No command in them runs `moon`, and
MoonBit tooling is being removed separately.

## The qualification is not sharded

Sharding the Oxlint qualification by branch, host version or n8n input slice,
with an aggregation job joining the shards, was considered and not done here,
because it would weaken what the qualification proves:

- `check-project-transport.mjs` qualifies the addon built in its own job and
  refuses a downloaded one. Every N-API build also receives a fresh
  `VIZE_NATIVE_BUILD_TOKEN` (`npm/native/scripts/native-build-environment.mjs`),
  so two jobs can never build the same binary. `qualification.json` binds the
  host transport results, `mixedDirectory8507` and the n8n replay to one
  `binarySha256`, and `finishN8nReplay` checks that every ledger records the
  same binary. Shards in separate jobs would each qualify a different
  binary.
- Splitting by host or by branch alone would not shorten the critical path.
  The two hosts and `mixedDirectory8507` (116 s) already overlap. The path is
  baseline preparation (199 s), then each host's n8n replay (about 200 s),
  then aggregation (10 s).
- Slicing the n8n inputs changes the replayed claims: each host lints all
  1369 inputs in one run per mode, the baseline uses two fixed contiguous
  partitions, and the ledgers are hashed in order.

Shortening this job needs a maintainer decision between:

1. per-shard builds, with the aggregator binding source and toolchain
   identity instead of one binary;
2. building the addon once and passing it to the shard jobs, which relaxes
   the rule against qualifying a downloaded addon; or
3. keeping one job, but starting each host's n8n replay before the baseline
   is ready, or using more baseline partitions.

## Expected wall time

These estimates add the measured steps above to 25 s of setup, about 15 s
for a warm native build and a few seconds of uploads. They are not yet
measured on Actions. A warm Rust target still recompiles `vize_vitrine`,
because each build has a fresh token. The cold debug build took 47 s.

| Job                                   | Estimate (warm target) |
| ------------------------------------- | ---------------------: |
| Package tests, `builders`             |               ~2.5 min |
| Package tests, `ui`                   |               ~1.9 min |
| Package tests, `oxlint`               |               ~1.5 min |
| Oxlint original project qualification |                 ~8 min |
| Fresco and UI conformance             |                 ~3 min |

The JS path therefore drops from about 15 to about 8 minutes. The unsharded
qualification is now its critical path. UI conformance stays above the
2.5-minute target because `npm/ui check` alone took 106 s. Each PR now uses
five runners instead of one and builds the debug addon four times.

## Unfinished

- The PR wall time has not been measured after this change.
- The Oxlint qualification is unsharded, pending the decision above.
