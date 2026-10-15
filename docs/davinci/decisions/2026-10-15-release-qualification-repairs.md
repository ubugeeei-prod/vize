# Release qualification repairs for the next cut

The frozen v0.455.0 candidate failed Check and the Nix package build.
Those failures are in current `main`, so the cut is abandoned and the
repairs land before the next cut.

- `code_action_kinds` is mutable only when the native feature extends it.
- The two tooling tests that `vp check` rejected are formatted.
- The Nix source filter keeps `.mjs` files under `crates/`, including the
  build preprocessor embedded with `include_str!`.
- Wasmtime is pinned to `48.0.6`, the patch release that covers the
  2026-10-14 RustSec advisories. The update is `cargo update -p wasmtime
  --precise 48.0.6` and the matching Cranelift/Pulley family only.
