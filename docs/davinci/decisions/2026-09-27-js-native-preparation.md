# Reuse native preparation in the JS package test task

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Evidence

[The observed merge-group JS job](https://github.com/ubugeeei-prod/vize/actions/runs/36300030832/job/108565884127)
took 351 seconds. Its package test step took 187 seconds, followed by 137
seconds of UI conformance checks. The package step compiled the native addon
once in the root task, then again in the Vite package's standalone test
command. The second invocation reported 39.08 seconds of dev-profile Cargo
compilation. A later CI-profile native build belongs to the UI check step.
This is one observed run; it does not establish a median or a speedup.

## Decision

- Keep the root task's native preparation and the existing single package
  test group, including all 16 packages, dependency ordering, concurrency
  limit of one, package pretests, and inherited environment.
- Wrap that exact group with `npm/native/scripts/test-preparation.mjs` only
  after `build:native:test` succeeds. The wrapper creates an exclusive
  receipt for the current checkout HEAD/tree, tracked working diff and addon
  SHA-256. It binds a live owner PID to a unique argument in that process's
  command line, preventing a reused PID from validating a stale receipt.
  The wrapper removes its receipt in `finally`; dead or invalid receipts
  cannot enable reuse and are reclaimed on the next managed invocation.
- Receipt ownership begins after root native preparation and covers the one
  serial package test group in its isolated CI checkout. The guard rejects
  an active owner before another group uses that receipt, and an addon hash
  mismatch prevents reuse. It does not serialize independent root native
  preparations or provide a system-wide native build mutex.
- Linux/macOS process inspection verifies this owner. If command-line
  inspection is unavailable, the wrapper records that reuse is unavailable
  and runs the original group, including Vite's ordinary native build. It
  never skips tests or accepts an unverifiable receipt.
- The fixed `vp` and `pnpm` package commands use the Windows shell to retain
  support for their `.cmd` shims. Node owner processes still launch directly;
  Linux/macOS package commands keep direct process execution.
- The Vite runner uses `test:prepared` during this managed scope. A filtered
  environment variable cannot signal reuse: the first Actions run still
  compiled twice because the root enables package script caching and that
  filters undeclared environment variables. Keep this cache policy intact.
- Pass inherited `process.env` unchanged. The prepared pnpm entry supplies
  its normal lifecycle metadata, such as `npm_lifecycle_event=test:prepared`.
- Standalone Vite `test` still prepares the native addon before running the
  same two test suites. `test:prepared` requires exactly one local addon and
  loads it directly before either suite; it cannot use an installed platform
  package as a substitute. Its caller is responsible for a fresh build.
- Keep UI conformance, its separate CI-profile build, and all merge-queue
  coverage. Package selection and native build profiles are separate work.

## Validation and remaining evidence

The focused tests run the old and new root commands through the real Vite+
runner with fixture packages and the root's script/task caching enabled.
They compare package ordering and inherited environment, verify successful
preparation precedes every package test, and
verify failed preparation prevents all package tests. They also check local
addon prerequisites, owner identity, source/addon drift, stale receipts,
cleanup on failure, and that build or test failures stay failures.

The PR's Actions JS job must show only the root dev-profile native build
during the package test step. Record its timings before making a performance
claim; the UI check's later CI-profile build is still expected.
