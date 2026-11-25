# Rust build accelerators: parallel frontend on the pinned compiler, kache for local builds

This record supersedes the scope of
[the 2026-11-17 parallel frontend record](./2026-11-17-rust-parallel-frontend.md): the
parallel frontend moves from an opt-in nightly shell to every Linux and macOS build, on
the pinned compiler. Its measurements and pitfalls stay valid background. The
[one Rust toolchain](./2026-10-20-single-rust-toolchain.md) decision is unchanged:
`rust-toolchain.toml` stays the only compiler.

## Decision

- **Parallel frontend everywhere Cargo builds the workspace on Linux and macOS.** Cargo runs
  every `rustc` call through
  [`tools/support/launchers/rustc.sh`](../../../tools/support/launchers/rustc.sh) (`RUSTC_WRAPPER`).
  For a workspace crate it appends `-Zunstable-options --jobs-frontend=N`. The Nix dev shell
  uses `N = 4`. Every CI job that uses
  [`setup-rust-sticky-cache`](../../../.github/actions/setup-rust-sticky-cache/action.yml)
  uses `N = 8`, which the action's `frontend-jobs` input can change, or turn off with `1`.
  Windows legs are unchanged.
- **The pinned 1.100.0 compiler, not a nightly.** Stable 1.100.0 already contains the
  parallel frontend and accepts `--jobs-frontend` when unstable options are enabled.
  The wrapper enables them for that one crate only, with `RUSTC_BOOTSTRAP=<crate name>`.
  The compiler binary, Clippy's lint set, sticky caches and instruction-count identity stay
  the same, and the `nix develop .#parallel-frontend` nightly shell is removed.
- **kache in the dev shell.** The default `nix develop` shell provides
  [kache](https://github.com/kunobi-ninja/kache) 2.3.1 (prebuilt release, pinned to the
  release's own SHA-256 files in [`tools/nix/kache.nix`](../../../tools/nix/kache.nix)).
  The wrapper hands every call to it. Every worktree and target directory on a machine
  shares one content-addressed store. Restored outputs are copy-on-write clones on APFS,
  so a crate that one worktree compiled is neither compiled nor stored again by the next.
- **No kache in CI yet.** CI keeps its sticky target disks. kache rewrites the paths it
  hashes (the checkout, Cargo home and target directory become `/kache/...` in debug
  information), so its outputs are not byte-identical to an uncached build. Release and
  measurement builds must not use it. A CI store also needs its own trusted-only sticky disk
  and a trusted seed before an A/B can show its value. The measurement below sizes that
  follow-up.

Either accelerator can be turned off when entering the shell: `VIZE_RUSTC_FRONTEND_JOBS=1`
or `VIZE_KACHE=` (empty).

## Why a wrapper, and why only workspace crates

The [earlier record's pitfalls](./2026-11-17-rust-parallel-frontend.md#pitfalls-found-while-measuring)
rule out the obvious ways to pass the flag:

- `RUSTFLAGS` replaces `target.<triple>.rustflags`, which is how CI selects the Wild
  linker. Cargo also hashes it into every crate's `-C metadata`, so changing the thread
  count rebuilds everything and misses every sticky target cache. Flags added by the
  wrapper are invisible to Cargo's fingerprints, so existing caches stay warm, and any
  thread count reuses the same artifacts.
- A workspace-wide `RUSTC_BOOTSTRAP=1` changes dependency code. Build scripts probe for
  nightly features. The first attempt routed every call through the flag:
  `proc-macro2`'s probe succeeded, enabled `#![feature(proc_macro_span)]`, and the build
  failed with E0554. A probe that compiled would have changed the dependency silently
  instead.

The wrapper therefore changes a call only when `CARGO_MANIFEST_DIR` is inside the checkout
that holds the wrapper _and_ `--crate-name` equals `CARGO_CRATE_NAME`. A build-script probe
inherits its package's environment but names its own crate, so it never qualifies.
Dependencies, probes and Cargo's `-vV`/`--print` queries reach `rustc` unchanged
([tests](../../../tests/tooling/rustc-wrapper.test.ts)).

## Evidence

### Output identity (local, macOS arm64)

`cargo build -p vize_canon --profile ci` from an empty target at one fixed path, built with
`VIZE_RUSTC_FRONTEND_JOBS=1` (passthrough) and `8`:

- All 335 dependency `.rlib`/`.rmeta` files are byte-identical.
- The 39 differing files are the workspace crates' own `.rlib`/`.rmeta`. Unpacked, every
  object member is identical (257 of 257 in `vize_l0`). Only `lib.rmeta` differs, in 54 bytes
  at a fixed size. `RUSTC_BOOTSTRAP` sets the tracked `unstable_features` option, which
  feeds the crate hash recorded in metadata. Metadata is not linked into executables or
  `cdylib`s, so the code that ships is the same.
- `cargo clippy -p vize_l0 --profile ci -- -D warnings -W clippy::pedantic` prints the same
  diagnostics, in the same order, at `j1`, at `j8` through kache, and from a kache hit.

### kache (local)

Same `vize_canon` build, kache store empty, then a second empty target directory against
the warm store: 88 s, then 16 s (253 of 253 crates restored, 833 MiB cloned into the target
with no copy). The two kache builds are byte-identical to each other. Wall times on the
shared machine (load average 18 to 32) are directional only.

### CI (quiet runner)

TODO: fill from the bench run on `blacksmith-32vcpu-ubuntu-2404`, which compares `j1`, `j4`
and `j8` and kache cold/warm in rotated order over three rounds.
