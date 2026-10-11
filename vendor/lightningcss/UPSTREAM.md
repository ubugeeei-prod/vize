# Pinned LightningCSS source

This unofficial Vize fork imports the Rust implementation, tests, examples and
benchmarks from the published `lightningcss-1.0.0-alpha.72.crate` archive.
The archive SHA256 is
`d31b760f96e8fdfe1d0c295e4bf76c503d6f15d2d470d53bd8cd1f7aa8c7d934`;
its recorded source revision is
[`c6a0c3cebf3395635e61075d2c81a96a710d4910`](https://github.com/parcel-bundler/lightningcss/tree/c6a0c3cebf3395635e61075d2c81a96a710d4910).

The adjacent LICENSE is the unchanged upstream MPL-2.0 license. Its SHA256 is
`5eba353fe5076ac3432177f8ab1cf75e3afcd0584251e37c3bfead5f447d040e`.
Original author metadata and source notices are retained. The source-copy commit
contains 121 byte-exact original files; `UPSTREAM-IMPORT.json` records every
selected original file. Website, npm bindings and upstream automation are outside
this Rust package's scope. No selected Rust source or original test is omitted.

Reproduce the source-only import from both authenticated archives with:

```sh
vp node tools/support/dependencies/vendor-css-engines.ts \
  lightningcss-1.0.0-alpha.72.crate parcel_sourcemap-2.1.1.crate
```

The following changes are deliberately separate from that import:

- Package `vize_lightningcss`, with library name `lightningcss`, synchronized
  workspace version and an exact registry fallback. Its optional source-map
  dependency uses the corresponding `vize_parcel_sourcemap` alias. Original
  features, bundler, CLI, tests and benchmarks remain available.
- `Percentage::parse` returns `InvalidValue` instead of executing `unreachable!()`
  when a calculation does not resolve to a percentage. No other calc body changes.
- HSL-to-sRGB conversion retains the old finite normalization whenever it is in
  range; invalid finite results use Euclidean remainder, and non-finite hues use
  zero as required by CSS Color 4. The cssparser-color assertion remains intact.
- Explicit Vize regression test registration and package provenance. The original
  ignored non-root release profile is removed; Vize's workspace profile applies.
- A separate formatting-only compatibility commit wraps attributes/calls in
  `bundler.rs`, `lib.rs` and `selector.rs` and removes one selector blank line for
  current Rustfmt. Complete original source hashes remain in the import record;
  the custody guard authenticates each of these exact formatted successors.

`Cargo.toml.orig` remains the original archive manifest. The unreleased upstream
NonNegative restructuring is not imported. Published dependency identity changes
can affect Rust consumers accepting upstream LightningCSS types; the paired
[decision](../../docs/davinci/decisions/2026-10-12-css-engine-vendor-exit.md)
records qualification and publication blockers. This source candidate grants no
installed or published acceptance.
