# Share the unused-bindings test and CLI profile

The unused-bindings workflow built its three Rust test selections with the default
test profile, then built the real CLI with `ci`. The successful pre-change run
38901452021 spent 12m13s in the final test build, followed by another 62s CLI
build and 8s seeded recall. These are one run's observed costs, not a measured
before/after speedup.

Use `--profile ci` for the existing `fact_demand`, `fact_spec` and Patina
`unused_setup_bindings` commands. `ci` still inherits `dev`, including debug
assertions and overflow checks; only its existing debug information, incremental
and codegen settings apply. Do not change package selections, test filters,
corpus environment, pinned submodules, assertions or seeded CLI recall. Keep the
`unused-bindings` cache role: compatibility with a different seed role has not
been proved.

For one controlled comparison, manually dispatch **Davinci unused bindings** on
the exact candidate head with `compare-profiles=true`. Its existing runner and
setup run the old three commands first, then the same selections with `ci`.
The helper preserves complete commands, stdout/stderr, elapsed time and
source/compiler/corpus/environment identity before checking results. It requires
identical selected testcase/status multisets and all Cargo summary counts,
including ignored and filtered tests. Missing output or zero selected tests
fails. Both debug-only fact-demand detectors and the corpus spec must actually
pass. Custom harnesses and doctests remain under the original Cargo commands.
The existing real CLI build and both seeded recall tests run afterwards.

The comparison artifact is `unused-bindings-profiles-<sha>`. A result applies to
that source and runtime only. This is a single old-first sample with cache warming;
it does not establish an all-CI speedup. Normal PRs run only the three `ci` commands.
