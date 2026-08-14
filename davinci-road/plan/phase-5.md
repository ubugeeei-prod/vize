# Phase 5 — Incrementality Substrate (provisional decomposition)

> [!WARNING]
> Provisional; re-cut at phase-4 exit. Suites referenced as TS-n from
> [test-suites.md](./test-suites.md).

## TODO index

- [ ] P5-1 Stage artifact keys
- [ ] P5-2 Per-SFC summary (declaration fingerprints)
- [ ] P5-3 Global summary (orphan-equivalent facts)
- [ ] P5-4 salsa DB, resident tier
- [ ] P5-5 Snapshot tree under salsa
- [ ] P5-6 Maestro onto cached artifacts (63 sites)
- [ ] P5-7 #698 block-level projection reuse
- [ ] P5-8 #699 Corsa session reuse
- [ ] P5-9 Incremental ≡ clean CI
- [ ] P5-10 Fault-tolerant analysis
- [ ] P5-11 Resource budgets enforced
- [ ] P5-12 LSP conformance multi-client
- [ ] P5-13 JS plugin caching integration
- [ ] P5-14 Phase exit

---

**P5-1 Artifact keys.** Content keys per stage artifact at block granularity:
normalized-structure hash (spans externalized to S0 side tables — identity
excludes presentation), schema version inside every key, span-relative
hashing so edits above a block change zero keys; ambient inputs (tsconfig,
toolchain, feature flags, platform) declared in key manifests — an undeclared
input is a cache-corruption bug. _Accept:_ TS-43 two-platform equality +
edit-locality cases; key manifests documented per artifact.

**P5-2 Per-SFC summary.** α-form export (P4-2) becomes the summary: exported
component signature, prop/emit/slot types, reactivity classes, component
refs — **fingerprinted per declaration** (GHC `.hi`), consumers record used
declarations, invalidation = any used fingerprint changed; **S3 code-shape
decisions type-cannot enter** (body elision by construction). _Accept:_
summary round-trip tests; a hot-path optimization change provably does not
ripple recompilation (fixture scenario).

**P5-3 Global summary.** Orphan-equivalent facts (app-level provide/inject,
global components, dialect-wide directives) in a dedicated global summary
with its own fingerprint — never smuggled into per-file summaries. _Accept:_
scenario test: adding a global component invalidates exactly the consumers
that resolve it.

**P5-4 salsa resident tier.** `crates/vize_maestro` (+ check-server/watch)
moves stage execution onto salsa (0.28+ pinned per charter #39): inputs =
file texts + project config; firewall queries = block content keys + P5-2
summaries (backdating stops edit noise); durability layers (`node_modules`/
tsconfig high, open buffers low); interning GC (`revisions`) + LRU memory
bounds from `budgets.toml`. One-shot CLI stays salsa-free (charter #10).
_Accept:_ TS-42 wiring starts here; RSS ceiling respected under a synthetic
10k-file session (TS-44).

**P5-5 Snapshot tree.** Under salsa, Lean-style snapshot tasks at joints
(SFC header → block → S2 region): reuse rule = old syntax ≡ new syntax ⇒
adopt old subtree; cascade-cancellation tokens through stage tasks;
threads + catch-unwind isolation (not per-file processes). _Accept:_ TS-46
adoption/cancellation scenarios with cache-hit accounting, **plus a
fault-isolation scenario**: a stage task that panics is caught at the
catch-unwind boundary, its file degrades per TS-47, and the server keeps
answering for every other file (extends TS-47).

**P5-6 Maestro migration.** The 63 `parse_sfc` request-path sites consume
cached S1/S2 artifacts via the salsa layer, migrated in waves (hover/
completion → diagnostics → semantic tokens/inlay → the rest), keystroke-cost
perf test per wave. _Accept:_ per-wave TS-44 improvements recorded;
`IdeContext::with_content` string-passing retired.

**P5-7 #698.** Block-level virtual-projection reuse on stage keys —
`VirtualTsCacheKey`'s stubbed `only_template_changed()` logic becomes real
against the P4-5 single projection. _Accept:_ template-only edit reuses
script projection segments (cache-hit assert, TS-46 pattern).

**P5-8 #699.** Corsa `ProjectSession` reuse keyed by the **full P5-1 ambient
manifest** — project identity plus tsconfig content, Corsa/toolchain version,
feature flags, and platform (`CorsaSessionKey` stub realized; a key covering
less than the manifest is a cache-corruption bug by the assurance rule):
spawn/idle-teardown lifecycle, session survives across `vize check` runs via
check-server. _Accept:_ second check
run skips TS project init (timed assert).

**P5-9 Incremental ≡ clean.** CI job: for a corpus shard, apply scripted
edit sequences, compare every resident-tier artifact + diagnostic against a
from-scratch run — **from the first salsa-backed release** (rustc 1.52.1
lesson). _Accept:_ TS-42 mandatory-green; divergence = release blocker.

**P5-10 Fault tolerance.** Analysis proceeds past errors: facts computed for
well-formed regions of broken files (S1 `Unexpected`/`Missing` feed partial
S2 fragments, Lean `PartialTermInfo` pattern); LSP features stay live mid-
edit. _Accept:_ TS-47 scenarios (hover/completion on a file with a parse
error elsewhere).

**P5-11 Budgets enforced.** `budgets.toml` gains resident-tier ceilings as
**numbers, not adjectives**: RSS presets by machine scale, cold-start ms,
idle-CPU (a numeric ceiling — e.g. mean CPU% over a 60 s idle window —
asserted via a documented sampler, /proc or platform equivalent), keystroke
p95 ms — the charter #35 targets pinned against P0 baselines, with the
measurement methodology (sampler, machine preset, corpus project set, run
count) written next to each number so results reproduce. _Accept:_ TS-44
green on the two largest corpus projects (Misskey-class), methodology doc
committed.

**P5-12 Multi-client conformance.** Scenario suite runs against Neovim
(headless `nvim --headless` + lsp attach), Helix, Zed, VS Code: initialize/
hover/completion/diagnostics/rename/formatting exact expectations per client
(TS-45); divergences fixed in vize, not worked around per client. _Accept:_
TS-45 mandatory-green in CI (client versions pinned).

**P5-13 JS plugin caching.** Plugin results enter the artifact-key world:
content key × plugin version × declared demands; invalidation shared with
Rust rules. _Accept:_ TS-51 determinism/caching pre-check (GA in P6).

**P5-14 Phase exit.**

- [ ] TS-42 incremental≡clean green; TS-43 key stability green
- [ ] TS-44 latency/RSS/idle budgets green on large projects; TS-45 conformance green
- [ ] TS-46 adoption accounting; TS-47 fault tolerance
- [ ] #698/#699 closed; `parse_sfc`-per-request pattern gone (grep ceiling: 0 request-path sites)
