# Rust build accelerators: parallel frontend on the pinned compiler, kache for local builds

This record supersedes the scope of
[the 2026-11-17 parallel frontend record](./2026-11-17-rust-parallel-frontend.md): the
parallel frontend moves from an opt-in nightly shell to the default dev shell and the Rust
CI jobs, on the pinned compiler. Its measurements and pitfalls stay valid background. The
[one Rust toolchain](./2026-10-20-single-rust-toolchain.md) decision is unchanged:
`rust-toolchain.toml` stays the only compiler.

## Decision

- **Parallel frontend in the dev shell and the Rust CI jobs.** Cargo runs every `rustc`
  call through
  [`tools/support/launchers/rustc.sh`](../../../tools/support/launchers/rustc.sh) (`RUSTC_WRAPPER`).
  For a workspace crate it appends `-Zunstable-options --jobs-frontend=N`. Both the Nix dev shell and CI
  use `N = 8`, the fastest count measured (16 and 32 are slower). In CI,
  [`setup-rust-sticky-cache`](../../../.github/actions/setup-rust-sticky-cache/action.yml)
  takes a `frontend-jobs` input. It defaults to `1` (off), and the Rust jobs set it:
  Clippy, doctests and feature lanes; the Rust test shards; and the Rust feature tests.
  Windows legs are unchanged.
- **Why CI opts in per job.** Jobs that build the native addon for a toolchain-history
  capture (`npm/native/scripts/formatter-history-toolchain.mts`) reject every
  `RUSTC_WRAPPER`. So does the installed-LSP acceptance
  (`tools/support/release/public_acceptance/lsp_assets.ts`). Both guards record the exact
  real toolchain, and a job-wide wrapper failed them on the first PR run. Those guards stay
  as they are, and jobs that run them keep the default.
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

Run [45565292781](https://github.com/ubugeeei-prod/vize/actions/runs/45565292781) on one
`blacksmith-32vcpu-ubuntu-2404` runner, with CI's Wild linker. It visits `j1` (the
wrapper's passthrough, identical to plain stable), `j4` and `j8` in rotated order over three
rounds. The workspace rows start from an empty target: `check` (dev), `clippy`
and `build` (`ci` profile). The edit rows append a public function to `vize_canon` and
rebuild `vize` in a warm dev target. Values are p50 wall seconds; CPU is user plus system
seconds of the whole tree; RSS is the largest single process. Every row's three samples
lie within 2% of each other, except the first `j1` edit-check (29.7 s, the first
incremental check after a build), which the p50 discards.

| Scenario                   | `j1` wall | `j4` wall | `j8` wall | `j8` vs `j1` | CPU `j1` → `j8` | Peak RSS `j1` → `j8` |
| -------------------------- | --------: | --------: | --------: | -----------: | --------------: | -------------------: |
| workspace `check`          |      33.2 |      20.8 |      18.6 |       -44.0% |     175 → 206 s |      1.34 → 1.52 GiB |
| workspace `clippy`         |      34.2 |      21.8 |      19.8 |       -42.1% |     174 → 204 s |      1.35 → 1.53 GiB |
| workspace `build`          |      51.7 |      35.0 |      31.7 |       -38.7% |     312 → 379 s |      1.99 → 2.12 GiB |
| edit `vize_canon`, `check` |      16.9 |       8.4 |       6.9 |       -59.4% |       17 → 24 s |      1.34 → 1.19 GiB |
| edit `vize_canon`, `build` |      33.2 |      20.5 |      17.4 |       -47.7% |      80 → 107 s |      2.66 → 3.50 GiB |

The parallel frontend trades CPU for wall time. At `j8` the CPU cost is +17% to +41%; at
`j4` it is +10% to +24%. The maintainer chose the fastest setting, `j8`, for both CI and the dev shell. kache
offsets the extra CPU locally: a hit costs almost none.

**Output identity on Linux.** Every build's rlibs were unpacked and their object
members hashed. Of 463 rlibs, all 460 that the edit loop did not touch carry identical
object code at `j1`, `j4` and `j8` in every round. The exceptions are `vize_canon`, whose
source the edit loop changed, and `libmimalloc_sys`, which differs between `j1` rounds
too. As on macOS, the rest of the full-file differences are workspace `lib.rmeta` crate
hashes.

**kache on the same runner** (`j8`, empty store, then two fresh targets against it):
`build` 34.5 s on the miss (+9% over `j8` alone, CPU 435 s), then 4.6 s on hits (CPU 14 s,
-96%). The warm `clippy` takes 4.5 s instead of 19.8 s. Its object code differs from the
uncached build in 378 rlibs, because the paths are normalized. That confirms why release
and measurement builds stay without kache.
