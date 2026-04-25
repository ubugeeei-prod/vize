---
title: Performance
---

# Performance

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. Benchmark numbers are from development builds and may change.

Vize achieves significant performance improvements over the standard JavaScript-based Vue compiler by leveraging Rust's zero-cost abstractions and native multi-threading. Speed is not a nice-to-have — it is a prerequisite for developer experience.

## Benchmark Environment

|             |                                           |
| ----------- | ----------------------------------------- |
| **Machine** | MacBook Pro (M2 Max, 12 cores, 96 GB RAM) |
| **OS**      | macOS 15.3.2 (Darwin 24.3.0)              |
| **Node.js** | v24.14.0                                  |
| **Vite**    | v8.0.0 (Rolldown)                         |
| **Vue**     | v3.6.0-beta.10                            |

## Benchmark: 15,000 SFC Files

Compiling **15,000 Vue SFC files** (36.9 MB total):

|                                | @vue/compiler-sfc | Vize  | Speedup   |
| ------------------------------ | ----------------- | ----- | --------- |
| **Single Thread**              | 9.35s             | 3.47s | **2.7x**  |
| **Multi Thread**               | 4.08s             | 353ms | **11.6x** |
| **compiler-sfc ST vs Vize MT** | 9.35s             | 353ms | **26.0x** |

The single-threaded improvement comes from Rust's zero-cost abstractions (no GC, no JIT warmup, cache-friendly memory layout). The multi-threaded improvement comes from Rayon's work-stealing thread pool, which scales near-linearly with CPU core count.

### Native Batch Scaling Behavior

| Files  | Vize batch (1 thread) | Vize batch (12 threads) | Parallel Speedup |
| ------ | --------------------- | ----------------------- | ---------------- |
| 100    | 25ms                  | 3ms                     | 8.5x             |
| 1,000  | 243ms                 | 26ms                    | 9.4x             |
| 5,000  | 1.25s                 | 128ms                   | 9.7x             |
| 15,000 | 3.75s                 | 373ms                   | 10.1x            |

These native batch numbers include file reads. Small batches are dominated by fixed overhead; larger batches settle around 10x parallel speedup on this 12-core machine.

## Why Rust?

### Zero-Cost Abstractions

Rust's ownership model eliminates garbage collection pauses. The compiler processes AST nodes through arena allocation (`vize_carton`), avoiding per-node heap allocations. This means:

- **No GC pauses** — In V8-based compilers, garbage collection can cause unpredictable latency spikes. Vize has zero GC overhead.
- **No JIT warmup** — V8's JIT compiler needs time to optimize hot paths. Vize runs at full speed from the first instruction.
- **Predictable performance** — Rust's ahead-of-time compilation means performance is consistent across runs, not dependent on V8's optimization heuristics.

### Native Multi-Threading

Vize uses [Rayon](https://docs.rs/rayon) for data-parallel compilation. Each SFC file is compiled independently, making the workload embarrassingly parallel. Rayon's work-stealing scheduler ensures optimal core utilization:

```rust
// Simplified: parallel compilation of all .vue files
files.par_iter().map(|file| {
    let arena = Bump::new();
    let ast = parse(file, &arena);
    let analyzed = analyze(ast, &arena);
    compile(analyzed, &arena)
}).collect()
```

The work-stealing approach means that if one file is significantly larger than others, idle threads will steal work from the busy thread's queue, maintaining near-perfect load balancing.

### Efficient Memory Layout

Rust's struct layout and enum discriminants are compact. The AST representation in `vize_relief` is cache-friendly, reducing memory bandwidth bottlenecks:

- **Enum discriminants** — Rust enums are sized to the smallest type that fits the discriminant. A `NodeKind` with 20 variants uses a single byte, not a heap-allocated string.
- **Struct packing** — Rust automatically reorders struct fields for optimal alignment, minimizing padding bytes.
- **No object headers** — Unlike JavaScript objects (which carry prototype chains, property maps, and hidden class pointers), Rust structs are pure data with zero overhead.

### No Runtime Overhead

Unlike JavaScript-based compilers that run in V8, Vize compiles directly to native code. There's no JIT warmup, no garbage collector, and no event loop contention. The compiler binary is a single, statically-linked executable that starts and runs at full speed.

## Architecture Choices for Performance

### Arena Allocation

`vize_carton` provides a bump allocator for AST nodes using [bumpalo](https://docs.rs/bumpalo). This means:

- **Allocation is O(1)** — Just bump a pointer forward. No free list traversal, no fragmentation management.
- **Deallocation is O(1)** — Drop the entire arena at once when compilation is complete. No per-node deallocation overhead.
- **Memory locality is excellent** — Nodes are packed contiguously in memory, maximizing L1/L2 cache hits during tree traversal.

This is a fundamental advantage over V8's generational garbage collector, which must trace reachable objects and compact memory periodically.

### Streaming Tokenizer

`vize_armature`'s tokenizer processes input as a stream of bytes, avoiding the need to build intermediate token arrays. The parser consumes tokens lazily — each token is produced on demand and immediately consumed. This reduces peak memory usage and improves cache behavior.

### String Interning

Common strings (directive names, attribute names, HTML tag names) are interned via `compact_str` and perfect hash tables (`phf`). This means:

- String comparison is pointer comparison (O(1)) instead of character-by-character comparison (O(n))
- Duplicate strings share a single allocation
- Hash lookups for known strings are compile-time computed

### Incremental Compilation

The Vite plugin (`@vizejs/vite-plugin`) uses file-level caching. Only modified files are recompiled during development, minimizing HMR latency. The cache key is the file content hash, ensuring that unchanged files are never recompiled.

## Benchmark: Linter — patina vs eslint-plugin-vue

Linting **15,000 Vue SFC files**:

|          | eslint-plugin-vue (ST) | Vize patina (ST) | Speedup   | eslint-plugin-vue (MT) | Vize patina (MT) | Speedup   | **eslint ST vs Vize MT** |
| -------- | ---------------------- | ---------------- | --------- | ---------------------- | ---------------- | --------- | ------------------------ |
| **Time** | 45.08s                 | 4.02s            | **11.2x** | 16.38s                 | 784ms            | **20.9x** | **57.5x**                |

Run `vp run --workspace-root bench:lint` to reproduce.

## Benchmark: Formatter — glyph vs Prettier

Formatting **15,000 Vue SFC files**:

|          | Prettier (CLI) | Vize glyph (ST) | Speedup   | Vize glyph (MT) | **Prettier CLI vs Vize MT** |
| -------- | -------------- | --------------- | --------- | --------------- | --------------------------- |
| **Time** | 101.20s        | 2.97s           | **34.1x** | 835ms           | **121.2x**                  |

Run `vp run --workspace-root bench:fmt` to reproduce.

## Benchmark: Type Checker — canon vs vue-tsc

Type checking **500 generated Vue SFC files** with the current Corsa-backed diagnostics path:

|          | vue-tsc (ST) | Vize canon (ST) | Speedup  | vue-tsc (MT) | Vize canon (MT) | Speedup  | **vue-tsc ST vs Vize MT** |
| -------- | ------------ | --------------- | -------- | ------------ | --------------- | -------- | ------------------------- |
| **Time** | 4.38s        | 511ms           | **8.6x** | 4.41s        | 493ms           | **8.9x** | **8.9x**                  |
| **Rate** | 114 files/s  | 979 files/s     |          | 113 files/s  | 1.0k files/s    |          |                           |

> **Note:** Vize canon is still in early development and the Corsa-backed diagnostics path is still catching up with vue-tsc fidelity. These measurements reflect the current CLI-first native implementation with a project-session fallback and will change as diagnostics coverage and parity improve.

Run `node bench/check.ts 500` after `cargo build --release -p vize` to reproduce this quick benchmark.

### Type checker profile

The 500-SFC profile fixture keeps most wall time inside the Corsa CLI command, while the import rewrite fast path removes the previous OXC parse cost for files without Vue specifiers:

| Metric                       | Before  | Current |
| ---------------------------- | ------- | ------- |
| `canon.import.rewrite.vue`   | 26.77ms | 2.45ms  |
| Largest generated Virtual TS | 15,401B | 14,414B |
| Total profile wall time      | 1.88s   | 668ms   |
| Corsa diagnostics phase      | 1.67s   | 482ms   |
| Corsa CLI parse              | n/a     | 10.41ms |

### Diagnostics-heavy e2e fixture

`bench/check.ts` also measures the `tests/_fixtures/_git/npmx.dev` app when the fixture is present. This catches the diagnostics mapping path on a real application fixture:

| Fixture      | Source SFC files | Virtual files | Diagnostics | Vize canon |
| ------------ | ---------------- | ------------- | ----------- | ---------- |
| npmx.dev app | 134              | 226           | 1,053       | 1.94s      |

The current profile for this fixture keeps CLI diagnostic parsing at ~7ms. Most time is now in the Corsa CLI command itself. Hoisting framework auto-import stubs into one ambient file also reduced the largest generated Virtual TS file from about 275KB to 144KB.

## Benchmark: Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue

Vite build with **1,000 Vue SFC imports** (all imported in a single entry):

|                | @vitejs/plugin-vue | @vizejs/vite-plugin | Speedup  |
| -------------- | ------------------ | ------------------- | -------- |
| **Build Time** | 957ms              | 479ms               | **2.0x** |

> Note: `@vizejs/vite-plugin` replaces only the Vue SFC compilation step — the performance difference comes entirely from that part. Dependency resolution, module graph construction, bundling (Rolldown), and all other Vite internals are identical to `@vitejs/plugin-vue`. For pure compilation performance, see the [Compiler benchmark](#benchmark-15000-sfc-files) above. `@vizejs/vite-plugin` eagerly pre-compiles `.vue` files using native multi-threaded compilation, which also enables faster HMR.

Run `vp run --workspace-root bench:vite` to reproduce.
