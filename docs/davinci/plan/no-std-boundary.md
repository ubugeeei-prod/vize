# The Davinci `no_std` boundary

> [!NOTE]
> TS-24 builds seven libraries on wasm32-wasip2. L0 uses std, five level
> and conversion libraries declare `#![no_std]`, and L4 declares `no_std`
> outside tests. The target supplies Rust std to the dependency graph.

## The exact claim

Five libraries declare `#![no_std]` unconditionally. L4 uses
`#![cfg_attr(not(test), no_std)]` so its unit tests can use the standard
library prelude. All six import `alloc` explicitly.

| Level    | Package         | Role                                 |
| -------- | --------------- | ------------------------------------ |
| L1       | `vize_l1`       | lossless surface syntax              |
| L2       | `vize_l2`       | semantic artifacts and summaries     |
| L3       | `vize_l3`       | shared decision contracts            |
| L4       | `vize_l4`       | emission, including the DOM target   |
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

| Library  | Direct normal dependencies                                                                                                                        |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| L1       | L0; `htmlize`                                                                                                                                     |
| L2       | L0; `oxc_ast`; `oxc_parser`; `oxc_span`                                                                                                           |
| L3       | L0; `serde_json`                                                                                                                                  |
| L4       | L0, L1, L2, L3, L1 to L2, `htmlize`, `serde_json`, OXC AST, visit, diagnostics, parser, semantic, span and syntax, opt-in codegen and transformer |
| L1 to L2 | L0; L1; L2; L3; `htmlize`; `smallvec`; OXC AST, visit, parser, semantic, span and syntax; opt-in codegen/transformer                              |
| L2 to L3 | L0; L2; L3                                                                                                                                        |

L0 and the OXC dependency closure include std-bound code. L1 to L2 and L4 each expose
an opt-in `typescript` feature for OXC's transformer path API. TS-24
exercises the default feature set and the build with defaults disabled.
Dev-only higher-level and legacy producer oracles are excluded from normal
dependency direction and stripped from L0 packaging when version-less.

## What TS-24 proves

Both library-only builds validate the same seven package targets, including
the std L0 foundation, with default features and with defaults disabled.
They retain 32-bit layout assertions, including target-independent NodeId
checks. Both build the L4 DOM emitter under its non-test `no_std`
attribute. The source checks also pin the five unconditional attributes.

The removed optimizer host does not reappear as a package or binary.
`vize dump` stays in the std CLI; compiling a host executable would not
establish the library source or dependency-closure claim.

## CI lanes

The scheduled and manual `rust-clippy-portability` job runs both commands. Routine
PR and merge-group Check use their separately pinned tier selection.

```sh
cargo build -p vize_l0 -p vize_l1 -p vize_l2 -p vize_l3 -p vize_l4 -p vize_l1_to_l2 -p vize_l2_to_l3 --lib --target wasm32-wasip2
cargo build -p vize_l0 -p vize_l1 -p vize_l2 -p vize_l3 -p vize_l4 -p vize_l1_to_l2 -p vize_l2_to_l3 --lib --target wasm32-wasip2 --no-default-features
```

The target is pinned by `rust-toolchain.toml`. The portability workflow law
checks both exact commands, the `--lib` boundary, scheduled/manual placement,
the six library attributes and `alloc` imports, std L0, and absence of
the retired package or host binary.

## Change protocol

A source library joins or leaves this claim only when its attribute, both
CI commands, the workflow law and this ledger change together. The #6833
retirement moved the shared foundation into L0. Platform/no-std isolation and embedded support require
#6834 implementation and evidence; this ownership move does not complete them.
