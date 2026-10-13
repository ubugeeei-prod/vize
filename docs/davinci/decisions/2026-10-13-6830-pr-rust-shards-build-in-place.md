# PR Rust shards build in place

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision
removes the serial build-then-transfer chain from the PR Rust tier. The same
Clippy, doctest, feature-lane and partitioned nextest checks run, over the
same affected packages, Cargo profile, nextest profile and hash partitions.

## Why

In [PR run 38355807070](https://github.com/ubugeeei-prod/vize/actions/runs/38355807070)
the PR Rust tier was a strict chain. First, one job ran:

- Clippy: 27 s.
- A cold `cargo nextest archive` build: 110 s.
- The parity, surface-corpus and navigation lanes: about 115 s.
- Doctests: 22 s.
- The upload of a 5,021,190,684-byte archive: 26 s.

After that, each of four shards spent about 45 s downloading the archive and
about 30 s extracting it before running for 53-78 s. The first test started
more than six minutes after the plan. Nothing in the second half depended on
anything but the compiled binaries.

## Decision

`pr-rust-build` keeps Clippy, the skeleton ratchet, the L1 parity and surface
corpus lanes, native navigation and doctests. It no longer builds or uploads
an archive. Its display name is now "Clippy, doctests and feature lanes".

On `pull_request`, `pr-rust-shard` no longer waits for that job. Each shard:

1. installs the same Wild linker;
2. mounts the `nextest-ci` sticky role;
3. runs `cargo nextest run @packages@ --cargo-profile ci --profile pr --partition hash:k/4`.

The affected package list is the one the archive was built from. The shard
still writes its timing record and uploads the same `target/nextest/pr/`
results. The Rust source report still requires both jobs, so the required
`test-report` gate is unchanged.

The merge queue keeps its archive unchanged. There, the archive receipt is
part of the typechecker observation evidence, which the source report
aggregates from all four workers.

## Trusted seed

The new `rust-pr-target-seed.yml` workflow runs on every push to `main` and
on a daily schedule, and it is the trusted writer of the `nextest-ci` disk.
It is its own workflow because `check.yml` is already over the source-length
budget.
It reuses a workspace-scope PR's setup exactly:

- toolchain, Wild linker and environment;
- the `plan-affected-rust.mjs` package list (merge-queue context, so every
  crate);
- the Clippy command;
- `cargo test @packages@ --profile ci --no-run`, which builds what
  `nextest run --cargo-profile ci` builds;
- the feature lanes.

Once read-only candidate clones are available
([#8833](https://github.com/ubugeeei-prod/vize/pull/8833)), each PR job rebuilds only the crates a persistent checkout would rebuild.
Without them, each shard still compiles in parallel instead of waiting for one
builder and a multi-gigabyte transfer.

## Measured

On this PR's own run, with no read-only clone and no seed yet, so every job
compiled cold:

- the lane job took 218 s;
- the four shards took 202-227 s;
- the Rust tier finished 227 s after it started, against about 505 s for the
  old build-then-shard chain.

## Tradeoff

Four shards and the lane job each compile the affected test graph, instead of
one builder doing it once, which spends more runner CPU for less wall time.

Narrower `affected` plans unify features over fewer packages than the seed
does, so they reuse fewer seeded units. That stays correct, just less warm.

The ~65 s longest single test remains the floor of each shard's run time.
Merging integration-test binaries, or nextest priorities for the slowest
tests, are separate follow-ups.
