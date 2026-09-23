# Phase 5 — Task contracts, P5-7 through P5-14

> [!NOTE]
> Continuation of [phase-5-tasks.md](./phase-5-tasks.md) under the 350-line source budget. Same authority, same format; the TODO index in [phase-5.md](./phase-5.md) links each task to whichever file holds its contract.

## P5-7 — Block-level projection reuse

**Start gate:** gated on P4-5c — reuse applies to the single projection.

**Lane:** E

**Deliverable:** issue #698 closed: the single projection reuses script segments on a template-only edit (and template segments on a script-only edit), keyed by P5-1a block keys through `vize_resident`.

**Steps:**

- [ ] `crates/vize_canon/src/projection/reuse.rs`; retire `VirtualTsCacheKey`'s whole-block hashes in favor of block keys

**Acceptance:** a template-only edit reuses every script segment (cache-hit assert, the TS-46 pattern) and the projection's diagnostics equal a from-scratch run (TS-42); TS-40 unchanged.

**Deps:** P4-5c, P5-1a, P5-4a.

**Non-goals:** Corsa sessions (P5-8).

## P5-8 — Corsa session reuse

**Landed 2026-09-22** — full record: [phase-5-records/p5-8.md](./phase-5-records/p5-8.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** F

**Deliverable:** issue #699 closed: Corsa `ProjectSession`s reused across `vize check` runs through `vize check-server`, keyed by the **full** P5-1b ambient manifest — project identity, tsconfig content, Corsa/toolchain version, feature flags, platform — replacing today's path-only `CorsaSessionKey`, with spawn and idle-teardown lifecycle.

**Steps:**

- [x] Widen `CorsaSessionKey` first (a test flips each manifest input and asserts a different key), then add the session map and lifecycle in `check_server`
- [x] A timed test: the second `vize check` run through the server skips TypeScript project initialization

**Acceptance:** the key test proves every manifest input separates sessions; the second-run test asserts project init did not run (an init counter, not wall time alone); `vize check` diagnostics identical with and without the server (TS-9).

**Deps:** P5-1b.

**Non-goals:** the LSP's own session management (P5-6b).

## P5-9 — Incremental equals clean in CI

**Landed 2026-09-22** — full record: [phase-5-records/p5-9.md](./phase-5-records/p5-9.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** G

**Deliverable:** TS-42 from the first salsa-backed release (the rustc 1.52.1 lesson): for a corpus shard, scripted edit sequences are applied through `vize_resident`, and every resident artifact and diagnostic is compared with a from-scratch run.

**Steps:**

- [x] `rust-script tools/commands/davinci/incremental-equivalence.rs` with committed edit scripts _(six scripts under `incremental-equivalence/scripts/`; the harness is `vize_resident::equivalence`)_
- [x] `.github/workflows/davinci-incremental.yml` required on PRs touching `crates/vize_resident/` _(the `incremental-equivalence` job; its shape is pinned by `davinci-incremental-workflow.test.ts`)_

**Acceptance:** TS-42 green with scope proof (edits applied and artifacts compared are counted; a zero-comparison run fails); a seeded stale-cache bug is caught.

**Deps:** P5-4a.

**Non-goals:** latency (P5-11b).

## P5-10 — Fault-tolerant analysis

**Landed 2026-09-22** — full record: [phase-5-records/p5-10.md](./phase-5-records/p5-10.md).

**Start gate:** gated on P4-1a — partial fragments produce facts through the fact API.

**Lane:** H

**Deliverable:** analysis proceeds past errors: S1 `Unexpected`/`Missing` holes feed partial S2 fragments (P2-8 already keeps them) whose facts are computed for the well-formed regions, so LSP features stay live mid-edit (Lean's `PartialTermInfo` pattern).

**Steps:**

- [x] `crates/vize_s1_to_s2/src/partial.rs` exposes the kept fragments to the fact manager
- [x] TS-47 scenarios in `tests/tooling/lsp-broken-file*`: hover and completion in a file with a parse error elsewhere

**Acceptance:** TS-47 scenarios exact; no diagnostic regression on the well-formed fixtures (TS-9).

**Deps:** P4-1a, P5-4a.

**Non-goals:** error recovery inside expressions.

## P5-11a — Resident resource baselines and methodology

**Landed 2026-09-22** — full record: [phase-5-records/p5-11a.md](./phase-5-records/p5-11a.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** I

**Deliverable:** `budgets.toml [resource]` populated from today's server as ratchets — RSS presets by machine scale, cold-start ms, idle CPU (mean over a 60 s idle window through a documented sampler), keystroke p95 ms — with the methodology beside each number (sampler, machine preset, corpus projects, run count).

**Steps:**

- [x] `rust-script tools/commands/davinci/resource-budgets.rs --measure` and `--check`
- [x] `tests/tooling/davinci-resource-budgets.test.ts` rejects a number without methodology and a loosened number

**Acceptance:** the section populated with numbers, not adjectives; the checker proven to fail on an injected loosening; results reproducible from the recorded command.

**Deps:** none (phase-2 exit).

**Non-goals:** enforcement on the large projects (P5-11b).

## P5-11b — Resource budgets enforced

**Start gate:** startable now — no open earlier-phase dependency (waits behind P5-6c).

**Lane:** I

**Deliverable:** TS-44 enforced on the two largest corpus projects (Misskey-class) after the Maestro waves, with charter #35's "keystroke→diagnostics p95 halved" target pinned against the P5-11a baseline.

**Steps:**

- [ ] CI job running the resource check on the pinned projects

**Acceptance:** TS-44 green on both projects; the halving target met or the miss recorded with its blocker (never a loosened number).

**Deps:** P5-11a, P5-6c, P5-4b.

**Non-goals:** new sampling infrastructure.

## P5-12 — Multi-client LSP conformance

**Landed 2026-09-22** — full record: [phase-5-records/p5-12.md](./phase-5-records/p5-12.md).

**Start gate:** startable now — no open earlier-phase dependency.

**Lane:** J

**Deliverable:** TS-45: one scenario suite — initialize, hover, completion, diagnostics, rename, formatting — with exact expectations run against Neovim (headless), Helix, Zed and VS Code with pinned client versions, building on the existing `editor-real-server-*` suites; divergences are fixed in vize, never worked around per client.

**Steps:**

- [x] `tests/editor-conformance/` scenarios and `tests/tooling/editor-conformance.test.ts`
- [x] CI matrix with pinned client versions

**Acceptance:** TS-45 green in CI for all four clients; a scenario with a deliberately wrong expectation fails.

**Deps:** none (phase-2 exit).

**Non-goals:** Emacs and Vim (covered by existing integration tests, not gated here).

## P5-13 — JS plugin caching

**Start gate:** gated on P4-16 — caching needs the spike's API shape.

**Lane:** K

**Deliverable:** plugin results enter the artifact-key world: content key × plugin version × declared demands, with invalidation shared with Rust rules.

**Steps:**

- [ ] `crates/vize_vitrine/src/napi/plugin_cache.rs` keyed by P5-1b manifests

**Acceptance:** TS-51 pre-check — deterministic across runs and content-key cached, with the plugin version and declared demands in the key.

**Deps:** P4-16, P5-1b.

**Non-goals:** GA (P6-7).

## P5-14 — Phase exit

**Start gate:** gated on P4-17 — phase order.

**Lane:** X

**Deliverable:** the exit gate in [phase-5.md](./phase-5.md) evaluated inline — ticked only when satisfied, unticked lines naming their blocker, nothing softened.

**Steps:**

- [ ] Evaluate every gate line with evidence; record the TS-42 and TS-44 runs

**Acceptance:** every exit-gate line ticked with evidence or carrying a named blocker.

**Deps:** every other phase-5 task, P4-17.

**Non-goals:** re-cutting phase 6.
