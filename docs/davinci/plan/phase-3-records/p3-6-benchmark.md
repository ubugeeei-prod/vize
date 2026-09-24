# P3-6 Vapor compile comparison

The manual `Criterion Bench` workflow can measure only the seven
`vapor_native_pair` fixtures in `crates/vize_atelier_vapor/benches/davinci.rs`.
Each fixture compiles identical source and options through the admitted S3
route and an explicitly selected retained route. The workflow uses the same
Blacksmith runner for both revisions, keeps their Cargo build graphs isolated,
and requires exact, ancestor-related base/head commit SHAs. It does not run in
ordinary pull-request CI.

From a pushed branch at the exact head commit:

```sh
gh workflow run criterion-bench.yml --ref <head-branch> \
  -f base_sha=<40-character-base-sha> \
  -f head_sha=<40-character-head-sha> \
  -f vapor_only=true
```

The artifact and job summary show the base/head Criterion comparison and,
separately, each revision's native/retained median ratio. A ratio below 1 is
faster. The native/retained rows are observations, not a promotion gate:
Criterion executes cases sequentially, shared-runner jitter remains, and the
`spreads` fixture emits different native and retained programs. A later gate
must pin repeatable reference-runner measurements and an explicit comparable
corpus before adding a numeric threshold to `budgets.toml`.

The pinned `@vue/compiler-vapor` 3.6.0-beta.10 oracle is used for behavioral
parity. This Rust Criterion lane does not time the JavaScript compiler: its
different runtime, API boundary, and output would make a raw ratio misleading.
An external throughput claim needs a separate same-corpus harness that reports
those boundaries explicitly.
