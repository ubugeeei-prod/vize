# Merge-queue Rust jobs build from seeded target disks

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

Every push to `main` now writes the two Rust target disks that merge-queue
candidates mount read-only:

- `nextest-ci` also receives the queue producer's two workspace builds,
  `cargo clippy --workspace --profile ci` and
  `cargo test --workspace --profile ci --no-run`. The producer mounts this disk
  and runs Clippy in the `ci` profile, the profile PR checks already use.
- `clippy-test` is written by a new seed job that runs the unchanged
  differential feature recipe. The queue's differential job keeps mounting it.

The seed workflow no longer cancels a running seed when the next push arrives.
A newer push replaces a waiting seed, so the disks lag by at most one run.

## Reason

No trusted run wrote `clippy-test`, and the producer built the `ci` profile on
that disk. In the candidate of 2026-10-13 12:14 UTC the producer compiled 315
crates from `libc` and `syn` upward (2 min 50 s before archiving), and the
differential job spent about 5.5 of its 9 minutes compiling.

A seed that every merge cancelled would also leave the PR disk stale during a
merge train, which is when it is used most.

## Tradeoff

Each push to `main` runs the differential recipe once more, about 10 minutes
on one runner. Runner capacity is not the limit: jobs wait a median of 6
seconds for a runner, and a queue candidate takes 20 minutes.

## Scope

Clippy still denies the same lints over the whole workspace; only its Cargo
profile changes, which selects where its artifacts live. The nextest archive
and its receipt are unchanged.
