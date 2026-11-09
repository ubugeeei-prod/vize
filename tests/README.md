# Tests

This directory contains several kinds of evidence. Start with the area that
owns the behavior, then use the corresponding fixture and expected output.

| Path                                                      | Purpose                                                                                                                                                                                                              |
| --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`tooling/`](./tooling/)                                  | Node tests for repository commands, CI contracts, packages, and cross-crate behavior. Files currently live mostly at this directory's top level; search by feature prefix.                                           |
| [`fixtures/`](./fixtures/) and [`expected/`](./expected/) | Vize-owned compiler inputs and their expected output, organized by parser, SFC, VDOM, Vapor, and related lanes. [`vize_test_runner/`](./vize_test_runner/) runs the fixture suites.                                  |
| [`_fixtures/`](./_fixtures/)                              | Pinned third-party Git submodules, real-project inputs, and compatibility baselines. Read its [fixture policy](./_fixtures/README.md) before changing a baseline; do not edit upstream projects to make a test pass. |
| [`snapshots/`](./snapshots/)                              | Real-project check, lint, build, and inspection cases with checked-in snapshots.                                                                                                                                     |
| [`app/`](./app/)                                          | Browser development, preview, and visual regression tests using Playwright.                                                                                                                                          |
| [`editor-conformance/`](./editor-conformance/)            | Editor-client and LSP conformance scenarios.                                                                                                                                                                         |
| [`performance/`](./performance/)                          | LSP latency and churn tests.                                                                                                                                                                                         |
| [`fuzz/`](./fuzz/)                                        | Fuzz targets and regression cases.                                                                                                                                                                                   |
| [`level_test_support/`](./level_test_support/)            | Shared Rust support for Levels test oracles.                                                                                                                                                                         |

The workspace's [`package.json`](./package.json) names the main browser,
snapshot, readiness, and performance commands. Tests under `tooling/` also run
in focused GitHub Actions lanes; inspect the relevant workflow when changing
one of their paths or assumptions. For the Levels stage model and phase-specific
oracles, start at the [Levels guide](../docs/davinci/README.md).
