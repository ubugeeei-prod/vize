# The Davinci `no_std` boundary

> [!NOTE]
> TS-24 builds std-hosted L0 and five level/conversion libraries on
> wasm32-wasip2. The five `#![no_std]` source boundaries do not imply a
> std-less dependency graph. #6834's platform isolation remains unfinished.

## The exact claim

These five library sources retain `#![no_std]` and `extern crate alloc`:

| Level    | Package         | Role                                 |
| -------- | --------------- | ------------------------------------ |
| L1       | `vize_l1`       | lossless surface syntax              |
| L2       | `vize_l2`       | semantic artifacts and summaries     |
| L3       | `vize_l3`       | shared decision contracts            |
| L1 to L2 | `vize_l1_to_l2` | surface lowering and semantic passes |
| L2 to L3 | `vize_l2_to_l3` | decision-layer lowering              |

The compatibility substrate is retired under #6833. Its shared Dump, keys,
fact/pass and diagnostic implementations now belong to the real `vize_l0`
package, alongside storage, configuration, profiling and host services.
L0 is accepted std infrastructure; `vize_carton` remains a legacy facade
in `crates/`. Neither is presented as a `no_std` library.

Rust permits a `#![no_std]` library to depend on std libraries.
wasm32-wasip2 provides Rust std, so successful target builds validate the
accepted closure rather than embedded or std-less support.

## Accepted normal dependency edges

The current workspace metadata has these direct normal edges. Native
level/conversion crates take no normal/build dependency on legacy products.

| Library  | Direct normal dependencies                                                                                           |
| -------- | -------------------------------------------------------------------------------------------------------------------- |
| L1       | L0; `htmlize`                                                                                                        |
| L2       | L0; `oxc_ast`; `oxc_parser`; `oxc_span`                                                                              |
| L3       | L0; `serde_json`                                                                                                     |
| L1 to L2 | L0; L1; L2; L3; `htmlize`; `smallvec`; OXC AST, visit, parser, semantic, span and syntax; opt-in codegen/transformer |
| L2 to L3 | L0; L2; L3                                                                                                           |

L0 and the OXC dependency closure include std-bound code. The L1-to-L2
`typescript` feature is off by default and explicitly imports std for OXC's
transformer path API when selected. The TS-24 commands leave it off; legacy
DOM/SSR products retain their existing explicit erasure feature selection.
Dev-only higher-level and legacy producer oracles are excluded from normal
dependency direction and stripped from L0 packaging when version-less.

## What TS-24 proves

Both library-only builds validate the same six package targets, including
the std L0 foundation, with default features and with defaults disabled.
They retain 32-bit layout assertions, including target-independent NodeId
checks, and reject accidental reliance on the std prelude in the five
unconditional source boundaries. Opt-in std remains explicit.

The removed optimizer host does not reappear as a package or binary.
`vize dump` stays in the std CLI; compiling a host executable would not
establish the library source or dependency-closure claim.

## CI lanes

The full scheduled/manual `clippy-and-test` job runs both commands. Routine
PR and merge-group Check use their separately pinned tier selection.

```sh
cargo build -p vize_l0 -p vize_l1 -p vize_l2 -p vize_l3 -p vize_l1_to_l2 -p vize_l2_to_l3 --lib --target wasm32-wasip2
cargo build -p vize_l0 -p vize_l1 -p vize_l2 -p vize_l3 -p vize_l1_to_l2 -p vize_l2_to_l3 --lib --target wasm32-wasip2 --no-default-features
```

The target is pinned by `rust-toolchain.toml`. The portability workflow law
checks both exact commands, the `--lib` boundary, scheduled/manual placement,
all five attribute pairs, std L0 and absence of the retired package or host
binary. The existing workflow step changes without adding a stage or job.

## Change protocol

A source library joins or leaves this claim only when its attribute, both
CI commands, the workflow law and this ledger change together. The #6833
retirement keeps all five native source boundaries and builds their actual
foundation directly. Platform/no-std isolation and embedded support require
#6834 implementation and evidence; this ownership move does not complete them.
