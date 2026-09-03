# The Davinci `no_std` boundary

> [!NOTE]
> This is the TS-24 portability contract for the Davinci stage libraries.
> It distinguishes a library's `#![no_std]` source boundary from a fully
> std-less dependency graph. Growing or shrinking the set is a review point.

## The exact claim

The claim covers exactly four **library targets**:

| layer                 | Cargo alias in Davinci code | package id      | role                                         |
| --------------------- | --------------------------- | --------------- | -------------------------------------------- |
| shared infrastructure | `vize_davinci`              | `vize_davinci`  | Folio, passes, diagnostics, stage vocabulary |
| S1                    | `vize_s1`                   | `vize_s1`       | lossless surface tree                        |
| S2                    | `vize_s2`                   | `vize_s2`       | neutral semantic IR                          |
| S1 → S2               | `vize_s1_to_s2`             | `vize_s1_to_s2` | Vue surface lowering and S2 passes           |

Every library has both `#![no_std]` and `extern crate alloc`. Its source can
use `core`, `alloc`, and dependency APIs without importing the `std` prelude.
The required wasm32-wasip2 lane builds all four libraries together.

#### One opt-in exception: `vize_s1_to_s2`'s `typescript` feature

The S2 DOM emitter compiles TypeScript templates the way the shipped lane
does — by running each expression through oxc's transformer — and
`Transformer::new` takes a `&std::path::Path`. That one API is the whole
`std` edge, so it rides a feature that is **off by default**:

```toml
typescript = ["dep:oxc_codegen", "dep:oxc_transformer"]
```

With the feature off, nothing changes: the library links no `std`, and an
`is_ts` emit refuses (`typescript_lane_unavailable`) rather than emitting
un-erased TypeScript. With it on, `#![no_std]` still stands and
`#[cfg(feature = "typescript")] extern crate std;` links `std` for that
call alone. Both wasm32-wasip2 lane commands below leave the feature off,
so the portability claim is proved without it; the witness batteries in
`vize_atelier_dom` select it on their dev edge.

### S0 is deliberately outside the claim

`vize_s0` is the workspace dependency alias for the package
`vize_carton`. It is Davinci's allocator, compact-storage, configuration,
profiling, and host-service foundation. Carton defines and bridges std types
and is **accepted std infrastructure by design**; it is not a fifth `no_std`
stage library and must not be presented as one.

Rust permits a `#![no_std]` library to depend on a library that uses std.
Therefore the four attributes describe their source boundary, not a std-less
link. wasm32-wasip2 includes Rust `std`, so the target can compile the accepted
edges below. Embedded targets without `std` remain out of scope.

### The host binary is also outside the claim

`davinci-opt` lives at `crates/vize_davinci/src/bin/davinci-opt/main.rs`.
It reads files, writes output, and returns process exit codes, so it is a std
host tool. TS-24 passes `--lib` explicitly: a WASI build of this binary would
not be evidence that it is `no_std` merely because WASI provides std.

## Accepted dependency edges

The source boundary is honest only when its std-bearing dependencies remain
visible. `cargo tree --edges normal --depth 1` gives this first-degree ledger:

| library                   | direct normal dependencies                                                                                                                                                                       | disposition                                                                                                                                                                     |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vize_davinci`            | `vize_s0`; `vize_davinci_derive`                                                                                                                                                                 | S0 is the accepted std foundation; the proc macro runs on the host and emits `core`-compatible code                                                                             |
| S1 / `vize_s1`            | `vize_s0`; `vize_armature`; `vize_relief`                                                                                                                                                        | accepted std parser/tokenizer and AST construction edges                                                                                                                        |
| S2 / `vize_s2`            | `vize_s0`; `vize_davinci`; `oxc_ast`; `oxc_parser`; `oxc_span`                                                                                                                                   | accepted std OXC expression parsing plus lower-layer edges                                                                                                                      |
| S1 → S2 / `vize_s1_to_s2` | `vize_s0`; `vize_davinci`; S1; S2; `htmlize`; `oxc_ast`; `oxc_ast_visit`; `oxc_parser`; `oxc_semantic`; `oxc_span`; `oxc_syntax`; and, only under `typescript`, `oxc_codegen`; `oxc_transformer` | accepted conversion-layer closure; dependency direction remains downward. The two feature-gated OXC crates are the TS lane's type erasure and are absent from the default build |

The aliases `vize_s0`, `vize_s1`, `vize_s2`, and `vize_s1_to_s2` are the
primary architectural names. S1, S2, and S1→S2 package ids now match that
vocabulary; S0 still retains `vize_carton` until its own compatibility change.

Carton's own direct dependencies include both `no_std`-capable storage crates
(`compact_str`, `smallvec`, `rustc-hash`) and std-bound host services
(`oxc_allocator`, `oxc_syntax`, `pklrust`, `stacker`). OXC crates used directly
by S2 and the conversion library also have no `no_std` marker at the pinned
revision. Those facts are accepted by this WASI contract; they must not be
smoothed into a claim that the dependency closure is std-less.

## Feature-off lane

The default and `--no-default-features` checks cover the same four libraries.
S1 and S1 → S2 currently expose only opt-in differential-corpus features;
neither has a default feature. `vize_davinci` and S2 have no feature table.
The second check is intentionally retained so a future default feature cannot
silently make the portable library graph unavailable.

A speculative `std` feature is not required. The libraries are unconditionally
`#![no_std]`; std behavior stays in S0 and explicit host edges.

## What TS-24 proves

The required lane proves all of the following on a 32-bit target:

- all four library targets and their accepted dependency closure build
  for `wasm32-wasip2`;
- direct accidental reliance on the std prelude in a stage library fails;
- pointer-width-specific layout assertions are correctly guarded;
- target-independent `NodeId` layout assertions remain active;
- default features cannot be required for the portable library graph.

It does **not** prove a std-less link, embedded support, or portability of the
`davinci-opt` host binary.

## The required CI lanes

TS-24 is an unconditional step of `.github/workflows/check.yml`'s
`clippy-and-test` job:

```sh
cargo build -p vize_davinci -p vize_s1 -p vize_s2 \
  -p vize_s1_to_s2 --lib --target wasm32-wasip2
cargo build -p vize_davinci -p vize_s1 -p vize_s2 \
  -p vize_s1_to_s2 --lib --target wasm32-wasip2 --no-default-features
```

`clippy-and-test` is a dependency of the required `test-report` status. The
target comes from `rust-toolchain.toml`, and
`tests/tooling/davinci-portability-lane.test.ts` pins:

- unconditional required-job placement;
- both exact commands and the `--lib` boundary;
- the four `#![no_std]`/`extern crate alloc` attribute pairs;
- the S0 alias and its exclusion from the claim;
- the current `davinci-opt` host-binary path.

The step remains inside the existing job because `check.yml` is already over
the repository's 350-line source ratchet. This change replaces the existing
step without growing that workflow.

## Local reproduction

With the pinned target installed, run the two commands above. A Nix toolchain
without the `wasm32-wasip2` rust-std component needs the matching component
overlaid into its sysroot; CI installs the target from `rust-toolchain.toml`
and remains the authoritative lane.

## Change protocol

A library joins or leaves this contract only when all four surfaces change in
one reviewed slice: its crate attribute, both CI commands, the tooling test's
crate list, and this dependency ledger. Public documentation must use the
same precise wording: four `no_std` stage-library sources over accepted std
edges, founded on std-hosted S0/Carton.
