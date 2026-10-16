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

## Observe the units before claiming duplicate builds

The controlled run 38923320909 measured 92.278s for the old three selections and
61.165s for `ci`, with identical 19 selected passing cases and 102 Cargo summaries.
It was one old-first warm sample. The following CLI still took 56.24s. Its text
progress repeated 115 crate/version names from the tests, but retained no Cargo
unit metadata or fingerprints. Those names cannot identify equivalent units.

Keep the same three tests and CLI build. Their ordinary stdout now includes
`json-render-diagnostics` records and is retained per command with `tee` and
pipefail. Raw compiler artifacts preserve package, target/kind, profile/test,
features, freshness and output paths; build-script records preserve emitted cfgs.
The collector copies only fingerprint JSON associated with hashes actually
present in emitted output paths, under the canonical target directory. It never
copies compiled objects or the whole target. Host/proc-macro, cross-target, test
and release observations retain their distinct raw identities and paths.
Export paths omit the hidden `.fingerprint` component so the artifact action's
default hidden-file exclusion does not discard the evidence. Original source
paths remain recorded separately in `units.json`.
Rust 1.99.1 pins Cargo revision `5f94df4789f005f9a352888e8355ffc645b7ed0e`.
Its [unit metadata implementation](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/compiler/build_runner/compilation_files.rs#L112-L118)
uses the unit ID for emitted filename/directory suffixes; its fingerprint directory
uses the corresponding package directory. This internal layout is not a public
compatibility guarantee: absent or ambiguous matches remain gaps.

Unhashed final executables and missing/ambiguous fingerprints are recorded as
gaps; matching by crate name is refused. Raw logs and bounded fingerprint JSON
are uploaded even if Cargo or acceptance checks fail. This adds no Cargo build,
test selection, job, cache policy or measurement campaign. It enables attribution
in the next required ordinary run; it does not prove main warm reuse or seed-role
compatibility.
