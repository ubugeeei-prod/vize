# Phase 5 — Incrementality Substrate

> [!NOTE]
> **Early re-cut 2026-09-21, while phases 3 and 4 are still live**, under the plan README's [early re-cut rule](./README.md#task-format) and in the same shape as [phase 4](./phase-4.md). The maintainer's completion target (**2026-09-23**) does not leave room to wait for the phase-4 exit, and a large part of this phase — stage keys, the salsa skeleton, the Maestro request-path waves, Corsa session reuse, the resource baselines and multi-client conformance — reads only S0–S2 artifacts that already exist. Every task states its **start gate**: _startable now_ (no open earlier-phase dependency) or _gated on_ a named phase-4 task. The exit gate is unchanged and still follows the phase-4 exit (P5-14 depends on P4-17). The date orders work; it relaxes no gate or ratchet.

**The per-task contracts live in [phase-5-tasks.md](./phase-5-tasks.md) (P5-1a…P5-6c) and [phase-5-tasks-later.md](./phase-5-tasks-later.md) (P5-7…P5-14)** — Start gate / Lane / Deliverable / Steps / Acceptance / Deps / Non-goals for all 19 tasks. [`davinci-phase5-contract-status.test.ts`](../../tests/tooling/davinci-phase5-contract-status.test.ts) enforces the structure through the shared re-cut checker.

## What the re-cut changed

Measured on `origin/main` (2026-09-21):

1. **The request-path `parse_sfc` count is 71, not 63.** `crates/vize_maestro/src` has 71 non-test `parse_sfc(` calls in 53 files (12 in `ide/`, 7 in `ide/hover/`, 6 in `ide/definition/`, 3 in `ide/template_scope/`, the rest spread across references, completion, diagnostics, ecosystem and server). `IdeContext::with_content` / `IdeContext::new` have 260 call sites. The exit gate's "grep ceiling: 0" applies to the measured set, and **P5-6 splits into three waves** (hover/completion/definition, diagnostics, the rest).
2. **`VirtualTsCacheKey::only_template_changed` is real, not stubbed** — it compares whole-block content hashes (`crates/vize_canon/src/virtual_ts/incremental.rs`) — but nothing reuses projection segments, and whole-block hashes are not the span-relative stage keys P5-1 defines. P5-7 keeps the name's intent and re-keys on P5-1a after P4-5c's single projection exists.
3. **`CorsaSessionKey` is a cache-corruption bug waiting for a consumer.** It keys only on the canonical `tsconfig.json` path (`crates/vize_canon/src/corsa_session_cache.rs`); the assurance rule says a key covering less than the ambient manifest is a corruption bug. P5-8 must widen it to the P5-1b manifest **before** any session is reused.
4. **No salsa (and no wasmtime) in the workspace.** P5-4a admits salsa under charter #39 — pinned, audited (`cargo audit --deny warnings` in `check.yml`), feature-isolated to the resident tier — so **P5-4 splits**: the database skeleton with block-key firewalls (P5-4a, startable now) and summary firewalls, durability layers and memory bounds (P5-4b, behind P5-2).
5. **`budgets.toml [resource]` is an empty reserved section** ("Populated by P5-11"). **P5-11 splits**: baselines and methodology from today's server (P5-11a, startable now) and enforcement on the large corpus projects after the Maestro waves (P5-11b).
6. **Multi-client coverage already exists** — `editor-real-server-{helix,zed}-e2e.test.ts`, `editor-real-server-e2e.test.ts` and the Neovim, Vim and Emacs integration tests — so TS-45 (P5-12) is built on them rather than from scratch.
7. **`vize check-server` exists** (`crates/vize/src/commands/check_server.rs`, 64 lines); P5-8 attaches the session lifecycle to it.

## Carried from phase 4 (so far)

Phase 4 was re-cut the same day; nothing has landed yet. These P4 tasks gate this phase:

| Phase-4 task | Phase-5 tasks gated | Why                                                                                 |
| ------------ | ------------------- | ----------------------------------------------------------------------------------- |
| P4-2         | P5-2                | The per-SFC summary is the α form of each fact group.                               |
| P4-3b        | P5-3                | Global components and provide/inject pairing are project fact groups.               |
| P4-5a        | P5-6b               | The Maestro diagnostics wave consumes the single diagnostic post-pass.              |
| P4-5c        | P5-7                | Block-level reuse (#698) applies to the single projection, not the three old ones.  |
| P4-10a       | P5-6c               | Maestro's ecosystem services move onto providers before the last wave touches them. |
| P4-1a        | P5-10               | Partial S2 fragments produce facts through the fact API.                            |
| P4-16        | P5-13               | Plugin caching needs the spike's API shape.                                         |
| P4-17        | P5-14               | Phase order.                                                                        |

## Start gates and parallel lanes

**Startable now, 11 of 19 tasks:** P5-1a, P5-1b, P5-4a, P5-4b (behind P5-2), P5-5, P5-6a, P5-8, P5-9, P5-11a, P5-11b (behind P5-6c), P5-12. **Gated on phase 4, 8 tasks:** P5-2, P5-3, P5-6b, P5-6c, P5-7, P5-10, P5-13, P5-14.

Lanes own disjoint paths within this phase. Cross-phase: lane D must not edit phase-4 lane paths (`crates/vize_maestro/src/ide/ecosystem*`, `crates/vize_maestro/src/virtual_code*`, `crates/vize_maestro/src/ide/diagnostics/corsa*`) until their owning P4 task has landed — which is exactly why P5-6b and P5-6c are gated. Registration touchpoints (`Cargo.toml`, `lib.rs`, `budgets.toml`, `test-suites.md`) are the expected rebase conflicts.

| Lane | Tasks               | Owns (only this lane edits)                                                                    |
| ---- | ------------------- | ---------------------------------------------------------------------------------------------- |
| A    | P5-1a, P5-1b        | `crates/vize_davinci/src/key*`, `davinci-road/plan/key-manifests.md`                           |
| B    | P5-2, P5-3          | `crates/vize_davinci/src/summary*`                                                             |
| C    | P5-4a, P5-4b, P5-5  | `crates/vize_resident/`                                                                        |
| D    | P5-6a, P5-6b, P5-6c | `crates/vize_maestro/src/ide/`, `crates/vize_maestro/src/server/`                              |
| E    | P5-7                | `crates/vize_canon/src/projection/reuse*`                                                      |
| F    | P5-8                | `crates/vize_canon/src/corsa_session_cache*`, `crates/vize/src/commands/check_server*`         |
| G    | P5-9                | `tools/commands/davinci/incremental-equivalence*`, `.github/workflows/davinci-incremental.yml` |
| H    | P5-10               | `crates/vize_s1_to_s2/src/partial*`, `tests/tooling/lsp-broken-file*`                          |
| I    | P5-11a, P5-11b      | `tools/commands/davinci/resource-budgets*`, `tests/tooling/davinci-resource-budgets*`          |
| J    | P5-12               | `tests/editor-conformance/`, `tests/tooling/editor-conformance*`                               |
| K    | P5-13               | `crates/vize_vitrine/src/napi/plugin_cache*`                                                   |
| X    | P5-14               | `davinci-road/plan/phase-5-records/`                                                           |

## Critical path

Ordered for the 2026-09-23 completion target; **bold** tasks carry the most demo value (visible editor speed and multi-editor support).

1. **P5-12** multi-client conformance and P5-11a resource baselines start at once (no dependencies)
2. P5-1a stage keys → P5-1b ambient manifests → **P5-8** Corsa session reuse
3. P5-1a → P5-4a salsa skeleton → P5-9 incremental ≡ clean from the first salsa-backed release
4. P5-4a → **P5-6a** hover/completion on cached artifacts → P5-6b diagnostics wave → P5-6c remaining wave → P5-11b budgets enforced → P5-14 exit

## TODO index

Each ID links to its contract; the box is checked only in the PR that satisfies its acceptance criteria, with a record under `phase-5-records/`.

- [x] [P5-1a](./phase-5-tasks.md#p5-1a--stage-artifact-keys) Stage artifact keys — lane A · startable now
- [x] [P5-1b](./phase-5-tasks.md#p5-1b--ambient-key-manifests) Ambient key manifests — lane A · startable now
- [ ] [P5-2](./phase-5-tasks.md#p5-2--per-sfc-summary) Per-SFC summary — lane B · gated on P4-2
- [ ] [P5-3](./phase-5-tasks.md#p5-3--global-summary) Global summary — lane B · gated on P4-3b
- [x] [P5-4a](./phase-5-tasks.md#p5-4a--salsa-resident-database-skeleton) Salsa resident database skeleton — lane C · startable now
- [ ] [P5-4b](./phase-5-tasks.md#p5-4b--summary-firewalls-durability-and-memory-bounds) Summary firewalls, durability and memory bounds — lane C · startable now (behind P5-2)
- [x] [P5-5](./phase-5-tasks.md#p5-5--snapshot-tree-under-salsa) Snapshot tree under salsa — lane C · startable now
- [x] [P5-6a](./phase-5-tasks.md#p5-6a--maestro-hover-completion-and-definition-wave) Maestro hover, completion and definition wave — lane D · startable now
- [ ] [P5-6b](./phase-5-tasks.md#p5-6b--maestro-diagnostics-wave) Maestro diagnostics wave — lane D · gated on P4-5a
- [ ] [P5-6c](./phase-5-tasks.md#p5-6c--maestro-remaining-wave-and-string-passing-retired) Maestro remaining wave and string passing retired — lane D · gated on P4-10a
- [ ] [P5-7](./phase-5-tasks-later.md#p5-7--block-level-projection-reuse) Block-level projection reuse — lane E · gated on P4-5c
- [x] [P5-8](./phase-5-tasks-later.md#p5-8--corsa-session-reuse) Corsa session reuse — lane F · startable now
- [x] [P5-9](./phase-5-tasks-later.md#p5-9--incremental-equals-clean-in-ci) Incremental equals clean in CI — lane G · startable now
- [x] [P5-10](./phase-5-tasks-later.md#p5-10--fault-tolerant-analysis) Fault-tolerant analysis — lane H · gated on P4-1a
- [x] [P5-11a](./phase-5-tasks-later.md#p5-11a--resident-resource-baselines-and-methodology) Resident resource baselines and methodology — lane I · startable now
- [ ] [P5-11b](./phase-5-tasks-later.md#p5-11b--resource-budgets-enforced) Resource budgets enforced — lane I · startable now (behind P5-6c)
- [x] [P5-12](./phase-5-tasks-later.md#p5-12--multi-client-lsp-conformance) Multi-client LSP conformance — lane J · startable now
- [ ] [P5-13](./phase-5-tasks-later.md#p5-13--js-plugin-caching) JS plugin caching — lane K · gated on P4-16
- [ ] [P5-14](./phase-5-tasks-later.md#p5-14--phase-exit) Phase exit — lane X · gated on P4-17

---

## Exit gate (machine-checkable)

The four provisional lines are normative and kept verbatim; the re-cut only adds lines.

- [ ] TS-42 incremental≡clean green; TS-43 key stability green
- [ ] TS-44 latency/RSS/idle budgets green on large projects; TS-45 conformance green
- [ ] TS-46 adoption accounting; TS-47 fault tolerance
- [ ] #698/#699 closed; `parse_sfc`-per-request pattern gone (grep ceiling: 0 request-path sites)
- [ ] `budgets.toml [resource]` populated with the methodology beside every number; `CorsaSessionKey` covers the full ambient manifest
- [ ] Standing gates held: TS-1..9, TS-11 empty for every surface the phase touches, TS-10 ratchets tightened only
