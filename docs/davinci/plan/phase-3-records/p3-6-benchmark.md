# P3-6 Vapor compile comparison

The manual `Criterion Bench` workflow measures the seven `vapor_native_pair`
fixtures shared by `crates/vize_atelier_vapor/benches/davinci.rs` and
`tools/benchmarks/scripts/vapor-compiler-compare.mjs`. Each fixture compiles
identical source through admitted S3, an explicitly selected retained route,
and pinned `@vue/compiler-vapor@3.6.0-beta.10`, all with identifier prefixing.
The workflow uses one Blacksmith runner for both Rust revisions and the
official compiler, keeps base/head Cargo build graphs isolated, and requires
exact, ancestor-related base/head commit SHAs. It does not run in ordinary
pull-request CI.

From a pushed branch at the exact head commit:

```sh
gh workflow run criterion-bench.yml --ref <head-branch> \
  -f base_sha=<40-character-base-sha> \
  -f head_sha=<40-character-head-sha> \
  -f vapor_only=true
```

The artifact and job summary show base/head Criterion medians, each revision's
native/retained ratios, and an exact-head three-compiler table. The official
compiler is loaded and warmed before nine batches of 100 compiles. Its timing
excludes process startup and package loading, like the Rust Criterion timings.
The artifact records the exact Vize SHA, official version, source hashes and
absolute medians. A ratio below 1 is faster. These are observations, not a
promotion gate: Criterion and JavaScript use different timing harnesses and
runtimes, cases execute sequentially, shared-runner jitter remains, and
`spreads` emits different native and retained programs. At least three
independent reference-runner comparisons with a comparable semantic corpus are
needed to choose and commit a numeric threshold to `budgets.toml`.

The pinned official compiler remains the independent behavioral oracle in
TS-33. A faster median alone cannot replace that parity gate or establish a
production-lane switch. P3-6 stays open while the native route is slower than
the retained lane on admitted cases and until the full semantic corpus and
repeatable reference-runner budget pass.
