---
title: Language Engineering Practices
---

# Language Engineering Practices

Vize is a Vue toolchain, but it has the same failure modes as a compiler: tiny syntax changes can
move diagnostics, code generation, editor behavior, package output, and performance at the same
time. This page records the language-processing practices Vize adopts from mature compiler and type
checker repositories, then maps them to Vize's own fixtures, snapshots, parity tests, benchmarks,
and release gates.

## Source Signals

| Source                                                                                                                                    | Practice observed                                                                                                                                                                                                                 | Vize translation                                                                                                                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [`rust-lang/rust`](https://github.com/rust-lang/rust) and the [`rustc-dev-guide`](https://rustc-dev-guide.rust-lang.org/tests/intro.html) | `compiletest` groups UI tests by suite, stores expected output near source cases, uses `tidy` for repository invariants, and tracks ecosystem and performance regressions separately.                                             | Treat compiler-facing changes as fixture changes first. Keep parser/compiler expectations in `tests/fixtures` and `tests/expected`, and keep repository invariants in `tests/tooling/*.test.ts`. |
| [`rustc` ecosystem and perf testing](https://rustc-dev-guide.rust-lang.org/tests/ecosystem.html)                                          | Crater, cargotest, large-project builders, and rustc-perf make broad compatibility and performance risk explicit before or after merging compiler changes.                                                                        | Escalate broad Vue semantics, generated-code shape, or hot-path changes to real-world fixtures, the Vue parity matrix, and the PR benchmark budget instead of relying only on unit fixtures.     |
| [`rust-fuzz/cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) and libFuzzer                                                           | Coverage-guided fuzz targets run arbitrary byte inputs, persist corpora, and minimize crash reproducers before turning them into deterministic regressions.                                                                       | Fuzz parser, lexer, CSS, expression, and template-compile boundaries from `tests/fuzz` with `cargo +nightly fuzz run <target>` before treating crash fixes as complete.                          |
| [Linux kernel testing](https://www.kernel.org/doc/html/next/dev-tools/testing-overview.html)                                              | KUnit covers small white-box units, kselftest covers user-visible system interfaces, KCOV feeds coverage-guided fuzzing, and `perf stat` captures repeatable counter and timing status.                                           | Split tiny crate-level checks from CLI/workspace integration checks, use coverage/fuzzing for arbitrary inputs, and attach profile or benchmark status when hot paths move.                      |
| [Chromium testing and CQ](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/testing_in_chromium.md)                      | Chromium layers hermetic unit, browser, web, Telemetry, and fuzzer tests; CQ/trybots make expensive or flaky lanes explicit, and ClusterFuzz runs discovered fuzz targets at scale.                                               | Keep Vize checks hermetic by default, escalate browser/app behavior to real-world fixtures, use the PR benchmark budget for Telemetry-like status, and retain fuzz reproducers for triage.       |
| [V8 testing](https://v8.dev/docs/test) and [feature launch](https://v8.dev/docs/feature-launch-process)                                   | V8 runs engine suites such as `mjsunit` and Test262, regenerates expected files only after review, uses `tools/run_perf.py` and benchmark compare flows, and requires fuzzing before shipping language features.                  | Treat Vue/TS compatibility changes like language features: cite the source behavior, add scenario tests, compare performance when relevant, and run or schedule fuzzing before promotion.        |
| [`microsoft/TypeScript`](https://github.com/microsoft/TypeScript)                                                                         | The Hereby task graph separates build, format, lint, test, and baseline tasks. Compiler output is reviewed through `tests/baselines/reference` versus local generated output before `baseline-accept`.                            | Keep snapshots as reviewed contracts. A changed `tests/snapshots/*` or Rust `insta` snapshot must be explained by the PR and limited to the changed behavior.                                    |
| [`TypeScript tests/cases/fourslash`](https://github.com/microsoft/TypeScript/tree/main/tests/cases/fourslash)                             | Editor-facing language service behavior is captured as thousands of scenario files rather than inferred from compiler tests alone.                                                                                                | LSP, quick-fix, completion, hover, and incremental editor changes should have scenario-level smoke or integration coverage, not only parser/compiler fixtures.                                   |
| [`microsoft/typescript-go`](https://github.com/microsoft/typescript-go)                                                                   | The native port keeps the TypeScript submodule as the reference implementation, adds minimal compiler tests, writes generated output to `testdata/baselines/local`, and treats reduced `.diff` baselines as convergence evidence. | Compare Vize output with official Vue and TypeScript behavior before introducing a Vize-specific rule. If Vize intentionally diverges, document the reason and the compatibility tier.           |
| [`facebook/flow`](https://github.com/facebook/flow)                                                                                       | Flow keeps directory-shaped integration tests with `.exp` expected output, supports re-recording intentional output changes, and uses action/assertion style `newtests` for editor and server flows.                              | Prefer small scenario fixtures for diagnostics and editor workflows. Re-recorded snapshots are acceptable only after reviewing the diff and keeping generated noise out of the baseline.         |

## Vize Change Classes

Every language-processing PR should name its change class and include evidence from the matching
row. Use the narrowest command during development, then broaden when the change touches shared
behavior.

| Change class                                     | Required evidence                                                                                                           | Common commands                                                                                                                                            |
| ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Parser or AST                                    | Minimal parser fixture, expected AST or error output, and no broad snapshot refresh.                                        | `cargo test -p vize_armature`, `cargo test -p vize_test_runner`, `node tests/tooling/support/generate-expected.ts <fixture>`                               |
| Compiler and codegen                             | Minimal source fixture, DOM/Vapor/SSR expected output, and real-world parity when the emitted runtime shape changes.        | `cargo test -p vize_atelier_dom`, `cargo test -p vize_atelier_vapor`, `vp run --filter './tests' test:build`                                               |
| Semantic analysis, lint, and cross-file analysis | Rule or analyzer fixture, JSON or agent output snapshot, and docs for changed diagnostics.                                  | `cargo test -p vize_patina`, `vp run --filter './tests' test:lint`, `node --test tests/tooling/snapshot-baselines.test.ts`                                 |
| Virtual TypeScript and type checking             | Minimal SFC fixture, mapped diagnostic snapshot, generated virtual TS review, and official Vue or TypeScript parity note.   | `vp run --filter './tests' test:check:fixtures`, `cargo test -p vize_canon`, `vize check --show-virtual-ts <file>`                                         |
| Formatter and LSP                                | Golden formatting output or protocol smoke coverage, plus a focused editor integration check when behavior is user-visible. | `cargo test -p vize_glyph`, `cargo test -p vize_maestro`, `node --test tests/tooling/lsp-smoke.test.ts`                                                    |
| Runtime packaging, release, or docs              | Governance test, smoke install or workflow coverage, and release/readiness docs when production posture changes.            | `node --test tests/tooling/*.test.ts`, `node tools/npm/smoke-release-install.mjs --prepare-manifests --runtime-checks`, `vp run --workspace-root check:ci` |

## Assurance Lanes

Some changes need a second lens in addition to the change class. These lanes make security status,
performance status, and fuzzing evidence explicit in the PR instead of leaving them as reviewer
memory.

| Lane        | Use when the change touches                                                                                          | Evidence to record                                                                                                                                                                                                                                                          |
| ----------- | -------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Security    | URL handling, HTML or SSR output, filesystem/config loading, native loading, package publishing, CI, or credentials. | `security-audit` in `.github/workflows/check.yml`, `vp exec pnpm audit --prod --audit-level moderate`, `cargo audit --deny warnings`, smoke-install runtime checks, pinned GitHub Actions checks, and any focused regression covering the risky input or boundary.          |
| Performance | Parser, compiler, linter, formatter, type-checker, caching, project graph traversal, generated output, or CLI I/O.   | `.github/workflows/benchmark.yml`, `bench/compare-pr.mjs`, `bench/enforce-pr-budget.mjs`, the `pr-benchmark-budget` status, local `bench:*` tasks, and `vize lint --profile`, `vize check --profile`, or `vize fmt --profile` output when the regression needs attribution. |
| Fuzzing     | Byte-oriented parsing, syntax recovery, CSS parsing, JS/TS expression parsing, template lexing, or codegen recovery. | `.github/workflows/fuzz.yml`, `tests/fuzz/Cargo.toml`, `tools/fuzz/seed_corpus.mjs`, `cargo +nightly fuzz run <target>`, uploaded `fuzz-reproducers-*` artifacts, and a minimized deterministic regression after the crash, timeout, or OOM has been understood.            |

## Baseline Policy

- Start with the smallest failing or illustrative case, then accept broader fixtures only when they
  prove a cross-cutting behavior.
- Snapshot and baseline files are user-visible contracts. If a diff changes diagnostics, generated
  code, public CLI output, or editor behavior, the PR should say why the new output is correct.
- Normalize volatile data before it reaches a baseline. Paths, timings, hashes, and environment
  details should not create recurring snapshot churn.
- Keep parity artifacts explicit. `tests/snapshots/check`, `tests/snapshots/lint`, real-world
  fixture snapshots, and the Vue parity matrix are the compatibility record.
- Do not refresh large snapshot baselines unless the PR is about those outputs. When many files move
  together, include a short explanation of the shared cause.

## Escalation Triggers

Reach for broader evidence when a change has one of these shapes:

- Syntax, transform, or virtual TypeScript behavior could affect ordinary Vue applications:
  add or update a real-world fixture and explain parity against official Vue tooling.
- Generated code shape, caching, project graph traversal, or type-aware analysis could move
  throughput: run the local benchmark that matches the surface and rely on the PR benchmark budget.
- URL handling, HTML/SSR output, config loading, package publication, native loading, CI, or
  credential-adjacent code changes: record security-audit status and add the focused regression that
  proves the boundary is still guarded.
- Parser recovery, arbitrary byte input, CSS/template/expression parsing, or crash fixes: run or
  schedule the matching fuzz target, retain the reproducer, and land a minimized deterministic
  regression before closing the fix request.
- LSP, editor, quick-fix, completion, hover, or incremental behavior changes: add scenario-level
  coverage that exercises the user-visible sequence, not just the final diagnostic.
- A snapshot changes because of paths, hashes, ordering, timing, environment, or host platform:
  normalize first, then accept the baseline only if the remaining diff is meaningful.

## Operational Guardrails

Vize keeps these practices executable instead of relying on memory:

- `CONTRIBUTING.md` names the change-class discipline for contributors.
- `.github/PULL_REQUEST_TEMPLATE.md` asks for behavior references, risks, and verification evidence.
- `bench/test-inventory.mjs` reports the current test asset inventory in PR CI.
- `.github/workflows/benchmark.yml` compares base and head CLI performance and enforces a PR budget.
- `.github/workflows/check.yml` runs the `security-audit` job for production npm and Rust
  dependency advisories.
- `.github/workflows/fuzz.yml` runs the `tests/fuzz` cargo-fuzz workspace and uploads crash
  reproducers for parser/compiler triage.
- `docs/release/production-readiness.md` and `docs/release/vue-parity-matrix.md` define when a
  behavior can be called production-ready or compatible.
- `tests/tooling/language-engineering-practices.test.ts` keeps this page, the contribution guide,
  and the PR template wired together.
