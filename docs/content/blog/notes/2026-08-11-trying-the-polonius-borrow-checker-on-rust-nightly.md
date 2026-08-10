---
title: Polonius on Nightly
description: Rust's next-generation borrow checker is now on by default on nightly. We ran the whole Vize workspace through it, measured what it costs, and audited every borrow-checker workaround in the codebase.
---

# Polonius on Nightly

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-08-11</span>
    </span>
  </span>
  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">Author</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

On August 4, 2026 the Rust project [enabled the next iteration of the borrow checker on nightly](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/). Starting with `nightly-2026-08-06`, the borrow checker known as Polonius Alpha is the default, with stabilization targeted before the end of the year and feedback collected on the [tracking issue](https://github.com/rust-lang/rust/issues/160456).

Vize is a 23-crate Rust workspace pinned to stable 1.95.0, with 411 locked packages in the dependency graph. That makes it a reasonable real-world subject for three questions:

- Does anything in the workspace behave differently under the new checker?
- What does the new checker cost in build time?
- How much of the code we wrote _around_ the old borrow checker can now be deleted?

This note answers all three. The short version: nothing breaks, it costs about 5% of check CPU time, and the amount of code Polonius lets us delete today is smaller than we expected — but not zero.

## What Polonius Changes

The current borrow checker (non-lexical lifetimes, NLL) rejects some programs that are actually sound. The best-known shape is a borrow returned from one branch while a later statement needs the same collection again — NLL "Problem Case #3":

```rust
use std::collections::HashMap;

fn get_or_insert(map: &mut HashMap<u32, String>) -> &String {
    if let Some(v) = map.get(&22) {
        return v;
    }
    map.insert(22, String::from("hi"));
    &map[&22]
}
```

The borrow of `map` in the `if let` only escapes through the early `return`, so the `map.insert` below is fine — but NLL extends the borrow to the whole function body and rejects it:

```text
error[E0502]: cannot borrow `*map` as mutable because it is also borrowed as immutable
 --> demo.rs:8:5
  |
5 |     if let Some(v) = map.get(&22) {
  |                      --- immutable borrow occurs here
6 |         return v;
  |                - returning this value requires that `*map` is borrowed for `'1`
7 |     }
8 |     map.insert(22, String::from("hi"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

Polonius makes the analysis flow-sensitive, so the borrow ends where it actually ends. Here is the same file across the configurations we tested, all on `rustc 1.99.0-nightly (2026-08-09)` except the stable row:

| Configuration                | Result                                          |
| ---------------------------- | ----------------------------------------------- |
| stable 1.95.0                | rejected (E0502)                                |
| nightly, default             | **accepted** — Polonius Alpha is on by default  |
| nightly, `-Zpolonius=no`     | rejected — the old NLL checker                  |
| nightly, `-Zpolonius=legacy` | accepted — the original datalog prototype       |
| nightly, `-Zpolonius=next`   | accepted — the full location-sensitive analysis |

The alpha that is heading for stabilization is a restricted formulation of the full analysis: flow-sensitive on lifetime outlives relationships, chosen because it covers the common false rejections at an acceptable compile-time cost. `-Zpolonius=next` is the complete location-sensitive implementation that is still under development, and `-Zpolonius=no` is the opt-out the Rust team suggests if the new default causes trouble.

## Running the Whole Workspace Through It

The interesting question for a codebase this size is not the demo — it is whether anything anywhere in 23 crates changes meaning.

```bash
cargo +nightly check --workspace                                  # Polonius Alpha (default)
RUSTFLAGS="-Zpolonius=no"   cargo +nightly check --workspace      # old NLL
RUSTFLAGS="-Zpolonius=next" cargo +nightly check --workspace      # full analysis
```

All three configurations check the entire workspace — every Vize crate plus every dependency — with byte-identical diagnostics: zero warnings, zero errors, in every configuration. Nothing in Vize or its dependency graph is affected by the new checker, under either the alpha or the stricter full implementation.

That is the boring result, and boring is what you want here: when Polonius Alpha stabilizes and Vize's pinned toolchain eventually moves past it, the bump will be a non-event as far as borrow checking is concerned.

## The Build-Time Cost

The Rust team's own measurements say most crates see minimal impact, with the worst known cases (outside the top 10,000 crates) regressing 2–3x. We measured what it costs Vize.

Our first attempt measured wall-clock time across eight interleaved rounds and produced garbage: ratios against the NLL baseline ranged from 0.66x to 1.97x, and in three of the eight rounds the _slower_ checker finished first. On a laptop with background load, wall clock is measuring the machine, not the compiler. So the numbers below are CPU time (`user + sys`), which is far less sensitive to contention, over three clean `cargo check --workspace` runs per configuration with a fresh `CARGO_TARGET_DIR` each time.

Machine: Apple M2 Max (12 cores, 96 GB), `rustc 1.99.0-nightly (969b803cb 2026-08-09)`.

| Configuration                     | Median CPU time | vs. old NLL |
| --------------------------------- | --------------- | ----------- |
| `-Zpolonius=no` (old NLL)         | 194.4s          | —           |
| default (Polonius Alpha)          | 204.2s          | **1.05x**   |
| `-Zpolonius=next` (full analysis) | 198.6s          | 1.02x       |

About 5% more CPU to check the entire workspace. That is comfortably inside the range the Rust team called reasonable, and nowhere near the 2–3x worst case. Interestingly the full `next` analysis measured _cheaper_ than the shipping alpha here; with three samples per configuration that gap is inside the noise, and the honest reading is that alpha and next cost about the same on this codebase.

Note also that this is worst-case exposure: a clean check of all 411 packages. Incremental flows recompile a handful of crates, and full builds bury the difference under codegen.

## Where the 5% Actually Goes

A workspace-level number tells you what to budget for but not what happened. `-Ztime-passes` reports the borrow-check pass on its own, so we measured it directly on the three largest crates, with `CARGO_INCREMENTAL=0` and a forced rebuild before each sample so no run could be served from the incremental cache. Five samples per crate per configuration, medians below.

| Crate                      | old NLL   | Polonius Alpha    | `-Zpolonius=next` |
| -------------------------- | --------- | ----------------- | ----------------- |
| `vize_patina` (90k lines)  | 0.622s    | 0.749s (1.20x)    | 0.674s (1.08x)    |
| `vize_canon` (75k lines)   | 0.729s    | 0.876s (1.20x)    | 0.798s (1.09x)    |
| `vize_croquis` (36k lines) | 0.443s    | 0.468s (1.06x)    | 0.472s (1.07x)    |
| **total**                  | **1.79s** | **2.09s (1.17x)** | **1.94s (1.08x)** |

So the borrow-check pass itself gets about 20% more expensive on the two largest crates — a much larger relative regression than the 5% the workspace showed. Both numbers are true, and the gap between them is the actual lesson: borrow checking is a small enough slice of compilation that a 20% regression inside it lands as a rounding error outside it. In absolute terms the alpha costs an extra 0.30 seconds across 200k lines of Rust.

This is also why we do not expect the eventual toolchain bump to be visible in day-to-day work. The pass that got slower is under a second on our biggest crate, and incremental rebuilds re-run it for one crate at a time.

## Auditing Every Borrow-Checker Workaround

The more interesting question is what the old checker cost us in code. We swept the workspace for borrow-checker workarounds — both by comment (`avoid borrow`, `to satisfy the borrow checker`, `collect first`) and, more importantly, by code shape, since the shapes that matter often carry no comment at all: double lookups, `contains_key` followed by `insert` followed by `get`, `match map.get_mut(k) { ... None => map.insert(...) }`, functions returning `&T`/`&mut T` that mutate the same collection, and every `unsafe` block in the workspace.

The key methodological point: we did not classify these by reading them. Every candidate was reduced to a minimal reproduction and compiled under `-Zpolonius=no` and under the new default, so each verdict is a compiler result rather than an opinion. That mattered — several candidates that looked exactly like Problem Case #3 turned out to compile fine under NLL, and one that we had already written off turned out to be trivially simplifiable.

### What Polonius genuinely unlocks: 3 sites

These three are real Problem Case #3. The natural formulation is rejected by NLL (verified: E0502, E0502, E0506) and accepted by Polonius Alpha:

- **`vize_canon`, `batch/executor/diagnostics.rs` — `original_source`.** The purest instance. It reads a file into a cache and returns `Option<&CachedSource>`. Because initialization is fallible (`read_to_string(path).ok()?`), neither `entry()` nor `or_insert_with` can express it — a closure cannot propagate the `?`. So the code does `contains_key`, then `insert`, then `get`: three lookups where one would do.
- **`vize`, `commands/check/tsconfig_inputs/loader.rs` — `load`.** Worked around with `entry().or_insert_with_key()`, which compiles today but forces an owned `PathBuf` key on every call, including cache hits. Polonius would allow a `get`-based hit path with no allocation.
- **`vize_patina`, `context/reporting.rs` — `sfc_directives`.** Lazy initialization guarded by a separate `sfc_directives_scanned` flag instead of matching on the field itself. This one comes with a caveat: the flag also serves as a negative cache, distinguishing "scanned, found nothing" from "not yet scanned". Removing it would change behavior, not just syntax, so this site is only partly a borrow-checker artifact.

**None of these are fixed in this PR, and that is deliberate.** Vize pins stable 1.95.0 in `rust-toolchain.toml`, in CI, and in every crate's `rust-version`. Writing any of them in the natural form would break the stable build immediately. They are recorded here so the eventual toolchain bump has a ready-made worklist.

### What we could delete today: 6 sites

Separately, the audit found six workarounds that were never required by NLL in the first place. These are fixed in this PR, and each was verified to compile on stable 1.95.0:

- `vize_croquis`, `scope/chain/resolution.rs` (two sites) — cloned a `SmallVec` of parent scope IDs before iterating. A shared reborrow works; the loop body only touches locals.
- `vize_croquis`, `call_graph/analysis.rs` (two sites) — built an intermediate `Vec` of `(index, bool)` pairs before writing back. An index loop reads the `Copy` field, does the lookup, and writes in place, with no allocation.
- `vize_croquis_cf`, `graph.rs` — collected all node IDs into a `Vec` before a DFS that only ever reads the same map. Two shared borrows never conflicted.
- `vize_canon`, `lsp_client/editor_lsp/client.rs` — hoisted a single `as_mut()` above the branch that uses it, removing a redundant second lookup and a duplicated unreachable error branch.

None of these are Polonius wins. They are code that was shaped defensively around a rule that did not apply, and the honest summary is that the audit found more of that than it found genuine NLL limitations.

### What Polonius will never fix

The largest category by far — ten sites — is not a borrow-checker precision problem at all. Every one is the same shape: a shared borrow derived from a context object held across a call that needs `&mut` on that same object. The lint rules are the clearest example, collecting owned strings out of `ctx.analysis()` before calling `ctx.report()`.

Polonius changes _when_ borrows end. It does not make them _finer-grained_. Splitting a borrow of `ctx` into disjoint borrows of `ctx.analysis` and `ctx.diagnostics` is view types, a separate feature still in design. Those ten sites stay exactly as they are, and would stay even under the full `-Zpolonius=next`.

Worth noting for anyone doing a similar audit: a sweep of this kind produces a lot of false positives. More than fifteen sites in this codebase have the exact silhouette of Problem Case #3 — `contains_key` guards, three-lookup cache reads, an `expect("just inserted")` on an impossible branch — and are accepted by NLL today, because the reference never escapes the function. Several of those triple lookups exist to avoid `entry()`'s owned-key allocation, which is a performance decision, not a borrow-checker one. Reading for the shape is not enough; the compiler has to be the judge.

## What This Means for Vize

Nothing changes today. The toolchain stays pinned to stable 1.95.0, and stable users are unaffected until the alpha ships in a stable release.

What we bought with this experiment is certainty about that future bump:

- The entire workspace already borrow-checks cleanly under the new default and under the stricter `-Zpolonius=next`.
- The cost for this workspace is measured, not guessed: about 5% more CPU on a full clean check, less in any incremental flow.
- Three specific sites are queued for simplification the day Polonius stabilizes, with the failing error code recorded for each.
- Six gratuitous workarounds are already gone, independent of any toolchain change.

When stabilization lands, the toolchain bump PR can point at this note instead of re-running the investigation.
