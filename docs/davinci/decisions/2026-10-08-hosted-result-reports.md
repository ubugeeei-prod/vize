# Hosted result reports

Paired issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The protected #8130 candidate `25863a10fe4d97232462c141b714bb1aadec4984`
finished its Rust producer jobs, but its Rust source report remained queued
without a runner on `blacksmith-32vcpu-ubuntu-2404` (job `113064241844`,
created at 2026-10-07 23:13:40 UTC). The report needs no Rust compilation or
native execution. Waiting for the same large runner pool delays the final
required result after the expensive work has finished.

Move these existing jobs to the standard GitHub-hosted `ubuntu-24.04` runner:

- `pr-rust-checks.yml`: `rust-source-report`.
- `pr-source-checks.yml`: `source-report`.
- `check.yml`: `test-report` and `test-report-comment`.

The Rust report checks job results, authenticates the four current Rust worker
artifacts against GitHub and the committed source, verifies their hashes and
JUnit observations, and aggregates captured typechecker results. Its only
subprocess queries the checkout with Git. The four observed compressed worker
artifacts total 2,794,333 bytes; the report never downloads or executes the
compiled test archive. Source aggregation consumes job results. The outer
inventory scans source and parses TOML after its existing filtered dependency
installation with lifecycle scripts disabled. Commenting reads that inventory
and GitHub metadata using the existing trusted base checkout.

All job identifiers, displayed names, dependencies, conditions, permissions,
steps, action pins, artifact integrity checks, strict failure propagation and
five-minute limits remain unchanged. Rust builders and workers retain their
existing runner and runtime envelope. Instruction ceilings, native workloads,
Real Project Matrix and release contracts are outside this change.

Fresh exact-head Actions and protected candidate results must qualify the
change. This removes the observed dependency on the large runner pool; it does
not promise that GitHub-hosted scheduling or artifact services never wait.
