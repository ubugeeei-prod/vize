<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/logo-dark.svg">
    <img alt="corsa-bind" src="./assets/logo.svg" width="320">
  </picture>
</p>

<p align="center">
  A stable systems interface for the native TypeScript compiler — over stdio, with no forks and no patches.
</p>

---

`corsa-bind` is a multi-crate workspace for talking to [Corsa](https://devblogs.microsoft.com/typescript/typescript-native-port/)
(the native TypeScript 7 implementation line) from Rust and JavaScript runtimes.
Hot paths live in Rust and stay zero-cost; `napi-rs`, Rustler, and a shared C
ABI carry that performance to JS/TS, Elixir, C, C++, Go, Zig, C#, Swift, and
MoonBit — so you can author custom checker tooling and lint rules without
reimplementing the checker.

It is deliberately _not_ a reimplementation of the TypeScript Compiler API. It
is the layer between upstream's checker and everything that wants to use it:

```text
  Vize · Oxlint · Node · Rust · C · Go · Swift · …
                       │
  ┌────────────────────▼────────────────────┐
  │              corsa-bind                 │
  │  stable API/ABI · worker pools          │
  │  snapshot + process lifecycle           │
  │  virtual projects · cancellation        │
  │  backpressure · observability           │
  └────────────────────┬────────────────────┘
                       │
        upstream native compiler service
                 TypeScript 7+
```

As upstream's own API matures, this layer should get _thinner_ — and more
important. See the [Architecture charter](./docs/architecture_charter.md).

> [!WARNING]
> This repository is still evolving. The local Rust and Node API/LSP surfaces
> are hardened for production-style use, but some upstream-facing endpoints
> remain explicitly experimental.

> [!IMPORTANT]
> `corsa-bind` is built around upstream-supported Corsa workflows. We follow
> Corsa's recommended stdio/API/LSP integration points, keep
> `ref/corsa-upstream` as an exact upstream checkout, and preserve a strict
> **no forks, no patches** policy.
>
> The second half of that policy is **own integration, not TypeScript
> semantics**: checker semantics are never ours, checker lifecycle is
> aggressively ours, and when upstream ships something we approximate today, we
> delete ours and wrap theirs. The
> [Architecture charter](./docs/architecture_charter.md) spells out what that
> rules in and out.

## Quick start

Enter the Nix dev shell and build the workspace:

```bash
nix develop
vp install
vp run -w build
```

A first program in Rust — no Corsa binary required:

```rust
use corsa::{
    lsp::{VirtualChange, VirtualDocument},
    runtime::block_on,
};
use lsp_types::{Position, Range};

fn main() -> Result<(), corsa::CorsaError> {
    block_on(async {
        let mut doc =
            VirtualDocument::untitled("/virtual/minimal.ts", "typescript", "const answer = 41;\n")?;
        doc.apply_changes(&[VirtualChange::splice(
            Range::new(Position::new(0, 15), Position::new(0, 17)),
            "42",
        )])?;
        Ok(())
    })
}
```

```bash
cargo run -p corsa --example minimal_start
```

The [Getting started guide](./docs/getting_started.md) covers the Node.js and
type-aware Oxlint entry points and how to run against the real pinned Corsa
binary.

## Documentation

Full guides live under [`docs/`](./docs/index.md):

| Guide                                                  | What it covers                                                  |
| ------------------------------------------------------ | --------------------------------------------------------------- |
| [Getting started](./docs/getting_started.md)           | First program in Rust, Node.js, and Oxlint                      |
| [Architecture charter](./docs/architecture_charter.md) | The ownership boundary and the rules that follow from it        |
| [Architecture](./docs/project_guide.md)                | Workspace shape, upstream policy, extension points, naming      |
| [Node.js binding](./docs/nodejs_binding.md)            | `@corsa-bind/napi` for Node, Deno, and Bun                      |
| [Language bindings](./docs/language_bindings.md)       | Native bindings for Elixir, C, C++, Go, Zig, C#, Swift, MoonBit |
| [Type-aware Oxlint](./docs/oxlint_guide.md)            | `corsa-oxlint` rule authoring and native rules                  |
| [Native rules](./docs/native_rules.md)                 | The type-aware Rust lint rules and their options                |
| [Performance](./docs/performance.md)                   | Benchmark entry points and measured numbers                     |
| [CI and local checks](./docs/ci_guide.md)              | Reproduce the GitHub checks locally                             |
| [Production readiness](./docs/production_readiness.md) | Runtime controls and release gates                              |
| [Support policy](./docs/support_policy.md)             | Supported platforms, bindings, and experimental scope           |

The generated documentation site starts at [`docs/index.md`](./docs/index.md)
and is built with `vp run -w docs_build`.

## Status

- **License:** MIT
- **Upstream:** `ref/corsa-upstream` is pinned by exact commit in
  [`corsa_ref.lock.toml`](./corsa_ref.lock.toml), with no local patching
- **Default transport:** `SyncMsgpackStdio` (msgpack-first)
- **Runtime:** custom in-house runtime, no `tokio`
- **Published packages:** [`@corsa-bind/napi`](./src/bindings/nodejs/corsa_node)
  and [`corsa-oxlint`](./src/bindings/nodejs/corsa_oxlint) (both expect a
  caller-provided Corsa executable)
- **Process isolation:** every checker query crosses a process boundary on
  purpose — crash, GC, ABI, and TypeScript-version isolation, plus timeouts,
  cancellation, restart, and memory kill
- **Scaling:** single-machine worker pools with project affinity, and
  repo-level sharding above that; the Raft-backed replication layer was removed
  in 2.0

Public APIs are still `0.x`, so treat compatibility as conservative. See
[Known limitations](./docs/support_policy.md) for the current experimental scope.

## Sponsors

CI/CD runners for this project are supported by [Blacksmith](https://www.blacksmith.sh/).
