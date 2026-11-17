# Davinci charter

**Goal.** Bring all front-end language processing into one toolchain. That
covers container formats (SFC), markup grammars (HTML templates, Pug, JSX/TSX,
TSRX), style languages (CSS, SCSS, Sass, Less, Stylus), JavaScript and
TypeScript, and framework semantics. Every tool is built on it: compiler,
linter, formatter, type checker, LSP, devtools, bundler integrations and
plugins. All of it must be very fast. In short, an LLVM/MLIR for the front end
that learns from what went wrong in GCC and LLVM.

This page holds the rules that follow from that goal. The machinery is
described elsewhere: the [architecture](./architecture.md) (levels, what we
take from MLIR and what we refuse), [prior art](./prior-art.md) and
[toolchains](./prior-art-toolchains.md). Whether the goal is met is decided by
the [completion gates](./completion-gates.md). Each rule below names the
failure it avoids and the gate or issue that enforces it.

## Rules

1. **Library first. Every tool uses the same levels.**
   _Avoids:_ GCC kept its front end a monolith and its IR deliberately
   unexported, so tools could not reuse it. Clang won on libclang, clangd,
   clang-format and clang-tidy because they all reused one front end.
   _Rule:_ the levels (L0–L4) are libraries. The compiler, linter, formatter,
   type checker and LSP use them as consumers, on equal terms. No tool has a
   private parser or a private semantic pass.
   _Enforced by:_ the `products.*` gates (#6847–#6851, #6874, #6876), and
   #6850/#6872 (the LSP never parses on its own).

2. **Keep every byte. Do not fold early.**
   _Avoids:_ lossy front ends and early constant folding in GCC. They produced
   poor diagnostics and left nothing to refactor from.
   _Rule:_ L1 is lossless (`render(tree) == source`) and total on malformed
   input, which becomes typed holes rather than errors that stop parsing.
   Authored spans pass through every level. No pass rewrites or folds before
   the pass designated for it.
   _Enforced by:_ the L1 fidelity verifier, `levels.markup`, `levels.container`
   and the source-map laws.

3. **One semantic artifact and one query API for every tool.**
   _Avoids:_ Clang's AST is not an IR. The static analyzer, clang-tidy and
   codegen each re-derive semantics, and ClangIR arrived about fifteen years
   late.
   _Rule:_ L2 is the one canonical semantic artifact. Every tool reads
   semantics through one L2 query API and never walks private side tables.
   _Enforced by:_ `levels.l2` (#6838), `levels.query` (#6871) and
   `levels.script` (#6844).

4. **Internals may move. The extension surface is versioned.**
   _Avoids:_ LLVM's C++ API changes from release to release and breaks every
   out-of-tree user.
   _Rule:_ level internals carry no stability promise. The only stable surface
   is the versioned plugin capability model and its append-only C ABI. TS/JS,
   native and WIT guests all target that one model.
   _Enforced by:_ #13074 and its conformance suite.

5. **Compile time is a gate from the first day.**
   _Avoids:_ LLVM noticed its compile-time creep only after it had built up.
   _Rule:_ per-level and per-product instruction ceilings, plus wall-time and
   RSS budgets, run in the merge queue and only ratchet down. A change that
   uses no plugins pays at most 1% for the plugin machinery. Performance
   numbers come from optimized builds and go in the PR.
   _Enforced by:_ [`plan/budgets.toml`](./plan/budgets.toml), the
   `level-instruction-counts` workflow, and the
   [performance guardrails](./architecture.md#performance-guardrails).

6. **Incremental and query-based from L1 up.**
   _Avoids:_ batch-only design, such as clangd's preamble workarounds,
   compared with the query models of rust-analyzer and Roslyn.
   _Rule:_ level artifacts are produced per block and per query, so they can
   be reused across edits. The CLI and the LSP share one project model.
   Resident processes have RSS and idle-CPU ceilings. The goal is query-based
   but lighter than rust-analyzer.
   _Enforced by:_ `products.lsp` (#6850, #6872) and `products.project` (#6874).

7. **Closed dialects. Each has one owner. Lowerings are total.**
   _Avoids:_ MLIR's dialect sprawl, unclear ownership and slow generic
   conversion.
   _Rule:_ ops are closed typed enums. Each dialect, input language and
   framework lives in one owner directory per level, and a gate checks that
   core modules do not reference it. Lowerings are one-shot and total.
   Input-specific code ends at L2: L3 and L4 read only neutral IR, and one
   input-neutral module description carries SFC, JSX/TSX and TSRX to every
   target.
   _Enforced by:_ `levels.dialects` (#6841, #6842), `levels.l3` (#6839) and
   `levels.l4` (#6840), together with the input-placement gate that their
   layout decision adds.

8. **Extension without runtime dialect registries.**
   _The tension:_ the architecture refuses dynamic dialect registries and
   per-node dynamic dispatch. Third parties still have to extend the toolchain
   with custom blocks, directives, macros and lint rules.
   _Rule:_ plugins contribute declaratively. Their contributions are resolved
   once, at plan time, into the native tables that the levels already
   dispatch on per file. They are never consulted per node through a dynamic
   registry. Code that does need to run per node goes through the versioned
   ABI (rule 4), and its cost is budgeted (rule 5).
   _Enforced by:_ #13074 (the canonical plugin plan) and the performance
   guardrails.

## When rules conflict

Performance and correctness come first, as the
[priority order](./architecture.md#priority-order-charter-22) says. Rule 8
does not license a slower core. A plugin feature that needs dynamic per-node
dispatch is refused or redesigned. The core never pays for it.
