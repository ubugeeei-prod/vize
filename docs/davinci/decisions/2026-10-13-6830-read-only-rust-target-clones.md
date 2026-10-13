# Read-only Rust target clones for candidates

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision lets
pull-request and merge-queue jobs on Blacksmith mount a never-committed clone
of the trusted Rust sticky disks. It also makes Cargo's freshness check survive
a fresh checkout. No build, test, lint, fixture, receipt or evidence step is
removed or selected away; every job still compiles and checks its exact source.

## Why

PR jobs used to restore only an Actions seed of `target/`. In
[PR run 38355807070](https://github.com/ubugeeei-prod/vize/actions/runs/38355807070)
and [docs PR run 38353645084](https://github.com/ubugeeei-prod/vize/actions/runs/38353645084)
the target restore missed for the JS package job (`test-js-packages`), all
eight tooling shards (`test-scripts`) and the Rust archive build
(`nextest-ci`). Each of those jobs compiled the whole dependency graph cold:
about 50 s per tooling shard, 56 s for the JS native build and 110 s for the
Rust archive. Trusted roles commit warm sticky disks, but candidates never
read them, and the Actions seeds they fall back to were absent for these roles
(a mounted `test-scripts` target never saves one). The `Check` run therefore
took 17-39 minutes even when nothing in Rust had changed.

## Decision

`cache-policy.mjs` keeps the trust boundary: only a `push`, `schedule` or
`workflow_dispatch` run on the default branch, checking out its own source, is
trusted. `pull_request` and `merge_group` runs on a self-hosted (Blacksmith)
runner now mount the same stable registry, git and target disk keys with
`commit: "false"`. Blacksmith clones the last committed snapshot and
[discards the clone](https://github.com/useblacksmith/stickydisk/blob/3f6be1451e3cad893b778bc93495c80452a5da26/src/post.ts)
at the end of the job, so no candidate can write to the disk a later trusted
run reads. Candidate keys never contain a PR number, a queue SHA or a branch
name. Other untrusted events keep their previous behavior. When a mount fails,
the existing restore-only Actions seeds still fill that path. When it
succeeds, those restores run in lookup-only mode and never overwrite the
clone.

The sticky action is pinned to commit
`3f6be1451e3cad893b778bc93495c80452a5da26` (tag `v1.7.2`), the first release
line with the `commit` input. Its post step still commits a trusted disk only
when no earlier step failed or was cancelled.

## Only Cargo's cache is reused

Tests and tools also write run outputs under `target/`, such as differential
evidence, staging projects, receipts beside binaries and nextest reports. On a
persistent disk these accumulate. The first read-only run uploaded a
2,747,454,784-byte Oxlint evidence artifact (37,197 files) instead of the
235,349,320 bytes of a fresh run. An output that a run did not produce could
also be read or uploaded as if it had.

Before anything else, the `reuse-rust-target` step therefore prunes the
mounted or restored target to Cargo's build cache:

- the root `.rustc_info.json` and `CACHEDIR.TAG`;
- each profile's `.fingerprint`, `build`, `deps`, `incremental` and
  `examples`;
- target triples and nested target directories, recognised by a profile with a
  `.fingerprint` within three levels.

Top-level files in a profile go too. Cargo relinks uplifted binaries and
dep-info for fresh units without recompiling, which a real Cargo test checks.
Trusted runs prune before committing, so their own uploads also stop carrying
old evidence.

## Source mtimes

Cargo treats a path source as dirty when one of its inputs is newer than the
recorded build. A fresh checkout stamps every tracked file with the checkout
time, so even a warm target rebuilds every workspace crate and test binary.
`sccache` does not help here, because it cannot cache the linked test binaries
that dominate this workspace's build.

The new `reuse-rust-target` action, registered last in
`setup-rust-sticky-cache`, emulates the persistent working tree that Cargo
assumes. It is TypeScript that the runner's Node 24 executes through native
type stripping, with no build step:

- Trusted runs record each tracked file's content (blob ID) and the whole
  second it was given, as `.vize-source-mtimes.json` inside the target
  directory.
- Every Linux run gives a file its recorded mtime only when its current
  content is identical, including content edited before the step. Every other
  file gets a time newer than every fingerprint in the target directory and
  every recorded value.
- The post step of a trusted run hashes files whose mtime moved during the
  job and records their final content and time. It runs before the Actions
  seed save and before the sticky disk commit.
- A manifest marked `pending` (the post step never ran), a manifest for
  another work-tree root, or an unreadable one is ignored. When another target
  directory in the same job shares the work tree, the step leaves mtimes
  untouched and a trusted run deletes the record.
- Any error leaves the checked-out mtimes, which are always conservative.
- `release.yml` opts out with `reuse-target: "false"`, so release binaries
  keep building every workspace crate from their fresh checkout.

Cargo therefore rebuilds exactly what a persistent checkout would rebuild.
`tests/tooling/reuse-rust-target.test.ts` drives real Cargo through fresh
clones that share one target directory. It checks that:

- nothing rebuilds when the source is unchanged;
- a library change rebuilds that library and its dependent;
- a read-only clone that changes a build-script input rebuilds only that
  package;
- a file rewritten during a trusted run is rebuilt again for the next
  candidate.

Its git-only cases cover pre-step edits, unfinished records, shared work trees
and foreign roots.

## Matching trusted writers

A clone only helps when a trusted job builds the same units. The PR
playground job previously read the `playground-test` role, which only a daily
job wrote, without building the native addon or packages. It now reads
`build-js-packages`, whose trusted `main` job runs the same
`npm/native build:ci` and `build:packages` commands with the same Wild linker
flags.

## Unfinished

The trusted disks gain their first manifest on the first `main` push after
this merges. Until then, candidates reuse only third-party crates.

Several PR jobs use roles that no trusted job writes with the same profile and
features, so they still compile cold:

- the `nextest-ci` archive build;
- the canonical corpus observers;
- the type checker phase evidence.

Follow-up changes under #6830 will add trusted writers for those roles and
split the remaining serial PR jobs. The net PR wall-time after integration has
not yet been measured.
