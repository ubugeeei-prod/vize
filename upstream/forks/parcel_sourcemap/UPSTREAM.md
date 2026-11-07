# Pinned Parcel Source Map source

This unofficial Vize fork imports all nine files from the published
`parcel_sourcemap-2.1.1.crate` archive, SHA256
`485b74d7218068b2b7c0e3ff12fbc61ae11d57cb5d8224f525bd304c6be05bbb`.
Its recorded source revision is
[`a136bd6ba9f1d3cded501ae824341be0a54c5ce3`](https://github.com/parcel-bundler/source-map/tree/a136bd6ba9f1d3cded501ae824341be0a54c5ce3).
`UPSTREAM-IMPORT.json` records every original file and checksum;
`Cargo.toml.orig` remains byte-exact.
Its byte-exact copy `UPSTREAM-Cargo.toml` ships in the package; Cargo reserves and
generates package `Cargo.toml.orig` for the effective fork manifest.

The original package declares `license = "MIT"` and author
`Jasper De Moor <jasperdemoor@gmail.com>`. Both metadata fields and all source
notices remain intact. Neither the published archive nor that recorded repository
tree contains a separate LICENSE file. This record does not invent an upstream
copyright notice. Root reviewed this exact archive/tree provenance and retained
MIT/author metadata for the concrete candidate. Publication remains held for the
dependency and API qualification described below.

Reproduce the source-only import with the command in the adjacent LightningCSS
[provenance record](../lightningcss/UPSTREAM.md).

The archive migration imports only the three core changes from
[`dbbd854f83c32fd8cb138489920e043c73da49c0`](https://github.com/atezet/source-map/commit/dbbd854f83c32fd8cb138489920e043c73da49c0),
the first commit of unmerged upstream PR130:

- The effective manifest pins `rkyv = "=0.8.18"`.
- `src/lib.rs` uses `rkyv::to_bytes` and checked `rkyv::from_bytes`.
- `src/sourcemap_error.rs` converts the new rancor error into the existing
  `BufferError` variant.

The original edition 2018, public method signatures, mapping and VLQ algorithms,
JSON/data URL contracts, and all original tests remain intact. Later PR130 N-API,
Wasm, borrowed-argument, edition and unrelated source changes are not imported.
Archive bytes are an intentional rkyv format migration; no claim of compatibility
with old rkyv 0.7 buffers is made, and no unsafe old decoder is executed by the new
malformed-input tests.

A separate vendor compatibility change retains `deny(clippy::all)` and permits
only the ten named diagnostics actually observed on its unchanged legacy spellings
and public std String/format interfaces with Vize's configuration/Rust1.99.
Other upstream diagnostics and all first-party lint/check policy remain intact.

The package is `vize_parcel_sourcemap`, with library name `parcel_sourcemap` and
the synchronized Vize workspace version. Consumers use a direct path with an
exact registry fallback, rather than a root-only Cargo patch. This candidate is
unpublished. The paired [decision](../../docs/davinci/decisions/2026-10-12-css-engine-vendor-exit.md)
keeps actual downstream, source-map and registry acceptance open.
