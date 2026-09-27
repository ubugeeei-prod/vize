# Affected Rust checks (#6862)

T0 selects the packages owning changed files and every transitive reverse
dependency reported by `cargo metadata --no-deps --locked`. Cargo's manifest
dependencies retain normal, dev and build edges, aliases, optional edges and
target-specific edges; an inactive Linux dependency must not hide a Windows
consumer. Package-owned tests, fixtures and examples select the same owner.

Root configuration, shared fixtures and expected output, removed package
directories, unknown inputs and empty diffs select every workspace package.
Git diffs disable rename detection so deletion and both sides of a move remain
visible. Missing or malformed metadata fails the planner. Markdown
documentation is exempt; executable or corpus files under docs are not.
`editors/zed` declares an independent workspace and remains in the editor lane;
the Rust plan explicitly reports its excluded paths.

T1 always selects every workspace package. T0 selection cannot reduce the full
merge queue differential recipe, mandatory tsgo coverage or acceptance gates.

`plan-affected-rust.mjs BASE HEAD EVENT OUTPUT` writes schema version 1 with
`scope`, sorted `packages`, `cargoArgs`, directly `changedPackages`, `reasons`
and `excludedPaths`. Its Actions `rust-plan` output contains the JSON itself,
so consumers in another job materialize it in their own runner directory.
`run-affected-rust.mjs PLAN cargo … @packages@ …` inserts validated package
arguments at the explicit marker and starts Cargo without a shell. Empty plans
cannot become unqualified default-workspace Cargo invocations.

The selector alone makes no p50/p90 claim. Archive compilation, concurrent test
execution and runner timing measurements are recorded with the workflow change.
