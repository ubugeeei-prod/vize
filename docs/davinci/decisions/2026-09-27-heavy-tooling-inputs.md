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
