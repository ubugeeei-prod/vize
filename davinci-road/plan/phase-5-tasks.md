# Phase 5 — Task contracts, P5-1a through P5-6c

> [!NOTE]
> Full contracts for [Phase 5 — Incrementality Substrate](./phase-5.md), early re-cut 2026-09-21; [phase-5-tasks-later.md](./phase-5-tasks-later.md) continues them under the 350-line source budget. House patterns apply by name: fixtures before behavior (#21), exact oracles only (TS-13), every new bench lands with its measured `allocs` (TS-10), and a cache key covering less than its declared inputs is a corruption bug, not a performance detail.

## P5-1a — Stage artifact keys

**Landed 2026-09-22** — full record: [phase-5-records/p5-1a.md](./phase-5-records/p5-1a.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** A

**Deliverable:** `vize_davinci::key` — content keys per stage artifact (S1 surface tree, S2 page) at block granularity: a normalized-structure hash with spans externalized to S0 side tables (identity excludes presentation), `schema_version` inside every key, and span-relative hashing so an edit above a block changes zero keys of that block.

**Steps:**

- [x] `crates/vize_davinci/src/key.rs` + `key/`: `ArtifactKey { stage, schema_version, hash: [u8; 16] }`; the hash walks the folio `Full` form with spans rebased to the block start _(S2: `impl KeyedArtifact for S2Folio`; S1: `vize_s1_to_s2::key::SurfacePage`, the lossless render; S0 source blocks: `source_block_key`)_
- [x] Edit-locality fixtures: insert above, inside and below a block; reorder blocks; whitespace-only edits _(ten cases over `tests/fixtures/keys/base.vue`, each pinning its exact changed set)_
- [x] Register the TS-43 command in [test-suites.md](./test-suites.md): `cargo test -p vize_davinci --test artifact_keys`

**Acceptance:** TS-43 — identical keys for identical block content regardless of offset, zero key changes for edits above a block, exactly the edited block's keys change otherwise; keys equal across two platforms (the Linux and macOS CI lanes print and compare them); TS-24; TS-1, TS-13.

**Deps:** none (phase-2 exit).

**Non-goals:** ambient inputs (P5-1b); caches that use the keys (P5-4a onward).

## P5-1b — Ambient key manifests

**Landed 2026-09-22** — full record: [phase-5-records/p5-1b.md](./phase-5-records/p5-1b.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** A

**Deliverable:** every cached artifact declares its ambient inputs — tsconfig content, toolchain version, Corsa version, feature flags, platform — in a key manifest, documented per artifact in `davinci-road/plan/key-manifests.md`; an undeclared input is a cache-corruption bug by definition.

**Steps:**

- [x] `KeyManifest` in `crates/vize_davinci/src/key/manifest.rs`, folded into `ArtifactKey` _(`ArtifactKey::with_manifest`; `KeyManifest::fingerprint` for the content-less Corsa session; both refuse a manifest that sets a missing or extra input)_
- [x] A test flips each declared input and asserts the key changes, and asserts an unchanged manifest keeps the key

**Acceptance:** `cargo test -p vize_davinci --test key_manifests` — every declared input changes the key, nothing else does; the manifest doc lists every artifact (a test reads it via `include_str!`); TS-43.

**Deps:** P5-1a.

**Non-goals:** consuming the manifest (P5-8, P5-13).

## P5-2 — Per-SFC summary

**Start gate:** gated on P4-2 — the summary is the fact groups' α form.

**Lane:** B

**Deliverable:** the per-SFC summary built from P4-2's α exports — component signature, prop/emit/slot types, reactivity classes, component references — fingerprinted **per declaration** (the GHC `.hi` model); consumers record which declarations they used and invalidate only on a changed fingerprint. S3 code-shape decisions cannot enter the summary by type (body elision by construction).

**Steps:**

- [ ] `crates/vize_davinci/src/summary.rs`: `SfcSummary` over α pages with per-declaration fingerprints
- [ ] A fixture proves a hot-path optimization change inside a component body does not change any fingerprint

**Acceptance:** `cargo test -p vize_davinci --test sfc_summary` — summary round trip exact (TS-16), the no-ripple fixture green, and a signature change invalidates exactly the recorded users.

**Deps:** P4-2, P5-1a.

**Non-goals:** the global summary (P5-3); salsa wiring (P5-4b).

## P5-3 — Global summary

**Start gate:** gated on P4-3b — global components and provide/inject pairing are project fact groups.

**Lane:** B

**Deliverable:** orphan-equivalent facts — app-level provide/inject, global components, dialect-wide directives — in a dedicated global summary with its own fingerprint, never smuggled into per-file summaries.

**Steps:**

- [ ] `crates/vize_davinci/src/summary/global.rs` over the P4-3b and P4-3f project groups

**Acceptance:** a scenario test: adding a global component invalidates exactly the files that resolve it, and nothing else (TS-46 accounting).

**Deps:** P5-2, P4-3b, P4-3f.

**Non-goals:** workspace-wide invalidation heuristics.

## P5-4a — Salsa resident database skeleton

**Landed 2026-09-22** — full record: [phase-5-records/p5-4a.md](./phase-5-records/p5-4a.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** C

**Deliverable:** the new crate `crates/vize_resident/` — the resident tier's salsa database (charter #10): inputs are file texts and project config, firewall queries are the P5-1a block keys over S1/S2 artifacts, and the one-shot CLI stays salsa-free. salsa is admitted under charter #39 (version pinned, `cargo audit` clean, only this crate and resident-tier features depend on it).

**Steps:**

- [x] Crate + `salsa` pinned in the workspace; a `tests/tooling` check that no crate outside the resident tier depends on `salsa` _(`salsa =0.28.4`, `rayon` off; `davinci-resident-salsa.test.ts`)_
- [x] Queries: `source_text` (input) → `sfc_blocks` → `s1_block` / `s2_page` keyed by block key, with backdating when the key is unchanged _(the firewall is the `Block` tracked struct: content with its S0 key and position are separate tracked fields)_
- [x] Cache-hit accounting counters exposed for TS-46 _(`ResidentDatabase::take_accounting`, read from salsa's event stream)_

**Acceptance:** `cargo test -p vize_resident` — editing one block re-executes only that block's queries (counters pinned exactly); the dependency check green and proven to fail on an injected `salsa` edge in `vize_atelier_sfc`; `cargo audit --deny warnings` green; the one-shot `vize build` binary has no `salsa` in `cargo tree -p vize --no-default-features`.

**Deps:** P5-1a.

**Non-goals:** summaries as firewalls and memory bounds (P5-4b); Maestro migration (P5-6a…P5-6c).

## P5-4b — Summary firewalls, durability and memory bounds

**Start gate:** startable now — no open earlier-phase dependency (waits behind P5-2).

**Lane:** C

**Deliverable:** P5-2 summaries as firewall queries (backdating stops edit noise), durability layers (`node_modules` and tsconfig high, open buffers low), interning GC by revision and LRU memory bounds read from `budgets.toml [resource]`.

**Steps:**

- [ ] Summary queries and durability annotations in `crates/vize_resident/`
- [ ] A synthetic 10k-file session under the P5-11a RSS preset

**Acceptance:** a body-only edit re-executes no dependent file's queries (counters exact); RSS stays under the preset in the synthetic session (TS-44 methodology); TS-42 green on the shard.

**Deps:** P5-4a, P5-2, P5-11a.

**Non-goals:** snapshot tasks (P5-5).

## P5-5 — Snapshot tree under salsa

**Landed 2026-09-22** — full record: [phase-5-records/p5-5.md](./phase-5-records/p5-5.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** C

**Deliverable:** Lean-style snapshot tasks at the joints SFC header → block → S2 region: old syntax ≡ new syntax ⇒ adopt the old subtree; cascade-cancellation tokens through stage tasks; threads plus `catch_unwind` isolation instead of per-file processes.

**Steps:**

- [x] Snapshot adoption and cancellation in `crates/vize_resident/src/snapshot*` _(region joint = a template's root regions; TS-42 runs the snapshot path beside the database)_
- [x] A fault-isolation scenario: a stage task that panics is caught, its file degrades per TS-47, and the server keeps answering for every other file

**Acceptance:** TS-46 adoption and cancellation scenarios with exact cache-hit accounting; the fault-isolation scenario green (extends TS-47).

**Deps:** P5-4a.

**Non-goals:** intra-file parallelism (charter #33 keeps file-parallel).

## P5-6a — Maestro hover, completion and definition wave

**Landed 2026-09-22** — full record: [phase-5-records/p5-6a.md](./phase-5-records/p5-6a.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** D

**Deliverable:** hover, completion and definition request paths (`crates/vize_maestro/src/ide/{hover,completion,definition,template_scope,references}`) read cached S1/S2 artifacts through `vize_resident` instead of calling `parse_sfc` per request; each wave records its keystroke-cost change.

**Steps:**

- [x] Replace the wave's `parse_sfc` calls (hover 7, definition 6, template scope 3, references 4, completion 2 of today's 71) with resident queries _(30 wave sites at landing plus the context's own parse: 81 → 50, pinned by `davinci-maestro-parse-sfc-ceiling.test.ts`)_
- [x] A keystroke perf test per feature on a large corpus project, results recorded in the task record _(exact parse accounting in `server/state/resident/tests.rs`; latency from `examples/keystroke_wave.rs`)_

**Acceptance:** the wave's `parse_sfc` sites gone (a ceiling test pins the remaining count, which only falls); LSP e2e suites for hover/completion/definition exact (TS-7 and the `lsp-*` tooling tests); the recorded keystroke p95 not worse than the P5-11a baseline.

**Deps:** P5-4a.

**Non-goals:** diagnostics (P5-6b); ecosystem services (P5-6c).

## P5-6b — Maestro diagnostics wave

**Landed 2026-09-22** — full record: [phase-5-records/p5-6b.md](./phase-5-records/p5-6b.md).

**Start gate:** gated on P4-5a — diagnostics come from the single post-pass.

**Lane:** D

**Deliverable:** the diagnostics request path reads cached artifacts and the P4-5a post-pass output; its `parse_sfc` calls are gone.

**Steps:**

- [x] Migrate `crates/vize_maestro/src/ide/diagnostics*` outside the P4-5a assembly function _(resident parse, including a memoized rejection; Corsa still goes through `assemble_diagnostics`; ceiling 50 → 47)_

**Acceptance:** diagnostics LSP suites exact; the `parse_sfc` ceiling falls by the wave's count; keystroke-to-diagnostics p95 recorded against the baseline.

**Deps:** P5-6a, P4-5a.

**Non-goals:** projection reuse (P5-7).

## P5-6c — Maestro remaining wave and string passing retired

**Start gate:** gated on P4-10a — ecosystem services move onto providers first.

**Lane:** D

**Deliverable:** semantic tokens, inlay hints, document structure, ecosystem and every remaining request path on cached artifacts; `IdeContext::with_content` string passing (260 call sites) retired.

**Steps:**

- [ ] Migrate the rest; delete `with_content`, `with_content_for_completion` and `with_content_at_position`

**Acceptance:** the `parse_sfc` ceiling at **0** request-path sites; `grep -rn "fn with_content" crates/vize_maestro/src` empty; the LSP suites exact.

**Deps:** P5-6b, P4-10a.

**Non-goals:** new LSP features.
