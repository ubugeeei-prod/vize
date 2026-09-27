# Inputs for heavy tooling subprocesses

Tracks [#6863](https://github.com/ubugeeei-prod/vize/issues/6863) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830), extending the
[tooling input selection](./2026-09-27-tooling-input-selection.md) catalog.

## Decision

Five existing tests receive explicit input scopes. Test bodies, subprocess
commands, fixture hydration and assertions remain unchanged. T1 continues to
run every test. Shared workspace/task/dependency changes and unknown paths
restore the broad T0 inventory; missing or nonliteral imports restore broad
inputs for the affected scoped test. Direct changes always select the test.

| Scope                        | Tests                                              | Runtime inputs beyond literal imports                                                                                                                                                    |
| ---------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust corpus                  | fact-spec, complexity, metamorphic corpus wrappers | Cargo workspace sources/config/lock/toolchain, tests and hydrated submodule shards, benchmark harness, npm schema inputs, workflow hydration, Davinci plan thresholds/schema/budget data |
| Moon build                   | `moonbit-warnings.test.ts`                         | Entire `tools/moon` module including `moon.mod`, commands and shared library; pinned MoonBit version, setup workflow and Nix toolchain definition                                        |
| Benchmark fixture comparison | `davinci-bench-compare.test.ts`                    | Standalone Rust script, Rust support helpers, explicit fixture budgets/baseline/current reports, Cargo/toolchain config and setup workflow                                               |

The Rust wrappers execute specific Cargo test targets. Their dependency scope
keeps every crate conservatively, all test fixtures and the out-of-workspace
`tools/benchmarks` harness. It also retains `docs/davinci/plan/**`: compiled
Rust tests read threshold Markdown such as `complexity-metrics.md`, and Rust
production code reads `budgets.toml`. Guide-only documents are outside this
scope. A newly introduced runtime input must be added to the catalog when its
consumer changes.

The Moon test enumerates every `tools/moon/cmd/**/main.mbt` and invokes
`runMoonScript(..., { buildOnly: true, denyWarn: true })`. This builds the module
without running its publication/package commands. Unrelated Rust crate source
is therefore outside its inputs. The selector separately includes its
transitive `tests/tooling/_helpers/moonbit.ts` import.

The benchmark test invokes `tools/commands/davinci/bench-compare.rs` through
`rust-script`. Its dependencies are declared in the script header; it imports
`tools/support/common.rs`, not workspace crates. Every invocation overrides
budgets, baseline and result paths with the committed benchmark fixture or a
unique temporary copy. Production plan budget data and unrelated workspace
Rust source are outside these fixture checks.

## Retained broad case

`vite-plus-fast-path.test.ts` packs CLI, unplugin and Vite artifacts, then runs
native and Vite+ tasks from a temporary consumer. Its embedded consumer source
contains `import App from "./App.vue"`. The conservative literal-import scanner
cannot distinguish this generated code from a repository import, so it restores
broad inputs. This change preserves that fallback instead of narrowing an
unresolved dependency closure. Package build inputs need a separate parser and
runtime dependency audit before a scope is introduced.

## Validation and limits

Seven new scope tests plus the six existing selector tests pass without skips.
They exercise each input root, deletions/new fixture paths, transitive helpers,
compiled threshold docs, unknown/global inputs, unresolved imports, direct
changes and complete T1 inventory. Synthetic inventories keep these contract
tests under one second; one real inventory check verifies the actual closures.

The previous successful PR
[Actions run 36303381597](https://github.com/ubugeeei-prod/vize/actions/runs/36303381597)
spent about 20.56/14.53/4.26 seconds in the three corpus wrappers and 7.07 seconds
building Moon commands. These are historical lane observations, not predicted
PR percentiles. A Rust-source change still selects all three corpus gates but
omits Moon compilation and standalone benchmark fixtures. Guide-only changes
omit all five if a tooling plan is requested. Native package build remains.

TODO: measure the resulting exact-head Actions durations after integration and
expand the catalog only after auditing each test's subprocess/read dependencies.
The T0 p50 ≤ 3-minute / p90 ≤ 6-minute target remains unproven.

## Queue source-length comparison base

The source-length gate previously returned no comparison base when
`GITHUB_BASE_REF` was empty. Merge-group events have that empty variable, so
`--check` completed an inventory without enforcing the growth ratchet. The
gate now fetches and compares the immutable `merge_group.base_sha` from the
event. PR checks retain `pull_request.base.sha`; an explicit local
`SOURCE_LENGTH_BASE_REF` retains precedence. Scheduled/local inventory checks
without a PR or merge-group event retain their existing behavior.

Missing event paths, unreadable JSON, malformed commit SHAs and failed fetches
fail the test instead of falling back to inventory-only success. Base resolution
is a small shared test helper; the Rust checker and 350-line limit are unchanged.

A real temporary Git remote advances `main` from a 346-line file to 353 lines.
Both PR and merge-group event checks fetch the earlier SHA and execute the Rust
checker, which rejects the growth. Comparing to the newer branch tip would pass.
Additional tests cover invalid queue metadata, failed fetches and override
precedence. This proves enforcement locally; fresh exact-head Actions and the
actual merge-group result remain required before claiming queue validation.
