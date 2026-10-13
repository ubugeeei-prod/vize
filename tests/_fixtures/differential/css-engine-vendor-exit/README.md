# CSS engine fork fixtures

Inputs for the tests that cover the vendored CSS engine forks
(`vendor/lightningcss`, `vendor/parcel_sourcemap`) and their consumers.

- `atelier-inputs.json` — CSS inputs replayed by the
  `css_engine_vendor_observer` example in `crates/vize`.
- `finite-successors.json` — the expected results for the percentage and hue
  inputs that used to panic inside the engine
  ([#3295](https://github.com/ubugeeei-prod/vize/issues/3295)). The Atelier,
  Glyph and Patina boundary tests read it.
- `finite-before/css_engine_panic_boundary.rs.txt` — the boundary test source
  as it was before the engine repairs, read by the Atelier stage-alias test.
- `release-inheritance/metadata/` — a frozen workspace manifest and lockfile
  used as input by the release version-rewrite test. It is test input only;
  the live `Cargo.toml` and `Cargo.lock` are not compared against it.

The candidate qualification that pinned the live workspace manifests and
release drivers to these fixtures was retired; see the
[decision](../../../../docs/davinci/decisions/2026-10-13-3864-css-fork-candidate-custody-retired.md).
