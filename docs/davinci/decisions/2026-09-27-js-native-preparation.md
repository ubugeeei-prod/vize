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
- Set `VIZE_TEST_NATIVE_PREPARED=1` on that group only after
  `build:native:test` succeeds. The Vite test runner uses `test:prepared`
  after the root preparation instead of invoking the native builder again.
- Standalone Vite `test` still prepares the native addon before running the
  same two test suites. `test:prepared` requires exactly one local addon and
  loads it directly before either suite; it cannot use an installed platform
  package as a substitute. Its caller is responsible for a fresh build.
- Keep UI conformance, its separate CI-profile build, and all merge-queue
  coverage. Package selection and native build profiles are separate work.

## Validation and remaining evidence

The focused tests run the old and new root commands through the real Vite+
runner with fixture packages. They compare package ordering and inherited
environment, verify successful preparation precedes every package test, and
verify failed preparation prevents all package tests. They also check local
addon prerequisites and that build or test failures stay failures.

The PR's Actions JS job must show only the root dev-profile native build
during the package test step. Record its timings before making a performance
claim; the UI check's later CI-profile build is still expected.
