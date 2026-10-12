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

- A bounded [import-order candidate](../../docs/davinci/decisions/2026-10-12-css-import-order-repair.md)
  retains parallel physical loading and replays conditions/owned errors in the
  existing ordering phase. Same-owner emission edges are pruned after context
  validation; original `SourceIndex` printer semantics remain. All original
  bundled tests and complete failed before packets are frozen. Source-after,
  performance and broader forward-composition qualification remain explicit.
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
- An explicit vendor-policy compatibility commit applies the existing Oxc
  third-party Clippy policy to this implementation. `deny(missing_docs)` remains
  intact; all first-party check commands remain unchanged.
- The optional CLI dependency moves from Clap3 to Clap4 (`^4.5`, locked4.6.6)
  to retire the old atty/proc-macro-error advisory paths. Entire original CLI
  source and tests are retained; all-features debug/release execution is required,
  including any diagnostic wording differences observed during qualification.
- After actual Clap4 failures, four internal CLI group references use distinct
  group IDs while preserving flag/field names and output algorithms. The complete
  original CLI source stays frozen in `tests/vize_fixtures/cli-original.rs.txt`
  and authenticated against its original import hash. That group-only repair
  preserved all27 test bodies; the distinct finite successor is recorded below.
- The one original negative target/browserslist conflict prefix has an explicit
  Clap4 casing successor (`The` to `the`), corroborated by its actual514 release
  failure and official formatter. The entire original27-test authority remains
  frozen as `cli-original-tests.rs.txt`; all other expectations stay byte-exact.
  A separate authored law additionally asserts the complete current exit2,
  stdout and stderr bytes. No runtime or oracle-wide normalization is applied.

Repository `Cargo.toml.orig` remains the original archive manifest. Its byte-exact
copy `UPSTREAM-Cargo.toml` ships in the package because Cargo reserves and
generates `Cargo.toml.orig` for the effective fork manifest. The unreleased
upstream NonNegative restructuring is not imported. Published dependency identity changes
can affect Rust consumers accepting upstream LightningCSS types; the paired
[decision](../../docs/davinci/decisions/2026-10-12-css-engine-vendor-exit.md)
records qualification and publication blockers. This source candidate grants no
installed or published acceptance.
