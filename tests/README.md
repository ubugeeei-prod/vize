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
| [`formal/`](./formal/)                                    | Independent Lean projects and fixtures for formal properties of the HTML content model and Impeto.                                                                                                                   |
| [`performance/`](./performance/)                          | LSP latency and churn tests.                                                                                                                                                                                         |
| [`fuzz/`](./fuzz/)                                        | Fuzz targets and regression cases.                                                                                                                                                                                   |
| [`davinci_test_support/`](./davinci_test_support/)        | Shared Rust support for Davinci test oracles.                                                                                                                                                                        |

The workspace's [`package.json`](./package.json) names the main browser,
snapshot, readiness, and performance commands. Tests under `tooling/` also run
in focused GitHub Actions lanes; inspect the relevant workflow when changing
one of their paths or assumptions. For the Davinci stage model and phase-specific
oracles, start at the [Davinci guide](../docs/davinci/README.md).
