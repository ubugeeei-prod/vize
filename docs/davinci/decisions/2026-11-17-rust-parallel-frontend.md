# Rust parallel frontend: local opt-in, not a CI or release toolchain

Since `nightly-2026-11-14`, rustc runs its frontend on two threads by default and
accepts `-Zunstable-options --jobs-frontend=N`
([announcement](https://blog.rust-lang.org/2026/11/16/enabling-parallel-frontend-on-nightly/),
tracking issue [rust-lang/rust#113349](https://github.com/rust-lang/rust/issues/113349)).
This record measures it on the vize workspace and decides where it is used.

## Decision

- **Local check, clippy and test loops** may opt into a rustup-managed nightly with
  `--jobs-frontend=8`, in a separate target directory. The command is in
  [CONTRIBUTING](../../../.github/CONTRIBUTING.md#setup). Eight threads is the
  measured optimum on a 32-vCPU runner; 16 and 32 threads are slower again.
- **CI gates, release builds, published artifacts and measurements** stay on the single
  pinned `rust-toolchain.toml` compiler, as [one Rust toolchain](./2026-10-20-single-rust-toolchain.md)
  decided. No workflow, cache key or Cargo configuration changes.
- [`rust-frontend-threads-bench.yml`](../../../.github/workflows/rust-frontend-threads-bench.yml)
  (manual dispatch) and its driver
  [`rust-frontend-threads-bench.ts`](../../../tools/support/github/rust-frontend-threads-bench.ts)
  stay so the comparison can be repeated for a newer nightly or when `--jobs-frontend`
  reaches stable. Stabilization is the trigger to revisit CI: it would then be a flag on
  the pinned compiler, not a second toolchain.

The wins are real but land where CI spends little time. CI runs on warm sticky
target caches, so a PR recompiles only the touched crates and their dependents.
Moving the required Clippy gate to nightly would change the lint contract (nightly
Clippy lints differ from 1.99.1), duplicate every sticky cache per toolchain, and
make required checks depend on nightly breakage, for seconds per job. Release
builds must stay reproducible on the toolchain that is published and audited.

## Method

All numbers come from `blacksmith-32vcpu-ubuntu-2404` runners (32 vCPUs, 113 GiB),
the class every Rust CI job uses, with CI's Wild linker configuration. Each job
runs on one machine and visits every configuration once per round in rotated order;
three rounds per configuration, reported as nearest-rank p50 and p95 (with three
samples p95 is the slowest run). CPU is user plus system time of the whole process
tree; peak RSS is the largest single process (`/usr/bin/time -v`).

- `stable` is the pinned 1.99.1; `jN` is `nightly-2026-11-16`
  (`rustc 1.102.0-nightly (5f4effd57 2026-11-13)`) with `--jobs-frontend=N`.
  `j1` isolates the compiler-version difference from the thread count.
- Workspace rows start from an empty target directory: `cargo check --workspace`
  (dev), and `clippy`, `build` and `test --no-run` with `--profile ci`.
- Crate rows recompile one crate with warm dependencies through `cargo rustc`, so the
  flag reaches exactly one rustc invocation: `check` (no incremental), `build`
  (`ci` profile) and `opt` (`ci-opt`, thin LTO).
- The incremental row adds one public function to `vize_canon` per iteration, then
  runs `cargo check` and `cargo build` of the `vize` binary in the dev profile.

Runs: [44022926005](https://github.com/ubugeeei-prod/vize/actions/runs/44022926005)
(all scenarios),
[44025453377](https://github.com/ubugeeei-prod/vize/actions/runs/44025453377) (Clippy and
diagnostics order with the flag forwarded to `cargo-clippy`, plus a second pair of release
builds) and [44025584681](https://github.com/ubugeeei-prod/vize/actions/runs/44025584681)
(stable reproducibility control). The Clippy row comes from the second run; the first
run's Clippy rows are void (see the pitfalls below).

## Results

Wall time p50 in seconds; deltas are against stable and against nightly `j1`.

| Scenario                               | stable |    j1 |    j2 |    j8 |  j32 | j8 vs stable | j8 vs j1 |
| -------------------------------------- | -----: | ----: | ----: | ----: | ---: | -----------: | -------: |
| workspace `check`                      |   33.0 |  32.9 |  25.4 |  18.0 | 24.9 |       -45.3% |   -45.2% |
| workspace `clippy --profile ci`        |   35.5 |  37.1 |  29.5 |  22.3 | 29.8 |       -37.1% |   -39.8% |
| workspace `build --profile ci`         |   52.8 |  51.8 |  45.8 |  31.9 | 37.6 |       -39.5% |   -38.3% |
| workspace `test --no-run --profile ci` |  110.5 | 106.9 | 104.1 |  94.4 | 99.3 |       -14.5% |   -11.6% |
| edit `vize_canon`, `check` `vize`      |   17.0 |  16.5 |  11.5 |   6.5 |    — |       -61.6% |   -60.6% |
| edit `vize_canon`, `build` `vize`      |   33.5 |  31.4 |  27.0 |  16.4 |    — |       -51.0% |   -47.7% |
| `vize_canon` check                     |    4.5 |   4.9 |   3.4 |   1.8 |  2.8 |       -59.6% |   -62.7% |
| `vize_canon` build                     |    8.4 |   9.0 |   7.7 |   4.6 |  5.4 |       -45.5% |   -49.1% |
| `vize_canon` opt                       |   14.1 |  14.5 |  13.3 |   9.0 | 10.8 |       -36.3% |   -38.3% |
| `vize_patina` check                    |    5.0 |   5.4 |   3.7 |   2.0 |  3.3 |       -59.0% |   -62.6% |
| `vize_patina` build                    |    9.3 |   9.9 |   8.5 |   5.2 |  6.2 |       -44.2% |   -47.7% |
| `vize_maestro` check                   |    5.8 |   4.3 |   3.1 |   1.8 |  3.3 |       -68.9% |   -57.7% |
| `vize_maestro` build                   |   11.2 |   7.8 |   6.9 |   4.3 |  5.5 |       -61.7% |   -45.3% |
| `vize` (lib) check                     |    2.2 |   2.3 |   1.7 |   1.0 |  1.4 |       -53.0% |   -55.6% |
| `vize` (lib) build                     |    4.2 |   4.3 |   3.8 |   2.8 |  3.1 |       -32.1% |   -34.5% |
| release `vize` binary (fat LTO)        |      — | 358.0 |     — | 340.2 |    — |            — |    -5.0% |

`j4` and `j16` were also measured for single crates and the incremental loop: `j4` sits
between `j2` and `j8` (`vize_canon` check 2.3 s, incremental check 8.0 s); `j16` is
slower than `j8` everywhere (`vize_canon` check 2.5 s). Every row's p95 is within 4% of
its p50.

Costs:

- **CPU.** Parallel work is not free: at `j8` the workspace build burns 467 CPU-seconds
  instead of 319 (+46%), the test build 1568 instead of 1215 (+29%). `j32` costs more
  CPU and is slower than `j8`. On a shared or already saturated machine the gain shrinks
  toward the `test --no-run` figure.
- **Memory.** Peak RSS grows modestly: `vize_canon` build 1.44 GiB (stable) to 1.74 GiB
  (`j8`, +21%), workspace test build 3.37 GiB to 3.62 GiB (+7%), release build unchanged
  at 5.5 GiB.
- **Nightly overhead.** `j1` is 3 to 10% slower than stable on most single crates (the
  parallel-capable compiler's synchronization), except `vize_maestro`, which nightly
  compiles 26 to 30% faster regardless of threads.

## Determinism

Two independent runs each built the shipped release binary
(`cargo build --release -p vize`, fat LTO, one codegen unit) twice at `j8` from an empty
target directory at the same path, and hashed every `.rlib`, `.rmeta` and the binary.

- In all four `j8` builds every rustc-produced artifact is bit-identical: 677 of 679
  artifacts match. The binary and one rlib differ: `libmimalloc_sys`, which embeds the C
  allocator that the `cc` crate compiles.
- The stable control shows the same thing: two stable release builds differ in exactly the
  binary and `libmimalloc_sys`. The shipped binary is therefore already not bit-reproducible
  on stable, and the parallel frontend adds no further difference. The `ci` profile
  workspace builds are consistent with it: the `vize` binary hash differs between rounds of the same
  configuration, stable included.
- `j8` against `j1` cannot be compared byte for byte, because the flag changes every
  crate's `-C metadata` hash and with it every artifact name and symbol.
- Diagnostics order: `cargo clippy -p vize_canon -- -W clippy::pedantic` emits 1272
  `vize_canon` diagnostics on the nightly. Three `j8` runs and one `j1` run print them in
  the same order; three stable runs also agree with each other (1224 lines). No reordering
  showed up, though the announcement warns that it can.

No compiler crash and no rustc-attributable non-reproducible output occurred, so nothing
is reported to rust-lang/rust#113349. The `libmimalloc_sys` difference is a separate,
pre-existing release-reproducibility gap.

A secondary local data point (the shared macOS arm64 development machine, 15 CPUs, load
average 32 to 37 from concurrent builds) shows the same direction with much more noise: `vize_canon` check p50 is
24.5 s on stable, 29.1 s at `j1`, 9.4 s at `j4` and 14.4 s at `j8` (p95 up to 1.6 times the
p50).

## Pitfalls found while measuring

- `cargo --config ... clippy` does not forward `--config` to `cargo-clippy`; the flag
  must follow the subcommand (`cargo clippy --config ...`) or go through `RUSTFLAGS`.
  The first Clippy run silently measured nightly's default two threads for every
  configuration.
- `RUSTFLAGS` replaces `target.<triple>.rustflags` (CI's Wild linker) and
  `build.rustflags`; the Nix shell sets `RUSTFLAGS` on Darwin. Append to it rather than
  setting it, and do not put `-Zunstable-options` in shared Cargo configuration: stable
  rustc rejects it.
- Different `--jobs-frontend` values give different `-C metadata` hashes, so switching
  thread counts in one target directory rebuilds everything.
