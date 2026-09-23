# Phase 6 — Extension Contracts GA

> [!NOTE]
> **Early re-cut 2026-09-21, while phases 3–5 are still live**, under the plan README's [early re-cut rule](./README.md#task-format), in the same shape as [phase 4](./phase-4.md) and [phase 5](./phase-5.md). The maintainer's completion target (**2026-09-23**) and the public presentation at Vue Fes Japan 2026 (**2026-10-24**) make the contract work that needs no later substrate worth starting now: the input-dialect WIT world, the versioning policy, the wasmtime lane, the MoonBit hosting spike and the communications decision. Every task states its **start gate**: _startable now_ or _gated on_ a named earlier-phase task. The exit gate is unchanged and still follows the phase-5 exit (P6-13 depends on P5-14). Tasks whose acceptance needs people outside the repository (the Volt maintainer, a third-party dialect author, the maintainer's own decisions) say so as **review points**, never as green checks.

**The per-task contracts live in [phase-6-tasks.md](./phase-6-tasks.md) (P6-1a…P6-6) and [phase-6-tasks-later.md](./phase-6-tasks-later.md) (P6-7…P6-13)** — Start gate / Lane / Deliverable / Steps / Acceptance / Deps / Non-goals for all 16 tasks. [`davinci-phase6-contract-status.test.ts`](../../tests/tooling/davinci-phase6-contract-status.test.ts) enforces the structure through the shared re-cut checker.

## What the re-cut changed

Measured on `origin/main` (2026-09-21):

1. **Nothing of the contract layer exists yet**: no `contracts/` directory, no WIT files, no `wasmtime` or `wit-bindgen` in the workspace. P6-1 therefore **splits by world**: the input-dialect world needs only S1/S2 (startable now); the expression world needs the fact API and the mapping model (gated on P4-5a); the output-target world needs P3-9's structured emission document (gated on P3-9).
2. **Marquette already classifies compatibility** (`crates/vize_marquette/src/compatibility*`, `canonical.rs`), so P6-8 extends its canonical serialization and additive/breaking classification to contract payloads instead of inventing a second classifier.
3. **The repository already builds MoonBit tooling** (`tools/moon/cmd/*`, with a MoonBit toolchain cached in CI), which makes the P6-4 hosting spike cheap to start now: whether a pinned `moonc` wasm runs under wasmtime with wasm-gc, or needs a Node sidecar behind the same capability boundary. **P6-4 splits**: spike (P6-4a, startable now) and dialect (P6-4b, gated on P4-5b's projection).
4. **The v1 go/no-go document exists** (`docs/release/v1-alpha-go-no-go.md`), so P6-11 wires evidence into its checklist rather than creating one.
5. **Charter #45 is live now, not at phase 6.** The maintainer set a public presentation target; P6-12's decision point has arrived early and is startable now (see [open questions](../open-questions.md#communications-charter-45-and-the-vue-fes-japan-2026-presentation)).

## Carried from earlier phases

| Earlier task | Phase-6 tasks gated | Why                                                                  |
| ------------ | ------------------- | -------------------------------------------------------------------- |
| P4-5a        | P6-1b               | The expression world exports facts and projection mapping rows.      |
| P3-9         | P6-1c               | The output-target world carries the S4 structured emission document. |
| P4-5b        | P6-4b               | MoonBit's projection is an instance of the single projection.        |
| P5-13        | P6-7                | GA hardens the cached plugin path.                                   |
| P5-14        | P6-10, P6-13        | Metrics are reviewed against the finished incrementality substrate.  |

## Start gates and parallel lanes

**Startable now, 10 of 16 tasks:** P6-1a, P6-2, P6-3, P6-4a, P6-5 (behind P6-4b), P6-6 (behind P6-1c), P6-8, P6-9 (behind P6-2, P6-4b, P6-6), P6-11 (behind P6-10), P6-12. **Gated on earlier phases, 6 tasks:** P6-1b, P6-1c, P6-4b, P6-7, P6-10, P6-13.

Lanes own disjoint paths within this phase; registration touchpoints (`Cargo.toml`, workspace members, `test-suites.md`) are the expected rebase conflicts. Lane F follows phase-4 lane O (`crates/vize_vitrine/src/napi/plugin*`) sequentially — P6-7 is gated behind P4-16 through P5-13.

| Lane | Tasks               | Owns (only this lane edits)                                                       |
| ---- | ------------------- | --------------------------------------------------------------------------------- |
| A    | P6-1a, P6-1b, P6-1c | `contracts/wit/`                                                                  |
| B    | P6-2                | `crates/vize_extension_sdk/`, `npm/extension-sdk/`                                |
| C    | P6-3                | `crates/vize_extension_host/`                                                     |
| D    | P6-4a, P6-4b, P6-5  | `crates/vize_dialect_moonbit/`, `davinci-road/plan/exprref-validation.md`         |
| E    | P6-6                | `examples/volt-target/`                                                           |
| F    | P6-7                | `npm/plugin-sdk/`, `crates/vize_vitrine/src/napi/plugin_sdk*`                     |
| G    | P6-8                | `crates/vize_marquette/src/contracts*`, `davinci-road/contracts-compat-policy.md` |
| H    | P6-9                | `tests/external-consumers/`                                                       |
| I    | P6-10, P6-11        | `davinci-road/completion-metrics.md`, `docs/release/v1-alpha-go-no-go.md`         |
| J    | P6-12               | `davinci-road/communications.md`                                                  |
| X    | P6-13               | `davinci-road/plan/phase-6-records/`                                              |

## Critical path

Ordered for the 2026-09-23 completion target; **bold** tasks carry the most presentation value.

1. **P6-12** communications decision and P6-4a MoonBit hosting spike start at once (no dependencies)
2. P6-1a input-dialect world → P6-8 versioning policy → P6-2 prebuilt SDK → P6-3 wasmtime lane
3. P6-1b expression world (at P4-5a) → **P6-4b** MoonBit expressions in templates (at P4-5b) → P6-5 `ExprRef` report
4. P6-1c output world (at P3-9) → P6-6 Volt exercise → P6-9 external validation → P6-10 metrics review (at P5-14) → P6-11 v1 package → P6-13 exit

## TODO index

Each ID links to its contract; the box is checked only in the PR that satisfies its acceptance criteria, with a record under `phase-6-records/`.

- [x] [P6-1a](./phase-6-tasks.md#p6-1a--input-dialect-wit-world-and-capability-handshake) Input-dialect WIT world and capability handshake — lane A · startable now
- [x] [P6-1b](./phase-6-tasks.md#p6-1b--expression-dialect-wit-world) Expression-dialect WIT world — lane A · gated on P4-5a
- [x] [P6-1c](./phase-6-tasks.md#p6-1c--output-target-wit-world) Output-target WIT world — lane A · gated on P3-9
- [x] [P6-2](./phase-6-tasks.md#p6-2--prebuilt-versioned-sdk) Prebuilt versioned SDK — lane B · startable now
- [x] [P6-3](./phase-6-tasks.md#p6-3--in-process-wasmtime-hosting-lane) In-process wasmtime hosting lane — lane C · startable now
- [x] [P6-4a](./phase-6-tasks.md#p6-4a--moonbit-hosting-spike) MoonBit hosting spike — lane D · startable now
- [ ] [P6-4b](./phase-6-tasks.md#p6-4b--moonbit-expression-dialect) MoonBit expression dialect — lane D · gated on P4-5b
- [ ] [P6-5](./phase-6-tasks.md#p6-5--exprref-validation-report) ExprRef validation report — lane D · startable now (behind P6-4b)
- [ ] [P6-6](./phase-6-tasks.md#p6-6--volt-output-target-exercise) Volt output-target exercise — lane E · startable now (behind P6-1c)
- [ ] [P6-7](./phase-6-tasks-later.md#p6-7--js-plugin-sdk-ga) JS plugin SDK GA — lane F · gated on P5-13
- [x] [P6-8](./phase-6-tasks-later.md#p6-8--contract-versioning-and-semver-policy) Contract versioning and semver policy — lane G · startable now
- [ ] [P6-9](./phase-6-tasks-later.md#p6-9--external-consumer-validation) External-consumer validation — lane H · startable now (behind P6-2, P6-4b, P6-6)
- [ ] [P6-10](./phase-6-tasks-later.md#p6-10--completion-metrics-review) Completion-metrics review — lane I · gated on P5-14
- [ ] [P6-11](./phase-6-tasks-later.md#p6-11--v1-go-no-go-input-package) v1 go-no-go input package — lane I · startable now (behind P6-10)
- [ ] [P6-12](./phase-6-tasks-later.md#p6-12--communications-decision) Communications decision — lane J · startable now
- [ ] [P6-13](./phase-6-tasks-later.md#p6-13--phase-exit) Phase exit — lane X · gated on P5-14

---

## Exit gate (machine-checkable)

The three provisional lines are normative and kept verbatim; the re-cut only adds lines.

- [ ] TS-48..51 green; contracts documented with semver policy
- [ ] MoonBit + Volt + JS-rule validations complete; `ExprRef` report closed
- [ ] Completion metrics reconciled; v1 package delivered
- [ ] Charter #45 decision recorded by the maintainer (P6-12); every review-point task names its reviewer and sign-off in its record
- [ ] Standing gates held: TS-1..9, TS-11 empty for every surface the phase touches, TS-10 ratchets tightened only
