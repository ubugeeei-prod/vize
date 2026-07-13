# vize_flow

Compatibility follows the [Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_flow` is Vize's frontend-neutral, single-compilation-unit flow IR. It
models basic blocks, operations, control transfer, values, symbols, and effects
without retaining a Relief tree, a Croquis model, an OXC AST, JSX nodes, or SFC
nodes.

The crate owns three related graphs:

- control-flow edges describe branches, loops, returns, and exceptions;
- data-flow edges describe definitions, uses, phi inputs, mutations, and
  captures;
- effect edges describe ordering, dependencies, and conflicts between reads,
  writes, calls, allocation, suspension, and other observable operations.

All graph entities use dense, type-safe IDs and carry either a source span or
explicit synthetic provenance. Checked construction keeps block adjacency,
value definitions, and effect endpoints internally consistent. Reachability and
dominance are reusable analyses over the resulting graph.

This is not `vize_croquis_cf`. Flow represents execution and value movement
inside one compilation unit. `vize_croquis_cf` owns opt-in cross-file semantic
aggregation, module/component relationships, and project-level analysis. A
cross-file pass may consume facts derived from Flow, but it does not turn Flow
into a project graph.

Frontends are producers and tools are consumers. SFC, JSX, Vapor, legacy, and
future frontends can each lower directly into this representation, while a
compiler, linter, or type checker can request the same product without choosing
one frontend's AST as the common foundation.
