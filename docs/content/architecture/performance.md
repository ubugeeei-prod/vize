---
title: Performance
---

# Performance

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. Benchmark numbers are from development builds and may change.

Vize achieves significant performance improvements over the standard JavaScript-based Vue compiler by leveraging Rust's zero-cost abstractions and native multi-threading. Speed is not a nice-to-have — it is a prerequisite for developer experience.

<!-- benchmark:environment:start -->

## Benchmark Environment

One committed artifact supplies the README and every locale. File counts are stated per row.

| Runner                                   | Measured                 | Runs | Warmups |
| ---------------------------------------- | ------------------------ | ---- | ------- |
| `blacksmith-32vcpu-ubuntu-2404` (32 CPU) | 2026-09-16T08:19:25.262Z | 3    | 1       |

[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669) · [f9cc50ee2e2f](https://github.com/ubugeeei-prod/vize/commit/f9cc50ee2e2ff14cd120241cd87a0162446df65b) · [JSON](https://github.com/ubugeeei-prod/vize/blob/main/tools/benchmarks/results/tool-benchmark-latest.json)

Versions: vize: `vize 0.424.11` · tsgo: `Version 7.0.2` · vueTsc: `Version 6.0.3` · verterTsc: `verter-tsc 0.0.1-beta.3` · vue: `3.6.0-beta.10` · node: `v24.14.0`.

The local linter, formatter and implementation profiles below are historical measurements, not results from this snapshot. Their recorded workstation is a MacBook Pro M2 Max (12 cores, 96 GB RAM), macOS 15.3.2, Node v24.14.0, Vite v8.0.0 and Vue v3.6.0-beta.10.

## Benchmark: SFC Compile

Files: **3,000** (12,302,100 bytes).

|               | @vue/compiler-sfc | Vize    | Speedup   |
| ------------- | ----------------- | ------- | --------- |
| 1T            | 3.57s             | 835.3ms | **4.3x**  |
| workers / max | 1.61s             | 65.0ms  | **24.7x** |
| 1T / max      | 3.57s             | 65.0ms  | **54.8x** |

[Full methodology and evidence](./performance-blacksmith)

<!-- benchmark:environment:end -->

## Why Rust?

### Zero-Cost Abstractions

Rust's ownership model eliminates garbage collection pauses. Template AST nodes live in a
per-compile arena (`vize_carton`) and borrow their text from the template source, so a node is
plain data with no owned heap allocations of its own (`crates/vize_relief/src/relief/elements.rs`).
This means:

- **No GC pauses** — In V8-based compilers, garbage collection can cause unpredictable latency spikes. Vize has zero GC overhead.
- **No JIT warmup** — V8's JIT compiler needs time to optimize hot paths. Vize runs at full speed from the first instruction.
- **Predictable performance** — Rust's ahead-of-time compilation means performance is consistent across runs, not dependent on V8's optimization heuristics.

### Native Multi-Threading

Vize uses [Rayon](https://docs.rs/rayon) for data-parallel compilation. Each SFC file is compiled
independently, making the workload embarrassingly parallel, and the batch driver in
`crates/vize/src/commands/build/runner.rs` fans the planned inputs across the pool:

```rust
// crates/vize/src/commands/build/runner.rs — the batch driver
planned_inputs
    .par_iter()
    .map(|input| compile_file_with_profile(&input.source, compile_settings, &stats))
    .collect()
```

The arena is not created here. It is acquired where it is born — at the template, script, and style
entry points inside `vize_atelier_sfc` — from a per-worker pool:

```rust
// e.g. crates/vize_atelier_sfc/src/compile.rs
let allocator = vize_carton::pool::acquire();
```

The work-stealing approach means that if one file is significantly larger than others, idle threads will steal work from the busy thread's queue, maintaining near-perfect load balancing.

### Efficient Memory Layout

Rust's struct layout and enum discriminants are compact. The AST representation in `vize_relief` is cache-friendly, reducing memory bandwidth bottlenecks:

- **One-byte discriminants** — `NodeType` is `#[repr(u8)]` with 27 variants
  (`crates/vize_relief/src/relief/core.rs`), so a node's kind costs a byte, not a heap-allocated
  string.
- **Pinned node sizes** — every template node carries a `const` size assertion, so a field that
  grows a node fails the build rather than the budget. `ElementNode` is 104 bytes,
  `SimpleExpressionNode` 88, `AttributeNode` 56, `TextNode` 24, `SourceLocation` 8
  (`crates/vize_relief/src/relief/{elements,expressions,control_flow,nodes}.rs`).
- **No object headers** — Unlike JavaScript objects (which carry prototype chains, property maps, and hidden class pointers), Rust structs are pure data with zero overhead.

### No Runtime Overhead

Unlike JavaScript-based compilers that run in V8, Vize compiles directly to native code. There's no
JIT warmup, no garbage collector, and no event loop contention. The CLI ships as a self-contained
native executable per platform — fully static on the musl Linux targets, which CI verifies
(`tools/commands/ci/github/verify-musl-cli-binary.rs`), and dynamically linked against the system C library on
the glibc, macOS, and Windows targets. The Vite plugin loads the same compiler as a native Node
addon (`@vizejs/native`) rather than as a separate process.

## Architecture Choices for Performance

### Arena Allocation

`vize_carton::Allocator` is a bump allocator for AST nodes, wrapping
[`oxc_allocator`](https://docs.rs/oxc_allocator) so template nodes and retained JavaScript
expressions share one arena and one lifetime (`crates/vize_carton/src/allocator.rs`). This means:

- **Allocation is O(1)** — Just bump a pointer forward. No free list traversal, no fragmentation management.
- **Reclamation is O(1) and reused** — At the end of a compile the arena is `reset()`, not dropped:
  the bump pointer returns to the start of the chunk and the arena goes back to a per-worker free
  list (`crates/vize_carton/src/pool.rs`, capped at 4 idle arenas per worker). The next file reuses
  the same memory instead of asking the OS for more.
- **Memory locality is excellent** — Nodes are packed contiguously in memory, maximizing L1/L2 cache hits during tree traversal.

Arena-backed values may not outlive their compile. That contract is enforced by the compiler
(`reset` takes `&mut self`, and the pool guard owns its arena) and, in debug builds, by a
generation stamp that panics if a value is read after its arena was recycled
(`crates/vize_carton/src/allocator/generation.rs`).

Nothing in the AST implements `Drop` — the arena container types reject payloads that need
dropping, so this is a compile error rather than a convention.

### Single-Pass Tokenizer

`vize_armature`'s tokenizer is a byte-oriented state machine over `&[u8]`
(`crates/vize_armature/src/tokenizer.rs`). It never materializes a token: there is no `Token` type
and no token vector anywhere in the compiler. Instead, `tokenize()` runs one pass to end of input
and pushes events to a `Callbacks` sink, which the parser implements — so each event is handled
synchronously as it is produced, and the intermediate array a two-phase design would need never
exists.

Note that this is push-based, not a lazy pull: the parser does not request tokens, and it cannot
stop the loop partway.

### String Interning

Names that recur within a compile — normalized directive names, asset names, camelized argument
names — are interned into arena-backed atoms by `vize_carton::interner`, with a compile-time
[`phf`](https://docs.rs/phf) set of 181 well-known names (HTML/SVG/MathML tags, Vue built-in
components, directive names, and the attributes the transforms special-case) resolving to `'static`
literals without touching the arena at all. This means:

- Repeated computed names share a single arena allocation
- Lookups for well-known names are a compile-time perfect hash, with no allocation

Interning is the fallback, not the common case. Most names are never copied at all: a tag name, an
attribute name, and most expression content are `&'a str` slices borrowed directly from the
template source, so the common path allocates nothing (`crates/vize_carton/src/interner.rs`
documents the per-field policy).

Atoms are ordinary `&'a str`, so name comparisons are content comparisons, not pointer identity.
Interning buys allocation savings and cache locality — it is not a fast-path for `==`.

### Incremental Compilation

The Vite plugin (`@vizejs/vite-plugin`) caches at file level, in two layers with different keys:

- **In-memory, for dev and HMR** — keyed by resolved file path
  (`npm/builder/vite/src/plugin/compiled-module-cache.ts`). Entries are evicted explicitly on hot
  update rather than re-keyed, so a changed file is recompiled and its neighbours are not.
- **Pre-compile change detection** — keyed by `mtime` + size, compared in Rust
  (`crates/vize_atelier_sfc/src/vite_plugin/precompile.rs`). This is the gate that decides which
  files a batch re-compiles.
- **On disk, across processes** — `node_modules/.vize/vite-precompile`, keyed by a SHA-256 hash of
  the source plus a manifest key covering the compiler binary's identity and the resolved options
  (`npm/builder/vite/src/plugin/precompile-cache-key.ts`). Content hashing is used here precisely
  because `mtime` is not trustworthy across machines and checkouts.

## Measured: Arena and Expression Work

The compiler-internals work described above is measured by a per-crate microbench harness
(`cargo bench --bench davinci`) over a fixed six-fixture ladder,
`tools/benchmarks/crates/davinci_harness/fixtures/{small,medium,large,stress-deep,stress-wide,stress-interp}.vue`.

**How to read these numbers.** Allocation counts are deterministic and machine-independent, so they
are exact facts and are used as the regression ratchet. Wall times were taken on a shared
development machine with `--quick` sampling and are **directional only** — the reference-runner
(Blacksmith) recordings are still pending, which is why every `wall_p50_ns` and `allocs` entry in
`davinci-road/plan/budgets.toml` is still `0`, meaning "not yet recorded, report-only". Per-run
result files land in `tools/benchmarks/results/davinci/` and are local artifacts, not committed baselines.

Allocation calls per compile, before and after the string-and-arena work (exact, same fixtures):

| Fixture         | Parse     | DOM compile | SSR compile   | Vapor compile |
| --------------- | --------- | ----------- | ------------- | ------------- |
| `small`         | 21 → 9    | 52 → 39     | 73 → 60       | 90 → 73       |
| `medium`        | 171 → 107 | 329 → 264   | 1,099 → 1,030 | 588 → 515     |
| `large`         | 350 → 272 | 656 → 573   | 1,106 → 983   | 1,136 → 1,003 |
| `stress-deep`   | 397 → 155 | 669 → 426   | 612 → 369     | 764 → 514     |
| `stress-wide`   | 213 → 204 | 255 → 245   | 416 → 405     | 280 → 261     |
| `stress-interp` | 616 → 105 | 1,048 → 536 | 3,149 → 2,637 | 1,495 → 974   |

Node sizes shrank with them, and the new sizes are pinned by `const` assertions: `RootNode`
296 → 224 bytes, `DirectiveNode` 208 → 176, `ElementNode` 128 → 104, `SimpleExpressionNode`
120 → 88, `AttributeNode` 80 → 56, `TextNode` 32 → 24.

**Peak resident memory.** Arena reuse across files is the largest single win, and it is a memory
result rather than a speed one. Compiling all 36,541 committed corpus SFCs
(`vize build "tests/_fixtures/_git/**/*.vue" --format stats`, `ci-opt` binaries, maximum resident
set size from `/usr/bin/time -l`, same machine before and after):

| Workers | Before   | After    | Change     | Runs each |
| ------- | -------- | -------- | ---------- | --------- |
| 12      | 766.5 MB | 171.1 MB | **−77.7%** | 5         |
| 1       | 717.0 MB | 88.2 MB  | **−87.7%** | 3         |

The single-worker figure is the accumulation signal: it is scheduling-free, so it shows the old
peak was per-file leakage rather than per-worker arenas. Wall time was unchanged within noise, and
all 36,541 emitted files were byte-identical (SHA-256 manifests compared).

**Expression re-parsing.** Template expressions are now parsed once, during template parse, and
retained on the node. Consumers read the retained AST instead of re-parsing the text. On the SSR
lane the `stress-interp` fixture went from 500 redundant expression re-parses per compile to zero,
and that fused lane is a net **−13.6%** wall against the pre-retention tree (346.8µs → 299.8µs) —
the parse now costs more and the consumers cost much less. The DOM and Vapor lanes had no
re-parses to delete on that fixture, so they still carry the added parse cost; closing that is
tracked as remaining phase work, not a shipped win.

## Benchmark: Linter — patina vs eslint-plugin-vue

Linting **15,000 Vue SFC files**, local workstation:

|          | eslint-plugin-vue (ST) | Vize patina (ST) | Speedup   | eslint-plugin-vue (MT) | Vize patina (MT) | Speedup   | **eslint ST vs Vize MT** |
| -------- | ---------------------- | ---------------- | --------- | ---------------------- | ---------------- | --------- | ------------------------ |
| **Time** | 45.08s                 | 4.02s            | **11.2x** | 16.38s                 | 784ms            | **20.9x** | **57.5x**                |

Run `vp run --workspace-root bench:lint` to reproduce.

### Type-aware lint profile

Type-aware linting is intentionally profiled at the phases where cost tends to cluster: SFC parsing,
Croquis analysis, virtual TypeScript generation, template query collection, and Corsa probes. When
multiple template-backed type-aware rules are enabled, Patina collects template expression and
template Promise queries in one AST walk before the Corsa probe phase. Query collection also shares
the OXC expression parse for unsafe-template and floating-Promise checks, so one template expression
does not pay duplicate parse cost when both rules are enabled.

Run `vize lint --profile --preset opinionated src` to see these rows in a local project. The
profile report also includes a strict audit section that checks wall-time coverage, cumulative
worker time, slow-threshold hits, and captured internal spans before listing hot files and internal
operations. Hot-file rows show per-stage share and throughput, and operation rows flag dominant
spans or max/avg spikes.

## Benchmark: Formatter — glyph vs Prettier

Formatting **15,000 Vue SFC files**, local workstation:

|          | Prettier (CLI) | Vize glyph (ST) | Speedup   | Vize glyph (MT) | **Prettier CLI vs Vize MT** |
| -------- | -------------- | --------------- | --------- | --------------- | --------------------------- |
| **Time** | 101.20s        | 2.97s           | **34.1x** | 835ms           | **121.2x**                  |

Run `vp run --workspace-root bench:fmt` to reproduce.

<!-- benchmark:typecheck:start -->

## Benchmark: Type Checker

One committed artifact supplies the README and every locale. File counts are stated per row.

| Runner                                   | Measured                 | Runs | Warmups |
| ---------------------------------------- | ------------------------ | ---- | ------- |
| `blacksmith-32vcpu-ubuntu-2404` (32 CPU) | 2026-09-16T08:19:25.262Z | 3    | 1       |

[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669) · [f9cc50ee2e2f](https://github.com/ubugeeei-prod/vize/commit/f9cc50ee2e2ff14cd120241cd87a0162446df65b) · [JSON](https://github.com/ubugeeei-prod/vize/blob/main/tools/benchmarks/results/tool-benchmark-latest.json)

Versions: vize: `vize 0.424.11` · tsgo: `Version 7.0.2` · vueTsc: `Version 6.0.3` · verterTsc: `verter-tsc 0.0.1-beta.3` · vue: `3.6.0-beta.10` · node: `v24.14.0`.

| Surface              | Files | Existing tool | Existing median | Vize 1T | Vize max | Speedup  |
| -------------------- | ----- | ------------- | --------------- | ------- | -------- | -------- |
| Type check           | 500   | verter-tsc    | 2.08s           | 2.03s   | 1.37s    | **1.5x** |
| Large SFC type check | 1     | verter-tsc    | 1.58s           | 186.4ms | 190.1ms  | **8.3x** |

Type-check ratios compare Vize with verter-tsc using the same pinned native tsgo backend. Each timed invocation passes diagnostic-work validation. Diagnostic coverage differs between tools; these ratios do not prove accuracy parity or a performance improvement over an earlier Vize version. Rejected comparators have no published timing or rank.

[Full methodology and evidence](./performance-blacksmith)

The profiles below are historical local measurements and are not part of this reference-runner comparison.

<!-- benchmark:typecheck:end -->

### Type checker profile

The 500-SFC profile fixture keeps most wall time inside the Corsa CLI command, while the import rewrite fast path removes the previous OXC parse cost for files without Vue specifiers:

| Metric                       | Before  | Current |
| ---------------------------- | ------- | ------- |
| `canon.import.rewrite.vue`   | 26.77ms | 2.45ms  |
| Largest generated Virtual TS | 15,401B | 14,414B |
| Total profile wall time      | 1.88s   | 668ms   |
| Corsa diagnostics phase      | 1.67s   | 482ms   |
| Corsa CLI parse              | n/a     | 10.41ms |

The Rust-side `virtual project` phase — per-file SFC parse, Croquis analysis,
Virtual TS generation, and import rewriting — is fanned across rayon's thread
pool inside `VirtualProject::register_paths`. Each `.vue` file is independent
once the workspace options are resolved, so a single batch parallelizes
cleanly. On a 1,000-SFC fixture the phase drops from ~71 ms to ~25 ms before
Corsa is even invoked.

### Diagnostics-heavy e2e fixture

`tools/benchmarks/scripts/check.ts` also measures the `tests/_fixtures/_git/npmx.dev` app when the fixture is present. This catches the diagnostics mapping path on a real application fixture:

| Fixture      | Source SFC files | Virtual files | Diagnostics | Vize canon |
| ------------ | ---------------- | ------------- | ----------- | ---------- |
| npmx.dev app | 134              | 226           | 1,053       | 1.94s      |

The current profile for this fixture keeps CLI diagnostic parsing at ~7ms. Most time is now in the Corsa CLI command itself. Hoisting framework auto-import stubs into one ambient file also reduced the largest generated Virtual TS file from about 275KB to 144KB.

<!-- benchmark:vite:start -->

## Benchmark: Vite Plugin

One committed artifact supplies the README and every locale. File counts are stated per row.

| Runner                                   | Measured                 | Runs | Warmups |
| ---------------------------------------- | ------------------------ | ---- | ------- |
| `blacksmith-32vcpu-ubuntu-2404` (32 CPU) | 2026-09-16T08:19:25.262Z | 3    | 1       |

[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669) · [f9cc50ee2e2f](https://github.com/ubugeeei-prod/vize/commit/f9cc50ee2e2ff14cd120241cd87a0162446df65b) · [JSON](https://github.com/ubugeeei-prod/vize/blob/main/tools/benchmarks/results/tool-benchmark-latest.json)

Versions: vize: `vize 0.424.11` · tsgo: `Version 7.0.2` · vueTsc: `Version 6.0.3` · verterTsc: `verter-tsc 0.0.1-beta.3` · vue: `3.6.0-beta.10` · node: `v24.14.0`.

Files: **1,000**.

|            | @vitejs/plugin-vue | @vizejs/vite-plugin | Speedup  |
| ---------- | ------------------ | ------------------- | -------- |
| Vite build | 1.66s              | 889.2ms             | **1.9x** |

[Full methodology and evidence](./performance-blacksmith)

<!-- benchmark:vite:end -->

The figure published here until then — `957ms` / `479ms` / `2.0x` — came from `tools/benchmarks/scripts/vite.ts` before #3392, which timed Vize with a warm persistent pre-compile cache left behind by its own warmup while `@vitejs/plugin-vue` compiled from scratch. That harness now reports separate cold and warm rows on the machine it runs on, so it produces a local diagnostic, not a publishable speedup; use `vp run --workspace-root bench:vite` to compare a change against itself, not to source a number for this page.
